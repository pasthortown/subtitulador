//! Puertos de entrada (drivers).

use anyhow::Result;
use crate::domain::AudioPower;

/// Información de un dispositivo de audio.
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    /// Nombre interno del dispositivo (para selección).
    pub name: String,
    /// Nombre amigable para mostrar al usuario.
    pub display_name: String,
    /// Descripción adicional.
    pub description: String,
    /// Es el dispositivo por defecto.
    pub is_default: bool,
}

/// Puerto para captura de audio.
pub trait AudioCapturePort {
    /// Lista los dispositivos de entrada disponibles.
    fn list_input_devices(&self) -> Result<Vec<AudioDeviceInfo>>;

    /// Selecciona un dispositivo por nombre.
    fn select_device(&mut self, device_name: &str) -> Result<()>;

    /// Inicia la captura de audio.
    fn start_capture(&mut self) -> Result<()>;

    /// Detiene la captura de audio.
    fn stop_capture(&mut self);

    /// Obtiene y limpia el buffer de samples (no bloqueante).
    fn take_samples(&self) -> Vec<f32>;

    /// Obtiene la potencia actual del audio.
    fn current_power(&self) -> AudioPower;

    /// Limpia el buffer de samples.
    fn clear_buffer(&self);
}
