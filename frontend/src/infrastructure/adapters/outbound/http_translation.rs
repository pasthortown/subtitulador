//! Adaptador HTTP para el servicio de traducción.

use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use tracing::{debug, error};

use serde::Deserialize;

use crate::domain::Language;
use crate::domain::ports::outbound::{TranslationPort, AvailableLanguage};
use crate::application::dtos::{TranslationRequest, TranslationResponse};

/// DTO interno para deserializar idiomas desde la API.
#[derive(Debug, Deserialize)]
struct AvailableLanguageDto {
    pub code: String,
    pub name: String,
}

/// Cliente HTTP para el servicio de traducción (LibreTranslate).
pub struct HttpTranslationClient {
    client: Client,
    base_url: String,
}

impl HttpTranslationClient {
    /// Crea un nuevo cliente.
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

impl TranslationPort for HttpTranslationClient {
    fn translate(&self, text: &str, source: &Language, target: &Language) -> Result<String> {
        if text.is_empty() {
            return Ok(String::new());
        }

        let url = format!("{}/translate", self.base_url);

        let request = TranslationRequest {
            q: text.to_string(),
            source: source.code().to_string(),
            target: target.code().to_string(),
        };

        debug!("Enviando request de traducción: {} -> {}", source, target);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .map_err(|e| anyhow!("Error de conexión: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            error!("Error de traducción: status {}", status);
            return Err(anyhow!("Error de traducción: status {}", status));
        }

        let body: TranslationResponse = response
            .json()
            .map_err(|e| anyhow!("Error parseando respuesta: {}", e))?;

        Ok(body.translated_text)
    }

    fn fetch_languages(&self) -> Result<Vec<AvailableLanguage>> {
        let url = format!("{}/languages", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .map_err(|e| anyhow!("Error de conexión al obtener idiomas: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Error al obtener idiomas: status {}", response.status()));
        }

        let dtos: Vec<AvailableLanguageDto> = response
            .json()
            .map_err(|e| anyhow!("Error parseando idiomas: {}", e))?;

        debug!("Idiomas disponibles: {}", dtos.len());

        Ok(dtos.into_iter().map(|d| AvailableLanguage {
            code: d.code,
            name: d.name,
        }).collect())
    }

    fn is_available(&self) -> bool {
        let url = format!("{}/languages", self.base_url);
        self.client.get(&url).send().is_ok()
    }
}
