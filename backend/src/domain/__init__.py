"""Capa de dominio - Núcleo de la aplicación."""

from .entities import AudioBuffer, Transcription
from .value_objects import Language, AudioFormat, AudioEncoding
from .services import TranscriptionService
from .ports import TranscriptionPort, SpeechRecognitionPort

__all__ = [
    # Entities
    "AudioBuffer",
    "Transcription",
    # Value Objects
    "Language",
    "AudioFormat",
    "AudioEncoding",
    # Services
    "TranscriptionService",
    # Ports
    "TranscriptionPort",
    "SpeechRecognitionPort",
]
