"""Capa de aplicación - Casos de uso y DTOs."""

from .use_cases import TranscribeAudioUseCase, HealthCheckUseCase, ServiceInfo
from .dtos import (
    TranscriptionRequest,
    TranscriptionStreamRequest,
    TranscriptionResponse,
    TranscriptionData,
    ErrorData,
    HealthResponse,
    ModelsResponse
)

__all__ = [
    # Use Cases
    "TranscribeAudioUseCase",
    "HealthCheckUseCase",
    "ServiceInfo",
    # DTOs
    "TranscriptionRequest",
    "TranscriptionStreamRequest",
    "TranscriptionResponse",
    "TranscriptionData",
    "ErrorData",
    "HealthResponse",
    "ModelsResponse",
]
