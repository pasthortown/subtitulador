"""Entity: AudioBuffer - Representa un buffer de audio para transcripción."""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional
from uuid import UUID, uuid4

import numpy as np

from ..value_objects import AudioFormat, Language


@dataclass
class AudioBuffer:
    """Entidad que representa un buffer de audio para procesar."""

    id: UUID = field(default_factory=uuid4)
    data: np.ndarray = field(default_factory=lambda: np.array([], dtype=np.float32))
    audio_format: AudioFormat = field(default_factory=AudioFormat.whisper_format)
    language: Language = field(default_factory=Language.spanish)
    created_at: datetime = field(default_factory=datetime.utcnow)

    def __post_init__(self):
        # Asegurar que el array sea float32
        if self.data.dtype != np.float32:
            self.data = self.data.astype(np.float32)
        # Normalizar si es necesario
        if len(self.data) > 0 and np.abs(self.data).max() > 1.0:
            self.data = self.data / 32768.0

    @property
    def duration_seconds(self) -> float:
        """Calcula la duración del buffer en segundos."""
        if len(self.data) == 0:
            return 0.0
        return len(self.data) / self.audio_format.sample_rate

    @property
    def duration_ms(self) -> int:
        """Calcula la duración del buffer en milisegundos."""
        return int(self.duration_seconds * 1000)

    @property
    def num_samples(self) -> int:
        """Retorna el número de samples en el buffer."""
        return len(self.data)

    @property
    def is_empty(self) -> bool:
        """Verifica si el buffer está vacío."""
        return len(self.data) == 0

    @property
    def rms_power(self) -> float:
        """Calcula la potencia RMS del buffer."""
        if self.is_empty:
            return 0.0
        return float(np.sqrt(np.mean(self.data ** 2)))

    def is_valid_for_transcription(self, min_duration_ms: int = 300) -> bool:
        """Verifica si el buffer es válido para transcripción."""
        return (
            not self.is_empty and
            self.duration_ms >= min_duration_ms and
            self.audio_format.is_whisper_compatible()
        )

    @classmethod
    def from_bytes(
        cls,
        audio_bytes: bytes,
        audio_format: AudioFormat,
        language: Optional[Language] = None
    ) -> "AudioBuffer":
        """Crea un AudioBuffer desde bytes raw."""
        # Convertir bytes a numpy array
        data = np.frombuffer(audio_bytes, dtype=np.float32)
        return cls(
            data=data,
            audio_format=audio_format,
            language=language or Language.spanish()
        )

    @classmethod
    def from_base64(
        cls,
        base64_data: str,
        audio_format: AudioFormat,
        language: Optional[Language] = None
    ) -> "AudioBuffer":
        """Crea un AudioBuffer desde datos base64."""
        import base64
        audio_bytes = base64.b64decode(base64_data)
        return cls.from_bytes(audio_bytes, audio_format, language)

    def __len__(self) -> int:
        return len(self.data)

    def __repr__(self) -> str:
        return (
            f"AudioBuffer(id={self.id}, samples={self.num_samples}, "
            f"duration={self.duration_seconds:.2f}s, language={self.language})"
        )
