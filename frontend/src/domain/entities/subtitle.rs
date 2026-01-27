//! Entity: Subtitle - Subtítulo para mostrar en pantalla.

use std::time::Instant;
use uuid::Uuid;

use crate::domain::Duration;

/// Multiplicador para tiempo de visualización.
const DISPLAY_TIME_MULTIPLIER: f64 = 2.5;
/// Duración mínima de visualización en ms.
const MIN_DISPLAY_MS: u64 = 2000;
/// Duración del fade en ms.
const FADE_DURATION_MS: u64 = 1000;

/// Entidad que representa un subtítulo para mostrar.
#[derive(Debug, Clone)]
pub struct Subtitle {
    id: Uuid,
    text: String,
    buffer_duration: Duration,
    display_duration: Duration,
    appear_time: Option<Instant>,
    expire_time: Option<Instant>,
}

impl Subtitle {
    /// Crea un nuevo subtítulo.
    pub fn new(text: String, buffer_duration: Duration) -> Self {
        // Tiempo de visualización = max(2s, buffer_duration * 2.5)
        let display_ms = (buffer_duration.as_millis() as f64 * DISPLAY_TIME_MULTIPLIER) as u64;
        let display_duration = Duration::from_millis(display_ms.max(MIN_DISPLAY_MS));

        Self {
            id: Uuid::new_v4(),
            text,
            buffer_duration,
            display_duration,
            appear_time: None,
            expire_time: None,
        }
    }

    /// ID único.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Texto del subtítulo.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Duración del buffer de audio que generó este subtítulo.
    pub fn buffer_duration(&self) -> Duration {
        self.buffer_duration
    }

    /// Duración de visualización.
    pub fn display_duration(&self) -> Duration {
        self.display_duration
    }

    /// Marca el subtítulo como visible ahora.
    pub fn show(&mut self) {
        let now = Instant::now();
        self.appear_time = Some(now);
        self.expire_time = Some(now + self.display_duration.as_std());
    }

    /// Marca el subtítulo para expirar después de otro.
    pub fn show_after(&mut self, previous_expire: Instant) {
        let now = Instant::now();
        self.appear_time = Some(now);
        self.expire_time = Some(previous_expire + self.display_duration.as_std());
    }

    /// Verifica si el subtítulo ha expirado.
    pub fn is_expired(&self) -> bool {
        match self.expire_time {
            Some(expire) => Instant::now() >= expire,
            None => false,
        }
    }

    /// Verifica si ya pasó el buffer_duration para mostrar el siguiente.
    pub fn can_show_next(&self) -> bool {
        match self.appear_time {
            Some(appear) => {
                let elapsed = appear.elapsed();
                elapsed >= self.buffer_duration.as_std()
            }
            None => false,
        }
    }

    /// Tiempo de expiración.
    pub fn expire_time(&self) -> Option<Instant> {
        self.expire_time
    }

    /// Calcula la opacidad para fade (1.0 = opaco, 0.0 = transparente).
    pub fn opacity(&self) -> f32 {
        let expire = match self.expire_time {
            Some(e) => e,
            None => return 1.0,
        };

        let now = Instant::now();
        if now >= expire {
            return 0.0;
        }

        let remaining = expire.duration_since(now);
        let remaining_ms = remaining.as_millis() as u64;

        if remaining_ms >= FADE_DURATION_MS {
            1.0
        } else {
            remaining_ms as f32 / FADE_DURATION_MS as f32
        }
    }

    /// Verifica si está vacío.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_subtitle() {
        let sub = Subtitle::new(
            "Hola mundo".to_string(),
            Duration::from_millis(2000),
        );

        assert_eq!(sub.text(), "Hola mundo");
        assert_eq!(sub.buffer_duration().as_millis(), 2000);
        // display = max(2000, 2000 * 2.5) = 5000
        assert_eq!(sub.display_duration().as_millis(), 5000);
    }

    #[test]
    fn test_min_display_duration() {
        let sub = Subtitle::new(
            "Corto".to_string(),
            Duration::from_millis(500),
        );

        // display = max(2000, 500 * 2.5) = 2000 (mínimo)
        assert_eq!(sub.display_duration().as_millis(), 2000);
    }

    #[test]
    fn test_show_and_expire() {
        let mut sub = Subtitle::new(
            "Test".to_string(),
            Duration::from_millis(100),
        );

        assert!(!sub.is_expired());

        sub.show();
        assert!(!sub.is_expired()); // Recién mostrado
    }
}
