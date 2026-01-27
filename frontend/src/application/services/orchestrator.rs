//! Orquestador de transcripción - Coordina captura, transcripción y traducción.

use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{bounded, Receiver, Sender};
use anyhow::Result;
use tracing::{info, warn, error};

use crate::domain::{
    AudioBuffer, Subtitle, CalibrationData, SilenceDetector,
    Language, AudioPower, Duration,
};
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::adapters::{
    HttpTranscriptionClient,
    HttpTranslationClient,
};

/// Mensaje de subtítulo para la UI.
#[derive(Debug, Clone)]
pub struct SubtitleMessage {
    pub text: String,
    pub duration: Duration,
}

/// Orquestador principal de transcripción.
pub struct TranscriptionOrchestrator {
    config: AppConfig,
    transcription_client: HttpTranscriptionClient,
    translation_client: HttpTranslationClient,
    subtitle_sender: Sender<SubtitleMessage>,
    subtitle_receiver: Receiver<SubtitleMessage>,
    calibration: Arc<Mutex<CalibrationData>>,
    is_running: Arc<Mutex<bool>>,
}

impl TranscriptionOrchestrator {
    /// Crea un nuevo orquestador.
    pub fn new(config: AppConfig) -> Result<Self> {
        let transcription_client = HttpTranscriptionClient::new(&config.backend_url);
        let translation_client = HttpTranslationClient::new(&config.translation_url);

        let (sender, receiver) = bounded(100);

        Ok(Self {
            config,
            transcription_client,
            translation_client,
            subtitle_sender: sender,
            subtitle_receiver: receiver,
            calibration: Arc::new(Mutex::new(CalibrationData::default_values())),
            is_running: Arc::new(Mutex::new(false)),
        })
    }

    /// Configura la calibración.
    pub fn set_calibration(&self, calibration: CalibrationData) {
        *self.calibration.lock().unwrap() = calibration;
    }

    /// Obtiene la calibración actual.
    pub fn calibration(&self) -> CalibrationData {
        self.calibration.lock().unwrap().clone()
    }

    /// Procesa un buffer de audio.
    pub fn process_buffer(&self, buffer: &AudioBuffer) -> Result<()> {
        if !buffer.is_valid_for_transcription() {
            return Ok(());
        }

        info!("Procesando buffer: {} samples, {} ms",
              buffer.num_samples(), buffer.duration().as_millis());

        // Transcribir
        let result = self.transcription_client.transcribe(buffer)?;

        if result.text.is_empty() {
            return Ok(());
        }

        info!("Transcripción: '{}' (confianza: {:.2})", result.text, result.confidence);

        // Traducir si es necesario
        let final_text = if self.config.input_language != self.config.output_language {
            let source = Language::new(&self.config.input_language)?;
            let target = Language::new(&self.config.output_language)?;

            match self.translation_client.translate(&result.text, &source, &target) {
                Ok(translated) => {
                    info!("Traducción: '{}'", translated);
                    translated
                }
                Err(e) => {
                    warn!("Error en traducción: {}", e);
                    result.text
                }
            }
        } else {
            result.text
        };

        // Enviar subtítulo a la UI
        let message = SubtitleMessage {
            text: final_text,
            duration: buffer.duration(),
        };

        if let Err(e) = self.subtitle_sender.try_send(message) {
            warn!("No se pudo enviar subtítulo: {}", e);
        }

        Ok(())
    }

    /// Recibe el próximo subtítulo (no bloqueante).
    pub fn receive_subtitle(&self) -> Option<SubtitleMessage> {
        self.subtitle_receiver.try_recv().ok()
    }

    /// Verifica si está corriendo.
    pub fn is_running(&self) -> bool {
        *self.is_running.lock().unwrap()
    }

    /// Inicia el orquestador.
    pub fn start(&self) {
        *self.is_running.lock().unwrap() = true;
        info!("Orquestador iniciado");
    }

    /// Detiene el orquestador.
    pub fn stop(&self) {
        *self.is_running.lock().unwrap() = false;
        info!("Orquestador detenido");
    }
}
