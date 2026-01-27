"""Port: TranscriptionPort - Interfaz para el caso de uso de transcripción."""

from abc import ABC, abstractmethod
from typing import AsyncGenerator

from ...entities import AudioBuffer, Transcription


class TranscriptionPort(ABC):
    """
    Puerto de entrada (driver) para transcripción de audio.

    Define la interfaz que expone el dominio para transcribir audio.
    Los adaptadores de entrada (HTTP, CLI, etc.) usan este puerto.
    """

    @abstractmethod
    async def transcribe(self, audio_buffer: AudioBuffer) -> Transcription:
        """
        Transcribe un buffer de audio completo.

        Args:
            audio_buffer: Buffer de audio a transcribir.

        Returns:
            Transcription con el resultado.

        Raises:
            InvalidAudioError: Si el audio no es válido.
            TranscriptionError: Si falla la transcripción.
        """
        pass

    @abstractmethod
    async def transcribe_stream(
        self,
        audio_chunks: AsyncGenerator[bytes, None],
        language: str
    ) -> AsyncGenerator[Transcription, None]:
        """
        Transcribe un stream de audio en tiempo real.

        Args:
            audio_chunks: Generador asíncrono de chunks de audio.
            language: Código de idioma para la transcripción.

        Yields:
            Transcription parciales conforme se procesan.

        Raises:
            InvalidAudioError: Si el formato de audio no es válido.
            TranscriptionError: Si falla la transcripción.
        """
        pass

    @abstractmethod
    async def health_check(self) -> dict:
        """
        Verifica el estado del servicio de transcripción.

        Returns:
            Diccionario con información de estado.
        """
        pass
