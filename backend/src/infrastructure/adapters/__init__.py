"""Adaptadores de infraestructura."""

from .inbound import (
    BaseHandler,
    TranscriptionHandler,
    TranscriptionStreamHandler,
    HealthHandler,
    ModelsHandler
)
from .outbound import WhisperAdapter

__all__ = [
    # Inbound
    "BaseHandler",
    "TranscriptionHandler",
    "TranscriptionStreamHandler",
    "HealthHandler",
    "ModelsHandler",
    # Outbound
    "WhisperAdapter",
]
