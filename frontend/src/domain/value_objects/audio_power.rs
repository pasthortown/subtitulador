//! Value Object: AudioPower - Potencia de audio RMS.

use std::fmt;

/// Value Object que representa la potencia RMS de una señal de audio.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AudioPower {
    value: f32,
}

impl AudioPower {
    /// Crea un nuevo AudioPower.
    pub fn new(value: f32) -> Self {
        Self {
            value: value.max(0.0),
        }
    }

    /// Crea AudioPower desde samples de audio.
    pub fn from_samples(samples: &[f32]) -> Self {
        if samples.is_empty() {
            return Self::zero();
        }

        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        let rms = (sum_squares / samples.len() as f32).sqrt();

        Self::new(rms)
    }

    /// Potencia cero (silencio completo).
    pub fn zero() -> Self {
        Self { value: 0.0 }
    }

    /// Obtiene el valor de potencia.
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Convierte a decibeles.
    pub fn to_db(&self) -> f32 {
        if self.value <= 0.0 {
            f32::NEG_INFINITY
        } else {
            20.0 * self.value.log10()
        }
    }

    /// Verifica si está por debajo de un umbral.
    pub fn is_below(&self, threshold: AudioPower) -> bool {
        self.value < threshold.value
    }

    /// Verifica si está por encima de un umbral.
    pub fn is_above(&self, threshold: AudioPower) -> bool {
        self.value > threshold.value
    }
}

impl fmt::Display for AudioPower {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}", self.value)
    }
}

impl Default for AudioPower {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_samples() {
        let samples = vec![0.5_f32; 100];
        let power = AudioPower::from_samples(&samples);
        assert!((power.value() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_empty_samples() {
        let power = AudioPower::from_samples(&[]);
        assert_eq!(power.value(), 0.0);
    }

    #[test]
    fn test_comparison() {
        let low = AudioPower::new(0.1);
        let high = AudioPower::new(0.5);

        assert!(low.is_below(high));
        assert!(high.is_above(low));
    }

    #[test]
    fn test_negative_value_clamped() {
        let power = AudioPower::new(-0.5);
        assert_eq!(power.value(), 0.0);
    }

    #[test]
    fn test_to_db() {
        let power = AudioPower::new(1.0);
        assert!((power.to_db() - 0.0).abs() < 0.001);

        let power_half = AudioPower::new(0.1);
        assert!(power_half.to_db() < 0.0);
    }

    #[test]
    fn test_to_db_zero() {
        let power = AudioPower::zero();
        assert!(power.to_db().is_infinite());
    }

    #[test]
    fn test_display() {
        let power = AudioPower::new(0.123456);
        assert_eq!(format!("{}", power), "0.123456");
    }

    #[test]
    fn test_default() {
        let power = AudioPower::default();
        assert_eq!(power.value(), 0.0);
    }

    #[test]
    fn test_zero() {
        let power = AudioPower::zero();
        assert_eq!(power.value(), 0.0);
    }
}
