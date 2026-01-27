"""Value Object: Language - Representa un código de idioma válido."""

from dataclasses import dataclass
from typing import ClassVar


@dataclass(frozen=True)
class Language:
    """Value Object inmutable que representa un código de idioma ISO 639-1."""

    code: str

    SUPPORTED_LANGUAGES: ClassVar[dict[str, str]] = {
        "es": "Spanish",
        "en": "English",
        "pt": "Portuguese",
        "fr": "French",
        "de": "German",
        "it": "Italian",
        "ja": "Japanese",
        "ko": "Korean",
        "zh": "Chinese",
        "ru": "Russian",
    }

    def __post_init__(self):
        if not self.code:
            raise ValueError("Language code cannot be empty")
        if self.code.lower() not in self.SUPPORTED_LANGUAGES:
            raise ValueError(f"Unsupported language code: {self.code}")
        # Normalizar a minúsculas
        object.__setattr__(self, 'code', self.code.lower())

    @property
    def name(self) -> str:
        """Obtiene el nombre del idioma."""
        return self.SUPPORTED_LANGUAGES.get(self.code, "Unknown")

    @classmethod
    def spanish(cls) -> "Language":
        """Factory method para español."""
        return cls("es")

    @classmethod
    def english(cls) -> "Language":
        """Factory method para inglés."""
        return cls("en")

    @classmethod
    def portuguese(cls) -> "Language":
        """Factory method para portugués."""
        return cls("pt")

    def __str__(self) -> str:
        return self.code

    def __eq__(self, other: object) -> bool:
        if isinstance(other, Language):
            return self.code == other.code
        if isinstance(other, str):
            return self.code == other.lower()
        return False

    def __hash__(self) -> int:
        return hash(self.code)
