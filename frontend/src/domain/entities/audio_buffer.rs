//! Entity: AudioBuffer - Buffer de audio para transcripción.

use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::domain::{Language, AudioPower, Duration};

/// Formato de audio para Whisper.
pub const SAMPLE_RATE: u32 = 16000;
pub const CHANNELS: u16 = 1;

/// Entidad que representa un buffer de audio para procesar.
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    id: Uuid,
    samples: Vec<f32>,
    language: Language,
    created_at: std::time::Instant,
}

impl AudioBuffer {
    /// Crea un nuevo buffer de audio.
    pub fn new(samples: Vec<f32>, language: Language) -> Self {
        Self {
            id: Uuid::new_v4(),
            samples,
            language,
            created_at: std::time::Instant::now(),
        }
    }

    /// Crea un buffer vacío.
    pub fn empty(language: Language) -> Self {
        Self::new(Vec::new(), language)
    }

    /// ID único del buffer.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Samples de audio.
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// Idioma del audio.
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Número de samples.
    pub fn num_samples(&self) -> usize {
        self.samples.len()
    }

    /// Verifica si está vacío.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Duración del buffer.
    pub fn duration(&self) -> Duration {
        Duration::from_samples(self.samples.len(), SAMPLE_RATE)
    }

    /// Potencia RMS del buffer.
    pub fn power(&self) -> AudioPower {
        AudioPower::from_samples(&self.samples)
    }

    /// Añade samples al buffer.
    pub fn append(&mut self, samples: &[f32]) {
        self.samples.extend_from_slice(samples);
    }

    /// Limpia el buffer.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Extrae y limpia el buffer, retornando los samples.
    pub fn take(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    /// Convierte a base64 para enviar al backend.
    pub fn to_base64(&self) -> String {
        let bytes: Vec<u8> = self.samples
            .iter()
            .flat_map(|&s| s.to_le_bytes())
            .collect();

        BASE64.encode(&bytes)
    }

    /// Verifica si es válido para transcripción (mínimo 300ms).
    pub fn is_valid_for_transcription(&self) -> bool {
        self.duration().as_millis() >= 300
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_buffer() {
        let samples = vec![0.1_f32; 16000]; // 1 segundo
        let buffer = AudioBuffer::new(samples, Language::spanish());

        assert_eq!(buffer.num_samples(), 16000);
        assert_eq!(buffer.duration().as_millis(), 1000);
        assert!(buffer.is_valid_for_transcription());
    }

    #[test]
    fn test_empty_buffer() {
        let buffer = AudioBuffer::empty(Language::spanish());
        assert!(buffer.is_empty());
        assert!(!buffer.is_valid_for_transcription());
    }

    #[test]
    fn test_append_samples() {
        let mut buffer = AudioBuffer::empty(Language::spanish());
        buffer.append(&[0.1, 0.2, 0.3]);

        assert_eq!(buffer.num_samples(), 3);
    }
}
