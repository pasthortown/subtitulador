//! Aplicación de subtítulos con egui.

use std::collections::VecDeque;
use std::process::Command;
use std::time::{Instant, Duration as StdDuration};
use eframe::egui;
use tracing::{info, debug, warn, error};

use crate::domain::{Subtitle, AudioBuffer, Language, SilenceDetector, AudioPower};
use crate::infrastructure::config::AppConfig;
use crate::infrastructure::adapters::{AudioCapture, AudioDeviceInfo};
use crate::application::services::TranscriptionOrchestrator;

/// Mínimo de samples para transcribir (300ms a 16kHz = 4800 samples).
const MIN_SAMPLES_FOR_TRANSCRIPTION: usize = 4800;

/// Máximo de samples en buffer (15 segundos a 16kHz - seguridad).
const MAX_SAMPLES_IN_BUFFER: usize = 240000;

/// Tamaño del chunk para análisis de potencia (100ms a 16kHz = 1600 samples).
/// Igual que Python: CHUNK_MS = 100.
const CHUNK_SIZE_FOR_ANALYSIS: usize = 1600;

/// Ancho de la barra de subtítulos.
const SUBTITLE_BAR_WIDTH: f32 = 900.0;

/// Distancia desde el borde inferior de la pantalla.
const BOTTOM_MARGIN: f32 = 50.0;

/// Transparencia del fondo (0-255, donde 204 = 80%).
const BACKGROUND_ALPHA: u8 = 204;

/// Dimensiones por defecto del monitor.
const DEFAULT_MONITOR: (f32, f32, f32, f32) = (1920.0, 1080.0, 0.0, 0.0);

/// Obtiene las dimensiones del monitor principal usando xrandr.
/// Retorna (width, height, x, y).
fn get_primary_monitor() -> (f32, f32, f32, f32) {
    let output = match Command::new("xrandr").arg("--query").output() {
        Ok(o) => o,
        Err(_) => return log_default_monitor(),
    };

    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return log_default_monitor(),
    };

    for line in stdout.lines() {
        if let Some(geometry) = parse_primary_line(line) {
            return geometry;
        }
    }

    log_default_monitor()
}

/// Registra y retorna las dimensiones por defecto.
fn log_default_monitor() -> (f32, f32, f32, f32) {
    info!("No se pudo detectar monitor primario, usando valores por defecto");
    DEFAULT_MONITOR
}

/// Parsea una línea de xrandr buscando el monitor primario.
fn parse_primary_line(line: &str) -> Option<(f32, f32, f32, f32)> {
    if !line.contains(" primary ") {
        return None;
    }

    for part in line.split_whitespace() {
        if let Some(geometry) = parse_geometry(part) {
            return Some(geometry);
        }
    }
    None
}

/// Parsea una geometría en formato WxH+X+Y.
fn parse_geometry(part: &str) -> Option<(f32, f32, f32, f32)> {
    if !part.contains('x') || !part.contains('+') {
        return None;
    }

    let geom: Vec<&str> = part.split('+').collect();
    if geom.len() < 3 {
        return None;
    }

    let dims: Vec<&str> = geom[0].split('x').collect();
    if dims.len() < 2 {
        return None;
    }

    let w = dims[0].parse().ok()?;
    let h = dims[1].parse().ok()?;
    let x = geom[1].parse().ok()?;
    let y = geom[2].parse().ok()?;

    info!("Monitor primario detectado: {}x{} en ({}, {})", w, h, x, y);
    Some((w, h, x, y))
}


/// Aplicación principal de subtítulos.
pub struct SubtitleApp {
    orchestrator: TranscriptionOrchestrator,
    config: AppConfig,
    audio_capture: Option<AudioCapture>,
    silence_detector: SilenceDetector,
    audio_buffer: Vec<f32>,
    active_subtitles: VecDeque<Subtitle>,
    pending_subtitles: VecDeque<Subtitle>,
    frames_since_last_log: u32,
    is_capturing: bool,
    window_positioned: bool,
    monitor_geometry: (f32, f32, f32, f32),
    /// Estado actual: true = hablando, false = silencio
    is_speaking: bool,
    /// Buffer temporal para acumular samples antes de analizar potencia.
    /// Similar al CHUNK_MS de Python (100ms = 1600 samples a 16kHz).
    chunk_buffer: Vec<f32>,
    /// Nombre del dispositivo de audio actual.
    device_name: String,
    /// Mostrar selector de dispositivos.
    show_device_selector: bool,
    /// Lista de dispositivos disponibles.
    available_devices: Vec<AudioDeviceInfo>,
    /// Índice del dispositivo siendo navegado en el diálogo.
    device_nav_index: usize,
    /// Indica si hay un dispositivo seleccionado (no solo por defecto).
    device_selected: bool,
    /// Mostrar diálogo de recalibración.
    show_calibration_dialog: bool,
    /// Mostrar diálogo de confirmación para salir.
    show_exit_dialog: bool,
    /// Estado de recalibración en curso.
    recalibration_state: Option<RecalibrationState>,
    /// Indica si se ha calibrado el silencio.
    silence_calibrated: bool,
    /// Indica si se ha calibrado la voz.
    voice_calibrated: bool,
    /// Potencia actual del micrófono (para mostrar nivel).
    current_mic_level: f32,
}

/// Tipo de recalibración.
#[derive(Clone, Copy, PartialEq)]
enum RecalibrationType {
    Silence,
    Voice,
}

/// Estado de una recalibración en curso.
struct RecalibrationState {
    calibration_type: RecalibrationType,
    start_time: Instant,
    samples: Vec<f32>,
}

impl SubtitleApp {
    /// Crea una nueva aplicación sin dispositivo seleccionado.
    pub fn new(
        orchestrator: TranscriptionOrchestrator,
        config: AppConfig,
    ) -> Self {
        info!("Creando SubtitleApp - esperando selección de dispositivo");

        let audio_capture = AudioCapture::new(config.sample_rate).ok();

        // Cargar lista de dispositivos disponibles al inicio
        let available_devices = if let Some(ref capture) = audio_capture {
            capture.list_input_devices().unwrap_or_default()
        } else {
            Vec::new()
        };

        let calibration = orchestrator.calibration();
        info!("Usando calibración por defecto: {}", calibration);
        let silence_detector = SilenceDetector::new(calibration.clone());

        let monitor_geometry = get_primary_monitor();

        orchestrator.start();

        Self {
            orchestrator,
            config,
            audio_capture,
            silence_detector,
            audio_buffer: Vec::new(),
            active_subtitles: VecDeque::new(),
            pending_subtitles: VecDeque::new(),
            frames_since_last_log: 0,
            is_capturing: false,
            window_positioned: false,
            monitor_geometry,
            is_speaking: false,
            chunk_buffer: Vec::with_capacity(CHUNK_SIZE_FOR_ANALYSIS),
            device_name: String::new(),
            show_device_selector: false,
            available_devices,
            device_nav_index: 0,
            device_selected: false, // Sin dispositivo seleccionado al inicio
            show_calibration_dialog: false,
            show_exit_dialog: false,
            recalibration_state: None,
            silence_calibrated: false,
            voice_calibrated: false,
            current_mic_level: 0.0,
        }
    }

    /// Posiciona la ventana en la parte inferior centrada del monitor primario.
    fn position_window(&mut self, ctx: &egui::Context) {
        if self.window_positioned {
            return;
        }

        let (monitor_width, monitor_height, monitor_x, monitor_y) = self.monitor_geometry;
        let content_height = self.calculate_content_height().max(100.0);

        let x = monitor_x + (monitor_width - SUBTITLE_BAR_WIDTH) / 2.0;
        let y = monitor_y + monitor_height - content_height - BOTTOM_MARGIN;

        info!(
            "Posicionando ventana: {}x{} en ({}, {})",
            SUBTITLE_BAR_WIDTH, content_height, x, y
        );

        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
            egui::pos2(x, y)
        ));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
            egui::vec2(SUBTITLE_BAR_WIDTH, content_height)
        ));

        self.window_positioned = true;
    }

    /// Calcula la altura necesaria para el contenido.
    fn calculate_content_height(&self) -> f32 {
        // Base: status bar + padding
        let base_height = 50.0;

        // Cada subtítulo puede ocupar múltiples líneas
        // Estimamos ~35px por línea de subtítulo
        let subtitle_height: f32 = if self.active_subtitles.is_empty() {
            35.0 // Espacio mínimo para un subtítulo
        } else {
            self.active_subtitles.iter()
                .map(|s| {
                    // Estimar líneas basado en longitud del texto
                    let chars = s.text().len();
                    let lines = ((chars as f32 / 50.0).ceil() as usize).max(1);
                    lines as f32 * 35.0
                })
                .sum()
        };

        base_height + subtitle_height
    }

    /// Actualiza el tamaño de la ventana según el contenido.
    fn update_window_size(&self, ctx: &egui::Context) {
        let (monitor_width, monitor_height, monitor_x, monitor_y) = self.monitor_geometry;
        let content_height = self.calculate_content_height().max(100.0);

        let x = monitor_x + (monitor_width - SUBTITLE_BAR_WIDTH) / 2.0;
        let y = monitor_y + monitor_height - content_height - BOTTOM_MARGIN;

        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
            egui::pos2(x, y)
        ));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
            egui::vec2(SUBTITLE_BAR_WIDTH, content_height)
        ));
    }

    /// Procesa el audio capturado.
    /// Algoritmo basado en Python:
    /// 1. Acumular samples hasta tener un chunk de 100ms (1600 samples)
    /// 2. Calcular potencia del chunk completo
    /// 3. Si es voz, agregar al buffer principal
    /// 4. Si es silencio sostenido (750ms), procesar el buffer
    fn process_audio(&mut self) {
        if !self.is_capturing {
            return;
        }

        let Some(ref audio_capture) = self.audio_capture else {
            return;
        };

        // Obtener samples nuevos y agregarlos al chunk_buffer
        let new_samples = audio_capture.take_samples();
        if !new_samples.is_empty() {
            self.chunk_buffer.extend(new_samples);
        }

        // Solo procesar cuando tengamos un chunk completo de 100ms (como Python)
        if self.chunk_buffer.len() < CHUNK_SIZE_FOR_ANALYSIS {
            return;
        }

        // Extraer el chunk para análisis
        let chunk: Vec<f32> = self.chunk_buffer.drain(..CHUNK_SIZE_FOR_ANALYSIS).collect();

        // Calcular potencia del chunk (como Python: np.sqrt(np.mean(chunk ** 2)))
        let current_power = AudioPower::from_samples(&chunk);

        // Usar calibración para determinar si es silencio
        let is_silence = self.silence_detector.calibration().is_silence(current_power);

        // Log cada ~10 chunks (~1 segundo a 100ms/chunk)
        self.frames_since_last_log += 1;
        if self.frames_since_last_log >= 10 {
            let buffer_seconds = self.audio_buffer.len() as f32 / self.config.sample_rate as f32;
            debug!(
                "Buffer: {:.1}s | Potencia: {:.6} | Umbral: {:.6} | {}",
                buffer_seconds,
                current_power.value(),
                self.silence_detector.calibration().silence_threshold().value(),
                if is_silence { "SILENCIO" } else { "VOZ" }
            );
            self.frames_since_last_log = 0;
        }

        // Algoritmo igual que Python:
        // Si NO es silencio (es voz), añadir al buffer
        if !is_silence {
            self.audio_buffer.extend(chunk);
            self.is_speaking = true;
            self.silence_detector.reset();
        } else {
            // Es silencio
            self.is_speaking = false;
        }

        // Verificar si el silencio se ha sostenido (fin de frase)
        // Solo si hay contenido en el buffer
        if !self.audio_buffer.is_empty() && self.audio_buffer.len() >= MIN_SAMPLES_FOR_TRANSCRIPTION {
            let silence_sustained = self.silence_detector.process(current_power);

            if silence_sustained {
                info!(
                    "Silencio sostenido 750ms - procesando buffer ({} samples, {:.1}s)",
                    self.audio_buffer.len(),
                    self.audio_buffer.len() as f32 / self.config.sample_rate as f32
                );
                self.send_for_transcription();
            }
        }

        // Seguridad: forzar procesamiento si buffer muy grande
        if self.audio_buffer.len() >= MAX_SAMPLES_IN_BUFFER {
            warn!("Buffer máximo 15s alcanzado - procesando");
            self.send_for_transcription();
        }
    }

    /// Envía el buffer actual para transcripción.
    fn send_for_transcription(&mut self) {
        if self.audio_buffer.len() < MIN_SAMPLES_FOR_TRANSCRIPTION {
            return;
        }

        let samples = std::mem::take(&mut self.audio_buffer);
        let duration_ms = (samples.len() as u64 * 1000) / self.config.sample_rate as u64;

        info!(
            "Enviando {} samples ({} ms) para transcripción",
            samples.len(),
            duration_ms
        );

        let language = Language::new(&self.config.input_language)
            .unwrap_or_else(|_| Language::spanish());

        let buffer = AudioBuffer::new(samples, language);

        match self.orchestrator.process_buffer(&buffer) {
            Ok(()) => {
                debug!("Buffer enviado correctamente al orquestador");
            }
            Err(e) => {
                error!("Error al procesar buffer: {}", e);
            }
        }

        // Resetear detector después de enviar
        self.silence_detector.reset();
        self.is_speaking = false;
    }

    /// Procesa mensajes de subtítulos del orquestador.
    fn process_messages(&mut self) {
        while let Some(msg) = self.orchestrator.receive_subtitle() {
            info!("Subtítulo recibido: '{}' (duración: {} ms)", msg.text, msg.duration.as_millis());
            let subtitle = Subtitle::new(msg.text, msg.duration);
            self.pending_subtitles.push_back(subtitle);
        }
    }

    /// Activa subtítulos pendientes según la lógica FIFO.
    fn activate_pending(&mut self) {
        while let Some(mut subtitle) = self.pending_subtitles.pop_front() {
            if self.active_subtitles.is_empty() {
                subtitle.show();
                info!(
                    "[UI] Mensaje activado: '{}' (expira en {:.1}s)",
                    subtitle.text(),
                    subtitle.display_duration().as_secs_f64()
                );
                self.active_subtitles.push_back(subtitle);
                continue;
            }

            let can_show = self.active_subtitles.back()
                .map(|last| last.can_show_next())
                .unwrap_or(false);

            if can_show {
                let last_expire = self.active_subtitles.back()
                    .and_then(|last| last.expire_time());

                if let Some(expire) = last_expire {
                    subtitle.show_after(expire);
                    let time_until_expire = subtitle.expire_time()
                        .map(|e| e.saturating_duration_since(Instant::now()).as_secs_f32())
                        .unwrap_or(0.0);
                    info!(
                        "[UI] Mensaje activado: '{}' (expira en {:.1}s)",
                        subtitle.text(),
                        time_until_expire
                    );
                } else {
                    subtitle.show();
                }
                self.active_subtitles.push_back(subtitle);
            } else {
                self.pending_subtitles.push_front(subtitle);
                break;
            }
        }
    }

    /// Elimina subtítulos expirados.
    fn remove_expired(&mut self) {
        self.active_subtitles.retain(|s| !s.is_expired());
    }

    /// Dibuja los subtítulos con soporte para múltiples líneas.
    fn draw_subtitles(&self, ui: &mut egui::Ui) {
        let max_text_width = SUBTITLE_BAR_WIDTH - 40.0;

        for subtitle in &self.active_subtitles {
            let opacity = subtitle.opacity();
            let alpha = (opacity * 255.0) as u8;

            let text_color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
            let shadow_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, alpha);

            let text = subtitle.text();
            let font = egui::FontId::proportional(28.0);

            // Layout con wrap automático
            let galley = ui.painter().layout(
                text.to_string(),
                font.clone(),
                text_color,
                max_text_width,
            );

            let galley_shadow = ui.painter().layout(
                text.to_string(),
                font,
                shadow_color,
                max_text_width,
            );

            // Centrar horizontalmente
            let text_width = galley.rect.width();
            let x_offset = (ui.available_width() - text_width) / 2.0;
            let pos = ui.cursor().left_top() + egui::vec2(x_offset, 0.0);

            // Sombra
            ui.painter().galley(
                pos + egui::vec2(2.0, 2.0),
                galley_shadow,
                egui::Color32::TRANSPARENT,
            );

            // Texto principal
            ui.painter().galley(
                pos,
                galley.clone(),
                egui::Color32::TRANSPARENT,
            );

            ui.add_space(galley.rect.height() + 10.0);
        }
    }

    /// Dibuja la barra de estado con dispositivo y controles.
    /// Retorna (click_dispositivo, click_calibrar, click_salir).
    fn draw_status(&mut self, ui: &mut egui::Ui) -> (bool, bool, bool) {
        let mut click_device = false;
        let mut click_calibrate = false;
        let mut click_exit = false;

        let is_recalibrating = self.recalibration_state.is_some();
        let needs_calibration = !self.silence_calibrated || !self.voice_calibrated;

        ui.horizontal(|ui| {
            self.draw_voice_indicator(ui);
            ui.add_space(5.0);

            if self.draw_device_button(ui) {
                click_device = true;
            }
            ui.add_space(10.0);

            if self.draw_calibration_button(ui, is_recalibrating, needs_calibration) {
                click_calibrate = true;
            }
            ui.add_space(10.0);

            self.draw_status_center(ui, is_recalibrating, needs_calibration);

            if Self::draw_exit_button(ui) {
                click_exit = true;
            }
        });

        (click_device, click_calibrate, click_exit)
    }

    /// Dibuja el indicador de voz (punto de estado).
    fn draw_voice_indicator(&self, ui: &mut egui::Ui) {
        let color = if self.is_speaking {
            egui::Color32::from_rgb(255, 165, 0)
        } else {
            egui::Color32::from_rgb(100, 100, 100)
        };
        let (rect, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
    }

    /// Dibuja el botón del dispositivo. Retorna true si fue clickeado.
    fn draw_device_button(&self, ui: &mut egui::Ui) -> bool {
        let (text, color) = if self.device_selected {
            (
                format!("🔊 ({})", self.get_current_device_display_name()),
                egui::Color32::from_rgb(200, 200, 200),
            )
        } else {
            (
                "🔊 (Pendiente seleccionar)".to_string(),
                egui::Color32::from_rgb(255, 200, 100),
            )
        };

        let button = ui.add(
            egui::Label::new(egui::RichText::new(&text).size(11.0).color(color))
                .sense(egui::Sense::click())
        );

        if button.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        button.clicked()
    }

    /// Dibuja el botón de calibración. Retorna true si fue clickeado.
    fn draw_calibration_button(&self, ui: &mut egui::Ui, is_recalibrating: bool, needs_calibration: bool) -> bool {
        let color = Self::get_gear_color(ui, is_recalibrating, needs_calibration);

        let button = ui.add(
            egui::Label::new(egui::RichText::new("⚙").size(14.0).color(color))
                .sense(egui::Sense::click())
        );

        if button.hovered() && !is_recalibrating {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        button.clicked() && !is_recalibrating
    }

    /// Obtiene el color del engranaje según el estado.
    fn get_gear_color(ui: &egui::Ui, is_recalibrating: bool, needs_calibration: bool) -> egui::Color32 {
        if is_recalibrating {
            egui::Color32::from_rgb(80, 80, 80)
        } else if needs_calibration {
            let blink = (ui.ctx().input(|i| i.time) * 2.0) as i32 % 2 == 0;
            if blink {
                egui::Color32::from_rgb(255, 80, 80)
            } else {
                egui::Color32::from_rgb(150, 150, 150)
            }
        } else {
            egui::Color32::from_rgb(180, 180, 180)
        }
    }

    /// Dibuja el área central de estado (progreso de calibración o texto).
    fn draw_status_center(&self, ui: &mut egui::Ui, is_recalibrating: bool, needs_calibration: bool) {
        if is_recalibrating {
            self.draw_recalibration_progress(ui);
        } else if needs_calibration {
            self.draw_calibration_needed_text(ui);
        }
    }

    /// Dibuja el progreso de recalibración con barras.
    fn draw_recalibration_progress(&self, ui: &mut egui::Ui) {
        let state = match &self.recalibration_state {
            Some(s) => s,
            None => return,
        };

        let elapsed = state.start_time.elapsed().as_secs_f32();
        let progress = (elapsed / 5.0).min(1.0);
        let remaining = (5.0 - elapsed).max(0.0);

        let (text, color) = match state.calibration_type {
            RecalibrationType::Silence => (
                format!("Calibrando Silencio: Silencio por favor {:.0}s", remaining),
                egui::Color32::from_rgb(100, 150, 255),
            ),
            RecalibrationType::Voice => (
                format!("Calibrando Voz: Hable por favor {:.0}s", remaining),
                egui::Color32::from_rgb(255, 180, 100),
            ),
        };

        ui.label(egui::RichText::new(&text).size(10.0).color(egui::Color32::WHITE));
        ui.add_space(5.0);

        Self::draw_progress_bar(ui, 50.0, 6.0, progress, color);
        ui.add_space(5.0);

        self.draw_mic_level_bar(ui);
    }

    /// Dibuja una barra de progreso.
    fn draw_progress_bar(ui: &mut egui::Ui, width: f32, height: f32, progress: f32, color: egui::Color32) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
        ui.painter().rect_filled(rect, 3.0, egui::Color32::from_rgb(60, 60, 60));

        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(width * progress, height));
        ui.painter().rect_filled(fill_rect, 3.0, color);
    }

    /// Dibuja la barra de nivel del micrófono.
    fn draw_mic_level_bar(&self, ui: &mut egui::Ui) {
        let width = 60.0;
        let height = 6.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

        ui.painter().rect_filled(rect, 3.0, egui::Color32::from_rgb(40, 40, 40));

        let level = (self.current_mic_level / 0.05).min(1.0);
        let color = Self::get_level_color(level);

        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(width * level, height));
        ui.painter().rect_filled(fill_rect, 3.0, color);
    }

    /// Obtiene el color según el nivel de audio.
    fn get_level_color(level: f32) -> egui::Color32 {
        if level > 0.7 {
            egui::Color32::from_rgb(255, 100, 100)
        } else if level > 0.3 {
            egui::Color32::from_rgb(100, 255, 100)
        } else {
            egui::Color32::from_rgb(100, 100, 255)
        }
    }

    /// Dibuja el texto de calibración necesaria.
    fn draw_calibration_needed_text(&self, ui: &mut egui::Ui) {
        let text = match (self.silence_calibrated, self.voice_calibrated) {
            (false, false) => "Necesario Calibrar Silencio y Voz",
            (true, false) => "Necesario Calibrar Voz",
            (false, true) => "Necesario Calibrar Silencio",
            _ => return,
        };

        ui.label(egui::RichText::new(text).size(10.0).color(egui::Color32::from_rgb(255, 150, 150)));
    }

    /// Dibuja el botón de salir. Retorna true si fue clickeado.
    fn draw_exit_button(ui: &mut egui::Ui) -> bool {
        let mut clicked = false;

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let button = ui.add(
                egui::Label::new(
                    egui::RichText::new("🚪").size(14.0).color(egui::Color32::from_rgb(180, 180, 180))
                ).sense(egui::Sense::click())
            );

            if button.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if button.clicked() {
                clicked = true;
            }
        });

        clicked
    }

    /// Obtiene el nombre corto del dispositivo actualmente seleccionado.
    fn get_current_device_display_name(&self) -> String {
        // Buscar el índice del dispositivo actual y usar la función común
        if let Some(index) = self.available_devices.iter().position(|d| d.name == self.device_name) {
            self.get_device_display_name(index)
        } else if self.device_name.is_empty() {
            "Sin seleccionar".to_string()
        } else {
            // Fallback: usar el nombre interno acortado
            Self::truncate_name(&self.device_name, 25)
        }
    }

    /// Trunca un nombre a la longitud especificada.
    fn truncate_name(name: &str, max_len: usize) -> String {
        if name.len() > max_len {
            format!("{}...", &name[..max_len - 3])
        } else {
            name.to_string()
        }
    }

    /// Crea el frame estándar para diálogos.
    fn dialog_frame() -> egui::Frame {
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(50, 50, 50))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(0.0, 6.0))
    }

    /// Dibuja un título de diálogo con estilo estándar.
    fn draw_dialog_title(ui: &mut egui::Ui, title: &str) {
        ui.label(
            egui::RichText::new(title)
                .size(11.0)
                .color(egui::Color32::WHITE)
        );
        ui.add_space(6.0);
    }

    /// Dibuja el diálogo de confirmación de salida.
    fn draw_exit_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("exit_confirm")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([160.0, 55.0])
            .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -25.0])
            .frame(Self::dialog_frame())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    Self::draw_dialog_title(ui, "¿Deseas salir?");

                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        if ui.add_sized([65.0, 20.0], egui::Button::new("Sí")).clicked() {
                            info!("Usuario confirmó salida");
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.add_space(10.0);
                        if ui.add_sized([65.0, 20.0], egui::Button::new("No")).clicked() {
                            self.show_exit_dialog = false;
                        }
                    });
                });
            });
    }

    /// Dibuja el diálogo de recalibración.
    fn draw_calibration_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("calibration_confirm")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 55.0])
            .anchor(egui::Align2::LEFT_BOTTOM, [60.0, -25.0])
            .frame(Self::dialog_frame())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    Self::draw_dialog_title(ui, "¿Deseas recalibrar?");

                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        if ui.add_sized([86.0, 20.0], egui::Button::new("Silencio")).clicked() {
                            info!("Iniciando recalibración de silencio");
                            self.show_calibration_dialog = false;
                            self.start_recalibration(RecalibrationType::Silence);
                        }
                        ui.add_space(10.0);
                        if ui.add_sized([86.0, 20.0], egui::Button::new("Voz")).clicked() {
                            info!("Iniciando recalibración de voz");
                            self.show_calibration_dialog = false;
                            self.start_recalibration(RecalibrationType::Voice);
                        }
                        ui.add_space(10.0);
                        if ui.add_sized([86.0, 20.0], egui::Button::new("Salir")).clicked() {
                            self.show_calibration_dialog = false;
                        }
                    });
                });
            });
    }

    /// Dibuja el diálogo de selección de dispositivo.
    fn draw_device_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("device_selector")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size([300.0, 55.0])
            .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -25.0])
            .frame(Self::dialog_frame())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    Self::draw_dialog_title(ui, "Seleccione el dispositivo");
                    self.draw_device_navigation(ui);
                });
            });
    }

    /// Dibuja la navegación de dispositivos (flechas + nombre + refrescar).
    fn draw_device_navigation(&mut self, ui: &mut egui::Ui) {
        let has_devices = !self.available_devices.is_empty();
        let device_count = self.available_devices.len();

        ui.horizontal(|ui| {
            ui.add_space(5.0);

            if Self::draw_nav_arrow(ui, "◀", has_devices) {
                self.navigate_device_previous(device_count);
            }
            ui.add_space(3.0);

            if self.draw_device_name_button(ui, has_devices) {
                self.select_device(self.device_nav_index);
                self.show_device_selector = false;
            }
            ui.add_space(3.0);

            if Self::draw_nav_arrow(ui, "▶", has_devices) {
                self.navigate_device_next(device_count);
            }
            ui.add_space(3.0);

            if Self::draw_refresh_button(ui) {
                self.refresh_device_list();
            }
        });
    }

    /// Dibuja una flecha de navegación. Retorna true si fue clickeada.
    fn draw_nav_arrow(ui: &mut egui::Ui, symbol: &str, enabled: bool) -> bool {
        let color = if enabled {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgb(80, 80, 80)
        };

        let btn = ui.add(
            egui::Label::new(egui::RichText::new(symbol).size(16.0).color(color))
                .sense(egui::Sense::click())
        );

        btn.clicked() && enabled
    }

    /// Dibuja el botón con el nombre del dispositivo. Retorna true si fue clickeado.
    fn draw_device_name_button(&self, ui: &mut egui::Ui, has_devices: bool) -> bool {
        let name = if has_devices {
            self.get_device_display_name(self.device_nav_index)
        } else {
            "Sin dispositivos".to_string()
        };

        let btn = ui.add_sized(
            [180.0, 20.0],
            egui::Button::new(egui::RichText::new(&name).size(10.0))
        );

        btn.clicked() && has_devices
    }

    /// Dibuja el botón de refrescar. Retorna true si fue clickeado.
    fn draw_refresh_button(ui: &mut egui::Ui) -> bool {
        ui.add_sized([24.0, 20.0], egui::Button::new(egui::RichText::new("🔄").size(12.0)))
            .clicked()
    }

    /// Navega al dispositivo anterior (cíclico).
    fn navigate_device_previous(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.device_nav_index = if self.device_nav_index == 0 {
            count - 1
        } else {
            self.device_nav_index - 1
        };
    }

    /// Navega al dispositivo siguiente (cíclico).
    fn navigate_device_next(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        self.device_nav_index = if self.device_nav_index >= count - 1 {
            0
        } else {
            self.device_nav_index + 1
        };
    }

    /// Obtiene el nombre amigable de un dispositivo por índice.
    fn get_device_display_name(&self, index: usize) -> String {
        if let Some(device) = self.available_devices.get(index) {
            Self::truncate_name(&device.display_name, 25)
        } else {
            "Desconocido".to_string()
        }
    }

    /// Selecciona un dispositivo por índice.
    fn select_device(&mut self, index: usize) {
        if let Some(device) = self.available_devices.get(index) {
            info!("Seleccionando dispositivo: {}", device.name);

            // Detener captura actual
            if let Some(ref mut audio_capture) = self.audio_capture {
                audio_capture.stop_capture();

                // Seleccionar nuevo dispositivo
                if let Err(e) = audio_capture.select_device(&device.name) {
                    error!("Error al seleccionar dispositivo: {}", e);
                    return;
                }

                // Reiniciar captura
                if let Err(e) = audio_capture.start_capture() {
                    error!("Error al reiniciar captura: {}", e);
                    return;
                }
            }

            self.device_name = device.name.clone();
            self.device_selected = true;
            self.is_capturing = true;

            // Resetear calibración al cambiar dispositivo
            self.silence_calibrated = false;
            self.voice_calibrated = false;

            info!("Dispositivo cambiado exitosamente, requiere nueva calibración");
        }
    }

    /// Refresca la lista de dispositivos disponibles.
    /// Intenta posicionar el índice en el dispositivo actualmente seleccionado.
    fn refresh_device_list(&mut self) {
        info!("Refrescando lista de dispositivos...");
        if let Some(ref audio_capture) = self.audio_capture {
            if let Ok(devices) = audio_capture.list_input_devices() {
                let old_count = self.available_devices.len();
                self.available_devices = devices;
                let new_count = self.available_devices.len();
                info!("Dispositivos actualizados: {} -> {}", old_count, new_count);

                // Buscar el dispositivo actual en la nueva lista
                if !self.device_name.is_empty() {
                    self.device_nav_index = self.available_devices
                        .iter()
                        .position(|d| d.name == self.device_name)
                        .unwrap_or(0);
                } else if self.device_nav_index >= new_count {
                    // Ajustar índice si excede el nuevo tamaño
                    self.device_nav_index = new_count.saturating_sub(1);
                }
            }
        }
    }

    /// Inicia el proceso de recalibración.
    fn start_recalibration(&mut self, calibration_type: RecalibrationType) {
        self.recalibration_state = Some(RecalibrationState {
            calibration_type,
            start_time: Instant::now(),
            samples: Vec::new(),
        });
    }

    /// Procesa la recalibración en curso (recolecta samples).
    fn process_recalibration(&mut self) {
        let should_finish = if let Some(ref mut state) = self.recalibration_state {
            // Recolectar samples del audio_capture
            if let Some(ref audio_capture) = self.audio_capture {
                let samples = audio_capture.take_samples();
                if !samples.is_empty() {
                    // Calcular nivel actual para mostrar
                    let power = AudioPower::from_samples(&samples);
                    self.current_mic_level = power.value();
                    state.samples.extend(samples);
                }
            }

            // Verificar si pasaron 5 segundos
            state.start_time.elapsed() >= StdDuration::from_secs(5)
        } else {
            false
        };

        if should_finish {
            self.finish_recalibration();
        }
    }

    /// Finaliza la recalibración y aplica los nuevos valores.
    fn finish_recalibration(&mut self) {
        if let Some(state) = self.recalibration_state.take() {
            if state.samples.is_empty() {
                warn!("Recalibración sin samples, ignorando");
                return;
            }

            // Calcular potencia promedio de los samples
            let power = AudioPower::from_samples(&state.samples);

            // Obtener calibración actual
            let mut calibration = self.silence_detector.calibration().clone();

            match state.calibration_type {
                RecalibrationType::Silence => {
                    info!(
                        "Recalibración de silencio completada: {} samples, potencia: {:.6}",
                        state.samples.len(),
                        power.value()
                    );
                    calibration = calibration.with_noise_floor(power);
                    self.silence_calibrated = true;
                }
                RecalibrationType::Voice => {
                    info!(
                        "Recalibración de voz completada: {} samples, potencia: {:.6}",
                        state.samples.len(),
                        power.value()
                    );
                    calibration = calibration.with_voice_power(power);
                    self.voice_calibrated = true;
                }
            }

            // Aplicar nueva calibración
            info!("Nueva calibración: {}", calibration);
            self.silence_detector = SilenceDetector::new(calibration.clone());
            self.orchestrator.set_calibration(calibration);

            // Resetear nivel del micrófono
            self.current_mic_level = 0.0;
        }
    }
}

impl eframe::App for SubtitleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.position_window(ctx);

        // Si hay recalibración en curso, procesar eso en lugar del audio normal
        if self.recalibration_state.is_some() {
            self.process_recalibration();
        } else {
            self.process_audio();
        }

        self.process_messages();
        self.activate_pending();
        self.remove_expired();

        self.update_window_size(ctx);

        let mut click_device = false;
        let mut click_calibrate = false;
        let mut click_exit = false;

        let frame = egui::Frame::none()
            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, BACKGROUND_ALPHA))
            .rounding(8.0)
            .inner_margin(10.0);

        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    self.draw_subtitles(ui);

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        let (cd, cc, ce) = self.draw_status(ui);
                        click_device = cd;
                        click_calibrate = cc;
                        click_exit = ce;
                    });
                });
            });

        // Manejar clicks en los controles
        if click_device {
            info!("Click en dispositivo - abriendo selector...");
            self.refresh_device_list();
            self.show_device_selector = true;
        }

        if click_calibrate {
            info!("Click en calibración - iniciando recalibración...");
            self.show_calibration_dialog = true;
        }

        if click_exit {
            self.show_exit_dialog = true;
        }

        // Diálogo compacto de confirmación para salir
        if self.show_exit_dialog {
            self.draw_exit_dialog(ctx);
        }

        // Diálogo de recalibración
        if self.show_calibration_dialog {
            self.draw_calibration_dialog(ctx);
        }

        // Diálogo de selección de dispositivo
        if self.show_device_selector {
            self.draw_device_dialog(ctx);
        }

        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(ref mut capture) = self.audio_capture {
            capture.stop_capture();
        }

        self.orchestrator.stop();
        info!("Aplicación cerrada");
    }
}
