//! Configuración de la aplicación.

use std::env;
use anyhow::Result;
use serde::Deserialize;

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
                .unwrap_or_else(|_| "http://192.168.97.10:8000".to_string()),
            translation_url: env::var("TRANSLATION_URL")
                .unwrap_or_else(|_| "http://192.168.97.11:5000".to_string()),
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend_url: "http://192.168.97.10:8000".to_string(),
            translation_url: "http://192.168.97.11:5000".to_string(),
            input_language: "es".to_string(),
            output_language: "pt".to_string(),
            sample_rate: 16000,
            silence_duration_ms: 750,
        }
    }
}
