//! Adaptadores de infraestructura.

pub mod inbound;
pub mod outbound;

pub use inbound::AudioCapture;
pub use outbound::{HttpTranscriptionClient, HttpTranslationClient};
