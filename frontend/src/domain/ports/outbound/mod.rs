//! Puertos de salida (driven).

use crate::domain::{AudioBuffer, Language};
use anyhow::Result;

/// Resultado de transcripción.
#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: Language,
    pub confidence: f32,
    pub duration_ms: u64,
    pub processing_time_ms: u64,
}

/// Idioma disponible en un servicio de traducción.
#[derive(Debug, Clone)]
pub struct AvailableLanguage {
    pub code: String,
    pub name: String,
}

/// Puerto para servicio de transcripción.
pub trait TranscriptionPort: Send + Sync {
    /// Transcribe un buffer de audio.
    fn transcribe(&self, buffer: &AudioBuffer) -> Result<TranscriptionResult>;

    /// Verifica si el servicio está disponible.
    fn is_available(&self) -> bool;
}

/// Puerto para servicio de traducción.
pub trait TranslationPort: Send + Sync {
    /// Traduce texto de un idioma a otro.
    fn translate(&self, text: &str, source: &Language, target: &Language) -> Result<String>;

    /// Obtiene la lista de idiomas disponibles.
    fn fetch_languages(&self) -> Result<Vec<AvailableLanguage>>;

    /// Verifica si el servicio está disponible.
    fn is_available(&self) -> bool;
}
