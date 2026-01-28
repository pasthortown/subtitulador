//! Configuración de la aplicación.

use std::env;
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Configuración de la aplicación.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub backend_url: String,
    pub translation_url: String,
    pub input_language: String,
    pub output_language: String,
    pub sample_rate: u32,
    pub silence_duration_ms: u64,
}

impl AppConfig {
    /// Carga la configuración desde variables de entorno.
    pub fn load() -> Result<Self> {
        Ok(Self {
            backend_url: env::var("BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:8000".to_string()),
            translation_url: env::var("TRANSLATION_URL")
                .unwrap_or_else(|_| "http://localhost:5001".to_string()),
            input_language: env::var("INPUT_LANGUAGE")
                .unwrap_or_else(|_| "es".to_string()),
            output_language: env::var("OUTPUT_LANGUAGE")
                .unwrap_or_else(|_| "pt".to_string()),
            sample_rate: env::var("SAMPLE_RATE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(16000),
            silence_duration_ms: env::var("SILENCE_DURATION_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(750),
        })
    }
}

/// Configuración persistida en config.json (solo idiomas).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedConfig {
    input_language: String,
    output_language: String,
}

impl AppConfig {
    /// Ruta del archivo config.json junto al ejecutable.
    fn config_path() -> Option<PathBuf> {
        env::current_exe().ok().and_then(|p| p.parent().map(|dir| dir.join("config.json")))
    }

    /// Carga idiomas desde config.json si existe y los aplica.
    pub fn load_persisted_languages(&mut self) {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return,
        };

        if !path.exists() {
            return;
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<PersistedConfig>(&content) {
                    Ok(persisted) => {
                        info!(
                            "Idiomas cargados desde config.json: {} -> {}",
                            persisted.input_language, persisted.output_language
                        );
                        self.input_language = persisted.input_language;
                        self.output_language = persisted.output_language;
                    }
                    Err(e) => warn!("Error parseando config.json: {}", e),
                }
            }
            Err(e) => warn!("Error leyendo config.json: {}", e),
        }
    }

    /// Guarda los idiomas actuales en config.json.
    pub fn save_languages(&self) {
        let path = match Self::config_path() {
            Some(p) => p,
            None => {
                warn!("No se pudo determinar la ruta del ejecutable para guardar config.json");
                return;
            }
        };

        let persisted = PersistedConfig {
            input_language: self.input_language.clone(),
            output_language: self.output_language.clone(),
        };

        match serde_json::to_string_pretty(&persisted) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    warn!("Error escribiendo config.json: {}", e);
                } else {
                    info!("Idiomas guardados en config.json: {} -> {}", self.input_language, self.output_language);
                }
            }
            Err(e) => warn!("Error serializando config.json: {}", e),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend_url: "http://localhost:8000".to_string(),
            translation_url: "http://localhost:5001".to_string(),
            input_language: "es".to_string(),
            output_language: "pt".to_string(),
            sample_rate: 16000,
            silence_duration_ms: 750,
        }
    }
}
