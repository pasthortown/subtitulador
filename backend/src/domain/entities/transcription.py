"""Entity: Transcription - Representa el resultado de una transcripción."""

from dataclasses import dataclass, field
from datetime import datetime
from typing import Optional
from uuid import UUID, uuid4

from ..value_objects import Language


@dataclass
class Transcription:
    """Entidad que representa el resultado de una transcripción de audio."""

    id: UUID = field(default_factory=uuid4)
    text: str = ""
    language: Language = field(default_factory=Language.spanish)
    confidence: float = 0.0
    audio_duration_ms: int = 0
    processing_time_ms: int = 0
    created_at: datetime = field(default_factory=datetime.utcnow)
    audio_buffer_id: Optional[UUID] = None

    def __post_init__(self):
        # Limpiar texto
        self.text = self.text.strip()
        # Validar confidence
        if not 0.0 <= self.confidence <= 1.0:
            self.confidence = max(0.0, min(1.0, self.confidence))

    @property
    def is_empty(self) -> bool:
        """Verifica si la transcripción está vacía."""
        return not self.text

    @property
    def word_count(self) -> int:
        """Cuenta el número de palabras en la transcripción."""
        if self.is_empty:
            return 0
        return len(self.text.split())

    @property
    def char_count(self) -> int:
        """Cuenta el número de caracteres en la transcripción."""
        return len(self.text)

    @property
    def processing_ratio(self) -> float:
        """Calcula el ratio de tiempo de procesamiento vs duración de audio."""
        if self.audio_duration_ms == 0:
            return 0.0
        return self.processing_time_ms / self.audio_duration_ms

    @property
    def is_realtime(self) -> bool:
        """Verifica si el procesamiento fue en tiempo real (ratio < 1)."""
        return self.processing_ratio < 1.0

    @classmethod
    def empty(cls, language: Optional[Language] = None) -> "Transcription":
        """Factory method para una transcripción vacía."""
        return cls(
            text="",
            language=language or Language.spanish(),
            confidence=0.0
        )

    @classmethod
    def from_whisper_result(
        cls,
        result: dict,
        language: Language,
        audio_duration_ms: int,
        processing_time_ms: int,
        audio_buffer_id: Optional[UUID] = None
    ) -> "Transcription":
        """Crea una Transcription desde el resultado de Whisper."""
        text = result.get("text", "").strip()

        # Extraer confianza promedio de los segmentos si está disponible
        segments = result.get("segments", [])
        if segments:
            confidences = [
                seg.get("avg_logprob", 0) for seg in segments
                if "avg_logprob" in seg
            ]
            # Convertir log prob a probabilidad aproximada
            if confidences:
                avg_logprob = sum(confidences) / len(confidences)
                confidence = min(1.0, max(0.0, 1.0 + avg_logprob / 5))
            else:
                confidence = 0.9  # Default si no hay info
        else:
            confidence = 0.9 if text else 0.0

        return cls(
            text=text,
            language=language,
            confidence=confidence,
            audio_duration_ms=audio_duration_ms,
            processing_time_ms=processing_time_ms,
            audio_buffer_id=audio_buffer_id
        )

    def __str__(self) -> str:
        return self.text

    def __repr__(self) -> str:
        return (
            f"Transcription(id={self.id}, text='{self.text[:50]}...', "
            f"confidence={self.confidence:.2f}, language={self.language})"
        )

    def __bool__(self) -> bool:
        return not self.is_empty
