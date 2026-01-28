//! Adaptadores de infraestructura.

pub mod inbound;
pub mod outbound;

pub use inbound::{AudioCapture, AudioDeviceInfo};
pub use outbound::{HttpTranscriptionClient, HttpTranslationClient, AvailableLanguage};
