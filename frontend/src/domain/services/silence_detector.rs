//! Service: SilenceDetector - Detección de silencio basada en calibración.

use std::time::{Duration, Instant};
use crate::domain::{AudioPower, CalibrationData};

/// Duración de silencio para cortar buffer (750ms).
/// Igual que la implementación Python.
const SILENCE_DURATION_TO_CUT_MS: u64 = 750;

/// Servicio de detección de silencio.
#[derive(Debug)]
pub struct SilenceDetector {
    calibration: CalibrationData,
    silence_start: Option<Instant>,
}

impl SilenceDetector {
    /// Crea un nuevo detector con calibración.
    pub fn new(calibration: CalibrationData) -> Self {
        Self {
            calibration,
            silence_start: None,
        }
    }

    /// Obtiene la calibración actual.
    pub fn calibration(&self) -> &CalibrationData {
        &self.calibration
    }

    /// Actualiza con la calibración.
    pub fn set_calibration(&mut self, calibration: CalibrationData) {
        self.calibration = calibration;
        self.reset();
    }

    /// Procesa una potencia de audio y retorna si debe cortar el buffer.
    pub fn process(&mut self, power: AudioPower) -> bool {
        if self.calibration.is_silence(power) {
            // Iniciar contador si es la primera detección de silencio
            if self.silence_start.is_none() {
                self.silence_start = Some(Instant::now());
            }

            // Verificar si se ha sostenido por el tiempo requerido
            if let Some(start) = self.silence_start {
                let duration = start.elapsed();
                return duration >= Duration::from_millis(SILENCE_DURATION_TO_CUT_MS);
            }
        } else {
            // Hay sonido, resetear contador
            self.silence_start = None;
        }

        false
    }

    /// Resetea el detector.
    pub fn reset(&mut self) {
        self.silence_start = None;
    }

    /// Verifica si actualmente hay silencio.
    pub fn is_silent(&self) -> bool {
        self.silence_start.is_some()
    }

    /// Duración del silencio actual.
    pub fn silence_duration(&self) -> Option<Duration> {
        self.silence_start.map(|start| start.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detection() {
        let cal = CalibrationData::new(
            AudioPower::new(0.01),
            AudioPower::new(0.1),
        );
        let mut detector = SilenceDetector::new(cal);

        // Voz - no cortar
        assert!(!detector.process(AudioPower::new(0.08)));
        assert!(!detector.is_silent());

        // Silencio - no cortar inmediatamente
        assert!(!detector.process(AudioPower::new(0.02)));
        assert!(detector.is_silent());
    }
}
