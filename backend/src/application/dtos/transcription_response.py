"""DTO: TranscriptionResponse - Response de transcripción de audio."""

from typing import Optional, Any
from pydantic import BaseModel, Field
from datetime import datetime


class TranscriptionData(BaseModel):
    """Datos de la transcripción."""

    text: str = Field(
        ...,
        description="Texto transcrito"
    )
    language: str = Field(
        ...,
        description="Código de idioma de la transcripción"
    )
    confidence: float = Field(
        ...,
        description="Nivel de confianza (0.0-1.0)",
        ge=0.0,
        le=1.0
    )
    duration_ms: int = Field(
        ...,
        description="Duración del audio en milisegundos",
        ge=0
    )
    processing_time_ms: int = Field(
        ...,
        description="Tiempo de procesamiento en milisegundos",
        ge=0
    )

    class Config:
        json_schema_extra = {
            "example": {
                "text": "Hola, este es un texto de prueba",
                "language": "es",
                "confidence": 0.95,
                "duration_ms": 2340,
                "processing_time_ms": 180
            }
        }


class ErrorData(BaseModel):
    """Datos de error."""

    code: str = Field(
        ...,
        description="Código de error"
    )
    message: str = Field(
        ...,
        description="Mensaje de error descriptivo"
    )
    details: Optional[dict[str, Any]] = Field(
        default=None,
        description="Detalles adicionales del error"
    )

    class Config:
        json_schema_extra = {
            "example": {
                "code": "INVALID_AUDIO_FORMAT",
                "message": "El formato de audio no es válido",
                "details": {"expected": "WAV 16kHz mono", "received": "MP3"}
            }
        }


class TranscriptionResponse(BaseModel):
    """Response estándar para transcripción."""

    success: bool = Field(
        ...,
        description="Indica si la operación fue exitosa"
    )
    data: Optional[TranscriptionData] = Field(
        default=None,
        description="Datos de la transcripción (si success=true)"
    )
    error: Optional[ErrorData] = Field(
        default=None,
        description="Datos del error (si success=false)"
    )
    timestamp: datetime = Field(
        default_factory=datetime.utcnow,
        description="Timestamp de la respuesta"
    )

    @classmethod
    def ok(
        cls,
        text: str,
        language: str,
        confidence: float,
        duration_ms: int,
        processing_time_ms: int
    ) -> "TranscriptionResponse":
        """Factory method para respuesta exitosa."""
        return cls(
            success=True,
            data=TranscriptionData(
                text=text,
                language=language,
                confidence=confidence,
                duration_ms=duration_ms,
                processing_time_ms=processing_time_ms
            )
        )

    @classmethod
    def failure(
        cls,
        code: str,
        message: str,
        details: Optional[dict] = None
    ) -> "TranscriptionResponse":
        """Factory method para respuesta de error."""
        return cls(
            success=False,
            error=ErrorData(
                code=code,
                message=message,
                details=details
            )
        )

    class Config:
        json_schema_extra = {
            "example": {
                "success": True,
                "data": {
                    "text": "Hola, este es un texto de prueba",
                    "language": "es",
                    "confidence": 0.95,
                    "duration_ms": 2340,
                    "processing_time_ms": 180
                },
                "error": None,
                "timestamp": "2024-01-15T10:30:00Z"
            }
        }


class HealthResponse(BaseModel):
    """Response para health check."""

    status: str = Field(
        ...,
        description="Estado del servicio"
    )
    version: str = Field(
        ...,
        description="Versión de la API"
    )
    model: str = Field(
        ...,
        description="Modelo de Whisper cargado"
    )
    device: str = Field(
        ...,
        description="Dispositivo de procesamiento (cpu/cuda)"
    )
    uptime_seconds: float = Field(
        ...,
        description="Tiempo de actividad en segundos"
    )

    class Config:
        json_schema_extra = {
            "example": {
                "status": "healthy",
                "version": "1.0.0",
                "model": "large-v3-turbo",
                "device": "cpu",
                "uptime_seconds": 3600.5
            }
        }


class ModelsResponse(BaseModel):
    """Response para lista de modelos."""

    models: list[dict[str, Any]] = Field(
        ...,
        description="Lista de modelos disponibles"
    )
    current_model: str = Field(
        ...,
        description="Modelo actualmente cargado"
    )
