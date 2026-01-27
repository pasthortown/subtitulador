"""DTOs de la capa de aplicación."""

from .transcription_request import TranscriptionRequest, TranscriptionStreamRequest
from .transcription_response import (
    TranscriptionResponse,
    TranscriptionData,
    ErrorData,
    HealthResponse,
    ModelsResponse
)

__all__ = [
    "TranscriptionRequest",
    "TranscriptionStreamRequest",
    "TranscriptionResponse",
    "TranscriptionData",
    "ErrorData",
    "HealthResponse",
    "ModelsResponse",
]
