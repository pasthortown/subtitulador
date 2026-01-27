//! Capa de dominio - Núcleo de la aplicación.

pub mod entities;
pub mod value_objects;
pub mod services;
pub mod ports;

pub use entities::*;
pub use value_objects::*;
pub use services::*;
pub use ports::*;
