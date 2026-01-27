//! DTOs de la aplicación.

use serde::{Deserialize, Serialize};

/// Request para transcripción.
#[derive(Debug, Serialize)]
pub struct TranscriptionRequest {
    pub audio: String,
    pub language: String,
    pub sample_rate: u32,
}

/// Response de transcripción.
#[derive(Debug, Deserialize)]
pub struct TranscriptionResponse {
    pub success: bool,
    pub data: Option<TranscriptionData>,
    pub error: Option<ErrorData>,
}

#[derive(Debug, Deserialize)]
pub struct TranscriptionData {
    pub text: String,
    pub language: String,
    pub confidence: f32,
    pub duration_ms: u64,
    pub processing_time_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct ErrorData {
    pub code: String,
    pub message: String,
}

/// Request para traducción.
#[derive(Debug, Serialize)]
pub struct TranslationRequest {
    pub q: String,
    pub source: String,
    pub target: String,
}

/// Response de traducción.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResponse {
    pub translated_text: String,
}
