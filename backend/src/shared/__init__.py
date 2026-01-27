"""Módulos compartidos."""

from .exceptions import (
    SubtituladorError,
    ValidationError,
    TranscriptionError,
    AudioFormatError
)
from .logger import get_logger, setup_logging

__all__ = [
    "SubtituladorError",
    "ValidationError",
    "TranscriptionError",
    "AudioFormatError",
    "get_logger",
    "setup_logging",
]
