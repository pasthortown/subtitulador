"""DTO: TranscriptionRequest - Request para transcripción de audio."""

from typing import Optional
from pydantic import BaseModel, Field, field_validator
import base64


class TranscriptionRequest(BaseModel):
    """DTO para solicitud de transcripción de audio."""

    audio: str = Field(
        ...,
        description="Audio en formato base64 (WAV, PCM float32, 16kHz mono)",
        min_length=100
    )
    language: str = Field(
        default="es",
        description="Código de idioma ISO 639-1",
        min_length=2,
        max_length=5
    )
    sample_rate: int = Field(
        default=16000,
        description="Frecuencia de muestreo del audio",
        ge=8000,
        le=48000
    )

    @field_validator('audio')
    @classmethod
    def validate_audio_base64(cls, v: str) -> str:
        """Valida que el audio sea base64 válido."""
        try:
            decoded = base64.b64decode(v)
            if len(decoded) < 100:
                raise ValueError("Audio data too small")
            return v
        except Exception as e:
            raise ValueError(f"Invalid base64 audio data: {e}")

    @field_validator('language')
    @classmethod
    def validate_language(cls, v: str) -> str:
        """Valida el código de idioma."""
        supported = ['es', 'en', 'pt', 'fr', 'de', 'it', 'ja', 'ko', 'zh', 'ru']
        v = v.lower()
        if v not in supported:
            raise ValueError(f"Unsupported language: {v}. Supported: {supported}")
        return v

    class Config:
        json_schema_extra = {
            "example": {
                "audio": "UklGRi4AAABXQVZFZm10IBAAAAABAAEA...",
                "language": "es",
                "sample_rate": 16000
            }
        }


class TranscriptionStreamRequest(BaseModel):
    """DTO para solicitud de transcripción en streaming."""

    language: str = Field(
        default="es",
        description="Código de idioma ISO 639-1",
        min_length=2,
        max_length=5
    )
    sample_rate: int = Field(
        default=16000,
        description="Frecuencia de muestreo del audio"
    )
    chunk_duration_ms: int = Field(
        default=100,
        description="Duración de cada chunk en milisegundos",
        ge=50,
        le=1000
    )

    @field_validator('language')
    @classmethod
    def validate_language(cls, v: str) -> str:
        """Valida el código de idioma."""
        supported = ['es', 'en', 'pt', 'fr', 'de', 'it', 'ja', 'ko', 'zh', 'ru']
        v = v.lower()
        if v not in supported:
            raise ValueError(f"Unsupported language: {v}")
        return v
