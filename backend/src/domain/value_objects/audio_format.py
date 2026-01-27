"""Value Object: AudioFormat - Representa el formato de audio."""

from dataclasses import dataclass
from enum import Enum
from typing import ClassVar


class AudioEncoding(Enum):
    """Tipos de codificación de audio soportados."""
    PCM_S16LE = "pcm_s16le"  # PCM 16-bit little-endian
    PCM_F32LE = "pcm_f32le"  # PCM 32-bit float little-endian
    WAV = "wav"
    MP3 = "mp3"
    OGG = "ogg"
    FLAC = "flac"


@dataclass(frozen=True)
class AudioFormat:
    """Value Object inmutable que representa el formato de audio."""

    sample_rate: int
    channels: int
    encoding: AudioEncoding

    # Constantes para Whisper
    WHISPER_SAMPLE_RATE: ClassVar[int] = 16000
    WHISPER_CHANNELS: ClassVar[int] = 1

    def __post_init__(self):
        if self.sample_rate <= 0:
            raise ValueError(f"Sample rate must be positive: {self.sample_rate}")
        if self.channels <= 0:
            raise ValueError(f"Channels must be positive: {self.channels}")
        if self.sample_rate > 192000:
            raise ValueError(f"Sample rate too high: {self.sample_rate}")
        if self.channels > 8:
            raise ValueError(f"Too many channels: {self.channels}")

    @classmethod
    def whisper_format(cls) -> "AudioFormat":
        """Factory method para el formato requerido por Whisper."""
        return cls(
            sample_rate=cls.WHISPER_SAMPLE_RATE,
            channels=cls.WHISPER_CHANNELS,
            encoding=AudioEncoding.PCM_F32LE
        )

    @classmethod
    def wav_16k_mono(cls) -> "AudioFormat":
        """Factory method para WAV 16kHz mono."""
        return cls(
            sample_rate=16000,
            channels=1,
            encoding=AudioEncoding.WAV
        )

    def is_whisper_compatible(self) -> bool:
        """Verifica si el formato es compatible con Whisper."""
        return (
            self.sample_rate == self.WHISPER_SAMPLE_RATE and
            self.channels == self.WHISPER_CHANNELS
        )

    def bytes_per_second(self) -> int:
        """Calcula bytes por segundo para PCM."""
        bytes_per_sample = 4 if self.encoding == AudioEncoding.PCM_F32LE else 2
        return self.sample_rate * self.channels * bytes_per_sample

    def duration_from_samples(self, num_samples: int) -> float:
        """Calcula la duración en segundos dado el número de samples."""
        return num_samples / self.sample_rate

    def samples_from_duration(self, duration_seconds: float) -> int:
        """Calcula el número de samples para una duración dada."""
        return int(duration_seconds * self.sample_rate)

    def __str__(self) -> str:
        return f"{self.sample_rate}Hz/{self.channels}ch/{self.encoding.value}"
