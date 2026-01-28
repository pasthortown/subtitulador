//! Adapter: AudioCapture - Captura de audio usando CPAL.

use std::sync::{Arc, Mutex};
use std::process::Command;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig, SampleRate};
use tracing::{info, error};

use crate::domain::AudioPower;
use crate::domain::ports::inbound::{AudioCapturePort, AudioDeviceInfo};

/// Tipo de backend de captura.
#[derive(Debug, Clone, PartialEq)]
enum CaptureBackend {
    /// Captura usando CPAL/ALSA.
    Cpal,
    /// Captura usando PipeWire (parecord).
    PipeWire(String), // device name
}

/// Adaptador para captura de audio.
pub struct AudioCapture {
    host: Host,
    device: Option<Device>,
    stream: Option<Stream>,
    sample_rate: u32,
    samples_buffer: Arc<Mutex<Vec<f32>>>,
    backend: CaptureBackend,
    pw_process: Option<std::process::Child>,
}

impl AudioCapture {
    /// Crea un nuevo capturador de audio.
    pub fn new(sample_rate: u32) -> Result<Self> {
        let host = cpal::default_host();

        Ok(Self {
            host,
            device: None,
            stream: None,
            sample_rate,
            samples_buffer: Arc::new(Mutex::new(Vec::new())),
            backend: CaptureBackend::Cpal,
            pw_process: None,
        })
    }

    /// Obtiene información de dispositivos desde PipeWire/PulseAudio.
    fn get_pipewire_sources() -> HashMap<String, (String, String)> {
        let output = match Command::new("pactl").args(["list", "sources"]).output() {
            Ok(o) if o.status.success() => o,
            _ => return HashMap::new(),
        };

        Self::parse_pactl_output(&String::from_utf8_lossy(&output.stdout))
    }

    /// Parsea la salida de pactl list sources.
    fn parse_pactl_output(text: &str) -> HashMap<String, (String, String)> {
        let mut sources = HashMap::new();
        let mut current_name = String::new();

        for line in text.lines() {
            let line = line.trim();

            if let Some(name) = Self::parse_pactl_name(line) {
                current_name = name;
            } else if let Some(desc) = Self::parse_pactl_description(line) {
                if !current_name.is_empty() {
                    let device_type = Self::get_device_type(&current_name);
                    sources.insert(current_name.clone(), (desc, device_type));
                }
            }
        }

        sources
    }

    /// Extrae el nombre de una línea de pactl.
    fn parse_pactl_name(line: &str) -> Option<String> {
        if line.starts_with("Nombre:") || line.starts_with("Name:") {
            line.split(':').nth(1).map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    /// Extrae la descripción de una línea de pactl.
    fn parse_pactl_description(line: &str) -> Option<String> {
        if line.starts_with("Descripción:") || line.starts_with("Description:") {
            line.split(':').nth(1).map(|s| s.trim().to_string())
        } else {
            None
        }
    }

    /// Determina el tipo de dispositivo basado en el nombre.
    fn get_device_type(name: &str) -> String {
        if name.contains("bluez_input") {
            "Bluetooth (micrófono)"
        } else if name.contains("bluez_output") && name.contains("monitor") {
            "Bluetooth (monitor)"
        } else if name.contains("monitor") {
            "Monitor"
        } else if name.contains("alsa_input") {
            "Micrófono"
        } else {
            "Audio"
        }.to_string()
    }

    /// Obtiene el dispositivo por defecto de PipeWire/PulseAudio.
    fn get_default_source() -> Option<String> {
        let output = Command::new("pactl")
            .args(["get-default-source"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        None
    }

    /// Construye lista de dispositivos desde PipeWire.
    fn build_pipewire_devices(
        sources: &HashMap<String, (String, String)>,
        default_source: &Option<String>,
    ) -> Vec<AudioDeviceInfo> {
        sources.iter()
            .filter(|(name, _)| !name.contains("monitor"))
            .map(|(name, (description, device_type))| {
                let is_default = default_source.as_ref().map(|d| d == name).unwrap_or(false);
                AudioDeviceInfo {
                    name: name.clone(),
                    display_name: format!("{} - {}", description, device_type),
                    description: description.clone(),
                    is_default,
                }
            })
            .collect()
    }

    /// Construye lista de dispositivos desde CPAL.
    fn build_cpal_devices(&self) -> Result<Vec<AudioDeviceInfo>> {
        let default_name = self.host.default_input_device()
            .and_then(|d| d.name().ok());

        let devices = self.host.input_devices()?
            .filter_map(|device| device.name().ok())
            .filter(|name| !Self::is_filtered_cpal_device(name))
            .map(|name| {
                let is_default = default_name.as_ref().map(|d| d == &name).unwrap_or(false);
                AudioDeviceInfo {
                    name: name.clone(),
                    display_name: name.clone(),
                    description: String::new(),
                    is_default,
                }
            })
            .collect();

        Ok(devices)
    }

    /// Verifica si un dispositivo CPAL debe ser filtrado.
    fn is_filtered_cpal_device(name: &str) -> bool {
        name.contains("dsnoop") || name.contains("dmix") ||
        name.contains("null") || name.contains("surround")
    }

    /// Ordena los dispositivos: default primero, luego alfabéticamente.
    fn sort_devices(devices: &mut Vec<AudioDeviceInfo>) {
        devices.sort_by(|a, b| {
            match (a.is_default, b.is_default) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.display_name.cmp(&b.display_name),
            }
        });
    }

    /// Selecciona el dispositivo por defecto.
    pub fn select_default_device(&mut self) -> Result<()> {
        let device = self.host.default_input_device()
            .ok_or_else(|| anyhow!("No hay dispositivo de entrada por defecto"))?;

        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        info!("Dispositivo por defecto seleccionado: {}", name);

        self.device = Some(device);
        Ok(())
    }

    /// Inicia captura usando CPAL/ALSA.
    fn start_capture_cpal(&mut self) -> Result<()> {
        let device = self.device.as_ref()
            .ok_or_else(|| anyhow!("No hay dispositivo seleccionado"))?;

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples_buffer = Arc::clone(&self.samples_buffer);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buffer = samples_buffer.lock().unwrap();
                buffer.extend_from_slice(data);
            },
            |err| {
                error!("Error en stream de audio: {}", err);
            },
            None,
        )?;

        stream.play()?;
        self.stream = Some(stream);

        info!("Captura CPAL iniciada");
        Ok(())
    }

    /// Inicia captura usando PipeWire/PulseAudio.
    fn start_capture_pipewire(&mut self, device_name: String) -> Result<()> {
        use std::io::Read;
        use std::process::Stdio;

        let samples_buffer = Arc::clone(&self.samples_buffer);
        let sample_rate = self.sample_rate;

        // Iniciar parec con baja latencia
        // parec es más simple que parecord y tiene menos overhead
        let mut child = Command::new("parec")
            .args([
                "--device", &device_name,
                "--rate", &sample_rate.to_string(),
                "--channels", "1",
                "--format", "float32le",
                "--latency-msec", "20",  // Latencia mínima de 20ms
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| anyhow!("Error al iniciar parec: {}", e))?;

        // Leer audio en un hilo separado con buffer pequeño
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow!("No se pudo obtener stdout de parec"))?;

        std::thread::spawn(move || {
            let mut reader = std::io::BufReader::with_capacity(640, stdout); // ~10ms de buffer a 16kHz
            let mut buf = [0u8; 640]; // 160 samples * 4 bytes = 10ms a 16kHz

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // Convertir bytes a f32
                        let samples: Vec<f32> = buf[..n]
                            .chunks_exact(4)
                            .map(|chunk| {
                                let bytes: [u8; 4] = chunk.try_into().unwrap();
                                f32::from_le_bytes(bytes)
                            })
                            .collect();

                        let mut buffer = samples_buffer.lock().unwrap();
                        buffer.extend(samples);
                    }
                    Err(e) => {
                        error!("Error leyendo audio: {}", e);
                        break;
                    }
                }
            }
        });

        self.pw_process = Some(child);
        info!("Captura PipeWire iniciada (baja latencia) para: {}", device_name);
        Ok(())
    }
}

impl AudioCapturePort for AudioCapture {
    fn list_input_devices(&self) -> Result<Vec<AudioDeviceInfo>> {
        let pw_sources = Self::get_pipewire_sources();
        let default_source = Self::get_default_source();

        info!("Fuentes PipeWire encontradas: {}", pw_sources.len());

        let mut devices = if !pw_sources.is_empty() {
            Self::build_pipewire_devices(&pw_sources, &default_source)
        } else {
            self.build_cpal_devices()?
        };

        Self::sort_devices(&mut devices);
        Ok(devices)
    }

    fn select_device(&mut self, device_name: &str) -> Result<()> {
        // Verificar si es un dispositivo PipeWire (bluez, alsa_input, etc.)
        if device_name.starts_with("bluez_") ||
           device_name.starts_with("alsa_input.") ||
           device_name.starts_with("alsa_output.") {
            info!("Dispositivo PipeWire seleccionado: {}", device_name);
            self.backend = CaptureBackend::PipeWire(device_name.to_string());
            return Ok(());
        }

        // Buscar en dispositivos CPAL/ALSA
        for device in self.host.input_devices()? {
            if let Ok(name) = device.name() {
                if name == device_name {
                    info!("Dispositivo CPAL seleccionado: {}", name);
                    self.device = Some(device);
                    self.backend = CaptureBackend::Cpal;
                    return Ok(());
                }
            }
        }

        Err(anyhow!("Dispositivo no encontrado: {}", device_name))
    }

    fn start_capture(&mut self) -> Result<()> {
        match &self.backend {
            CaptureBackend::Cpal => self.start_capture_cpal(),
            CaptureBackend::PipeWire(device) => self.start_capture_pipewire(device.clone()),
        }
    }

    fn stop_capture(&mut self) {
        // Detener stream CPAL
        if let Some(stream) = self.stream.take() {
            drop(stream);
            info!("Captura CPAL detenida");
        }

        // Detener proceso PipeWire (parec)
        if let Some(mut process) = self.pw_process.take() {
            let _ = process.kill();
            let _ = process.wait();
            info!("Captura PipeWire detenida");
        }

        // Limpiar cualquier proceso parec huérfano
        let _ = Command::new("pkill")
            .args(["-f", "parec"])
            .output();
    }

    fn take_samples(&self) -> Vec<f32> {
        let mut buffer = self.samples_buffer.lock().unwrap();
        std::mem::take(&mut *buffer)
    }

    fn current_power(&self) -> AudioPower {
        let buffer = self.samples_buffer.lock().unwrap();
        if buffer.is_empty() {
            AudioPower::zero()
        } else {
            AudioPower::from_samples(&buffer)
        }
    }

    fn clear_buffer(&self) {
        let mut buffer = self.samples_buffer.lock().unwrap();
        buffer.clear();
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        self.stop_capture();
    }
}
