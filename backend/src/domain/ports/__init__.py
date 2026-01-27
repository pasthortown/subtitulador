"""Puertos del dominio (interfaces hexagonales)."""

from .inbound import TranscriptionPort
from .outbound import SpeechRecognitionPort

__all__ = ["TranscriptionPort", "SpeechRecognitionPort"]
