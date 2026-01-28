//! Subtitulador Frontend - Aplicación de subtítulos en tiempo real.
//!
//! Captura audio del micrófono, lo envía al backend para transcripción,
//! y muestra subtítulos en una ventana transparente.

mod domain;
mod application;
mod infrastructure;
mod presentation;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use crate::infrastructure::config::AppConfig;
use crate::application::services::TranscriptionOrchestrator;
use crate::presentation::ui::SubtitleApp;

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .init();
}

/// Ejecuta la aplicación principal de subtítulos.
fn run_subtitle_app(config: AppConfig) -> Result<()> {
    info!("Iniciando aplicación de subtítulos...");

    // Crear orquestador con calibración por defecto
    let orchestrator = TranscriptionOrchestrator::new(config.clone())?;

    // Configurar ventana de subtítulos
    // Transparente, sin decoraciones, siempre visible
    // La posición se ajusta dinámicamente en SubtitleApp
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 100.0])
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Subtitulador",
        options,
        Box::new(move |cc| {
            // Configurar estilo transparente
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.window_fill = egui::Color32::TRANSPARENT;
            style.visuals.panel_fill = egui::Color32::TRANSPARENT;
            cc.egui_ctx.set_style(style);

            Box::new(SubtitleApp::new(orchestrator, config))
        }),
    ).map_err(|e| anyhow::anyhow!("Error al ejecutar aplicación: {}", e))
}

fn main() -> Result<()> {
    // Cargar variables de entorno
    dotenv::dotenv().ok();

    // Configurar logging
    setup_logging();

    info!("Iniciando Subtitulador Frontend...");

    // Cargar configuración
    let mut config = AppConfig::load()?;
    config.load_persisted_languages();
    info!("Configuración cargada: {:?}", config);

    // Ejecutar aplicación de subtítulos directamente
    run_subtitle_app(config)
}
