//! Entity: CalibrationData - Datos de calibración del micrófono.

use crate::domain::AudioPower;

/// Factor para calcular el umbral de silencio.
/// umbral = ruido + THRESHOLD_FACTOR * (voz - ruido)
const THRESHOLD_FACTOR: f32 = 0.3;

/// Entidad que almacena los datos de calibración del micrófono.
#[derive(Debug, Clone)]
pub struct CalibrationData {
    noise_floor: AudioPower,
    voice_power: AudioPower,
    silence_threshold: AudioPower,
}

impl CalibrationData {
    /// Crea datos de calibración desde mediciones.
    pub fn new(noise_floor: AudioPower, voice_power: AudioPower) -> Self {
        // Calcular umbral: ruido + 30% de la diferencia entre voz y ruido
        let threshold_value = noise_floor.value()
            + THRESHOLD_FACTOR * (voice_power.value() - noise_floor.value());

        Self {
            noise_floor,
            voice_power,
            silence_threshold: AudioPower::new(threshold_value),
        }
    }

    /// Crea calibración con valores por defecto.
    pub fn default_values() -> Self {
        Self::new(
            AudioPower::new(0.001),
            AudioPower::new(0.05),
        )
    }

    /// Potencia del ruido de fondo.
    pub fn noise_floor(&self) -> AudioPower {
        self.noise_floor
    }

    /// Potencia de la voz.
    pub fn voice_power(&self) -> AudioPower {
        self.voice_power
    }

    /// Umbral de silencio calculado.
    pub fn silence_threshold(&self) -> AudioPower {
        self.silence_threshold
    }

    /// Determina si una potencia corresponde a silencio.
    pub fn is_silence(&self, power: AudioPower) -> bool {
        power.is_below(self.silence_threshold)
    }

    /// Determina si una potencia corresponde a voz.
    pub fn is_voice(&self, power: AudioPower) -> bool {
        !self.is_silence(power)
    }

    /// Crea una nueva calibración con el noise_floor actualizado.
    pub fn with_noise_floor(&self, noise_floor: AudioPower) -> Self {
        Self::new(noise_floor, self.voice_power)
    }

    /// Crea una nueva calibración con el voice_power actualizado.
    pub fn with_voice_power(&self, voice_power: AudioPower) -> Self {
        Self::new(self.noise_floor, voice_power)
    }
}

impl std::fmt::Display for CalibrationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Calibración: ruido={}, voz={}, umbral={}",
            self.noise_floor, self.voice_power, self.silence_threshold
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_threshold() {
        let noise = AudioPower::new(0.01);
        let voice = AudioPower::new(0.1);
        let cal = CalibrationData::new(noise, voice);

        // threshold = 0.01 + 0.3 * (0.1 - 0.01) = 0.01 + 0.027 = 0.037
        let expected = 0.01 + 0.3 * (0.1 - 0.01);
        assert!((cal.silence_threshold().value() - expected).abs() < 0.001);
    }

    #[test]
    fn test_is_silence() {
        let cal = CalibrationData::new(
            AudioPower::new(0.01),
            AudioPower::new(0.1),
        );

        // Bajo el umbral = silencio
        assert!(cal.is_silence(AudioPower::new(0.02)));

        // Sobre el umbral = voz
        assert!(cal.is_voice(AudioPower::new(0.05)));
    }

    #[test]
    fn test_default_values() {
        let cal = CalibrationData::default_values();
        assert!((cal.noise_floor().value() - 0.001).abs() < 0.0001);
        assert!((cal.voice_power().value() - 0.05).abs() < 0.001);
    }

    #[test]
    fn test_with_noise_floor() {
        let cal = CalibrationData::default_values();
        let new_cal = cal.with_noise_floor(AudioPower::new(0.002));
        assert!((new_cal.noise_floor().value() - 0.002).abs() < 0.0001);
    }

    #[test]
    fn test_with_voice_power() {
        let cal = CalibrationData::default_values();
        let new_cal = cal.with_voice_power(AudioPower::new(0.1));
        assert!((new_cal.voice_power().value() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_display() {
        let cal = CalibrationData::default_values();
        let display = format!("{}", cal);
        assert!(display.contains("ruido="));
        assert!(display.contains("voz="));
        assert!(display.contains("umbral="));
    }
}
