//! Adaptador HTTP para el servicio de transcripción.

use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use tracing::{debug, error};

use crate::domain::{AudioBuffer, Language};
use crate::domain::ports::outbound::TranscriptionResult;
use crate::application::dtos::{TranscriptionRequest, TranscriptionResponse};

/// Cliente HTTP para el servicio de transcripción.
pub struct HttpTranscriptionClient {
    client: Client,
    base_url: String,
}

impl HttpTranscriptionClient {
    /// Crea un nuevo cliente.
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Transcribe un buffer de audio.
    pub fn transcribe(&self, buffer: &AudioBuffer) -> Result<TranscriptionResult> {
        let url = format!("{}/api/v1/transcribe", self.base_url);

        let request = TranscriptionRequest {
            audio: buffer.to_base64(),
            language: buffer.language().code().to_string(),
            sample_rate: 16000,
        };

        debug!("Enviando request de transcripción a {}", url);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| anyhow!("Error de conexión: {}", e))?;

        let status = response.status();
        let body: TranscriptionResponse = response
            .json()
            .map_err(|e| anyhow!("Error parseando respuesta: {}", e))?;

        if !body.success {
            let error_msg = body.error
                .map(|e| format!("{}: {}", e.code, e.message))
                .unwrap_or_else(|| "Error desconocido".to_string());
            error!("Error de transcripción: {}", error_msg);
            return Err(anyhow!("Error de transcripción: {}", error_msg));
        }

        let data = body.data.ok_or_else(|| anyhow!("Respuesta sin datos"))?;

        Ok(TranscriptionResult {
            text: data.text,
            language: Language::new(&data.language).unwrap_or_default(),
            confidence: data.confidence,
            duration_ms: data.duration_ms,
            processing_time_ms: data.processing_time_ms,
        })
    }

    /// Verifica si el servicio está disponible.
    pub fn is_available(&self) -> bool {
        let url = format!("{}/api/v1/health", self.base_url);
        self.client.get(&url).send().is_ok()
    }
}
