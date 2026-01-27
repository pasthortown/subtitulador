//! Value Object: Language - Código de idioma ISO 639-1.

use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LanguageError {
    #[error("Idioma no soportado: {0}")]
    UnsupportedLanguage(String),
    #[error("Código de idioma vacío")]
    EmptyCode,
}

/// Value Object inmutable que representa un código de idioma.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Language {
    code: String,
}

impl Language {
    /// Idiomas soportados
    const SUPPORTED: &'static [(&'static str, &'static str)] = &[
        ("es", "Spanish"),
        ("en", "English"),
        ("pt", "Portuguese"),
        ("fr", "French"),
        ("de", "German"),
        ("it", "Italian"),
        ("ja", "Japanese"),
        ("ko", "Korean"),
        ("zh", "Chinese"),
        ("ru", "Russian"),
    ];

    /// Crea un nuevo Language validado.
    pub fn new(code: &str) -> Result<Self, LanguageError> {
        if code.is_empty() {
            return Err(LanguageError::EmptyCode);
        }

        let code_lower = code.to_lowercase();

        if !Self::is_supported(&code_lower) {
            return Err(LanguageError::UnsupportedLanguage(code.to_string()));
        }

        Ok(Self { code: code_lower })
    }

    /// Verifica si un código de idioma es soportado.
    pub fn is_supported(code: &str) -> bool {
        Self::SUPPORTED.iter().any(|(c, _)| *c == code)
    }

    /// Obtiene el código del idioma.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Obtiene el nombre del idioma.
    pub fn name(&self) -> &str {
        Self::SUPPORTED
            .iter()
            .find(|(c, _)| *c == self.code)
            .map(|(_, name)| *name)
            .unwrap_or("Unknown")
    }

    /// Factory methods
    pub fn spanish() -> Self {
        Self { code: "es".to_string() }
    }

    pub fn english() -> Self {
        Self { code: "en".to_string() }
    }

    pub fn portuguese() -> Self {
        Self { code: "pt".to_string() }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code)
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::spanish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_valid_language() {
        let lang = Language::new("es").unwrap();
        assert_eq!(lang.code(), "es");
        assert_eq!(lang.name(), "Spanish");
    }

    #[test]
    fn test_invalid_language_fails() {
        assert!(Language::new("xx").is_err());
    }

    #[test]
    fn test_empty_language_fails() {
        assert!(Language::new("").is_err());
    }

    #[test]
    fn test_case_insensitive() {
        let lang = Language::new("ES").unwrap();
        assert_eq!(lang.code(), "es");
    }
}
