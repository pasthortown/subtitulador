//! Adaptadores de salida.

mod http_transcription;
mod http_translation;

pub use http_transcription::HttpTranscriptionClient;
pub use http_translation::{HttpTranslationClient, AvailableLanguage};
