//! Puertos de entrada (drivers).

use crate::domain::{AudioBuffer, Subtitle};
use anyhow::Result;

/// Puerto para captura de audio.
pub trait AudioCapturePort: Send + Sync {
    /// Inicia la captura de audio.
    fn start(&mut self) -> Result<()>;

    /// Detiene la captura de audio.
    fn stop(&mut self) -> Result<()>;

    /// Obtiene los samples capturados (no bloqueante).
    fn get_samples(&mut self) -> Option<Vec<f32>>;

    /// Verifica si está capturando.
    fn is_capturing(&self) -> bool;
}

/// Puerto para la interfaz de usuario.
pub trait UIPort: Send + Sync {
    /// Muestra un subtítulo.
    fn show_subtitle(&mut self, subtitle: Subtitle);

    /// Limpia todos los subtítulos.
    fn clear_subtitles(&mut self);

    /// Actualiza la UI (para animaciones).
    fn update(&mut self);
}
