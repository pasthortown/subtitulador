//! Value Object: Duration - Duración temporal.

use std::fmt;
use std::time::Duration as StdDuration;

/// Value Object que representa una duración temporal.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Duration {
    millis: u64,
}

impl Duration {
    /// Crea una duración desde milisegundos.
    pub fn from_millis(millis: u64) -> Self {
        Self { millis }
    }

    /// Crea una duración desde segundos.
    pub fn from_secs(secs: f64) -> Self {
        Self {
            millis: (secs * 1000.0) as u64,
        }
    }

    /// Crea una duración desde samples y sample rate.
    pub fn from_samples(samples: usize, sample_rate: u32) -> Self {
        let secs = samples as f64 / sample_rate as f64;
        Self::from_secs(secs)
    }

    /// Duración cero.
    pub fn zero() -> Self {
        Self { millis: 0 }
    }

    /// Obtiene milisegundos.
    pub fn as_millis(&self) -> u64 {
        self.millis
    }

    /// Obtiene segundos como f64.
    pub fn as_secs_f64(&self) -> f64 {
        self.millis as f64 / 1000.0
    }

    /// Convierte a std::time::Duration.
    pub fn as_std(&self) -> StdDuration {
        StdDuration::from_millis(self.millis)
    }

    /// Verifica si es cero.
    pub fn is_zero(&self) -> bool {
        self.millis == 0
    }

    /// Verifica si es mayor o igual a otra duración.
    pub fn is_gte(&self, other: Duration) -> bool {
        self.millis >= other.millis
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let secs = self.as_secs_f64();
        if secs < 1.0 {
            write!(f, "{}ms", self.millis)
        } else {
            write!(f, "{:.2}s", secs)
        }
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<StdDuration> for Duration {
    fn from(d: StdDuration) -> Self {
        Self::from_millis(d.as_millis() as u64)
    }
}

impl From<Duration> for StdDuration {
    fn from(d: Duration) -> Self {
        StdDuration::from_millis(d.millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_millis() {
        let d = Duration::from_millis(1500);
        assert_eq!(d.as_millis(), 1500);
        assert!((d.as_secs_f64() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_from_samples() {
        // 16000 samples a 16kHz = 1 segundo
        let d = Duration::from_samples(16000, 16000);
        assert_eq!(d.as_millis(), 1000);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Duration::from_millis(500)), "500ms");
        assert_eq!(format!("{}", Duration::from_millis(1500)), "1.50s");
    }

    #[test]
    fn test_from_secs() {
        let d = Duration::from_secs(2.5);
        assert_eq!(d.as_millis(), 2500);
    }

    #[test]
    fn test_zero() {
        let d = Duration::zero();
        assert_eq!(d.as_millis(), 0);
        assert!(d.is_zero());
    }

    #[test]
    fn test_is_gte() {
        let d1 = Duration::from_millis(1000);
        let d2 = Duration::from_millis(500);
        assert!(d1.is_gte(d2));
        assert!(!d2.is_gte(d1));
        assert!(d1.is_gte(d1));
    }

    #[test]
    fn test_as_std() {
        let d = Duration::from_millis(1500);
        let std_d = d.as_std();
        assert_eq!(std_d.as_millis(), 1500);
    }

    #[test]
    fn test_default() {
        let d = Duration::default();
        assert!(d.is_zero());
    }

    #[test]
    fn test_from_std_duration() {
        let std_d = StdDuration::from_millis(2000);
        let d: Duration = std_d.into();
        assert_eq!(d.as_millis(), 2000);
    }

    #[test]
    fn test_into_std_duration() {
        let d = Duration::from_millis(3000);
        let std_d: StdDuration = d.into();
        assert_eq!(std_d.as_millis(), 3000);
    }
}
