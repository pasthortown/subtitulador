"""Port: SpeechRecognitionPort - Interfaz para servicios de reconocimiento de voz."""

from abc import ABC, abstractmethod

from ...entities import AudioBuffer, Transcription


class SpeechRecognitionPort(ABC):
    """
    Puerto de salida (driven) para servicios de reconocimiento de voz.

    Define la interfaz que debe implementar cualquier adaptador de
    reconocimiento de voz (Whisper, Google Speech, etc.).
    """

    @abstractmethod
    async def transcribe(self, audio_buffer: AudioBuffer) -> Transcription:
        """
        Transcribe un buffer de audio a texto.

        Args:
            audio_buffer: Buffer de audio a transcribir.

        Returns:
            Transcription con el texto transcrito y metadatos.

        Raises:
            TranscriptionError: Si ocurre un error durante la transcripción.
        """
        pass

    @abstractmethod
    async def is_available(self) -> bool:
        """
        Verifica si el servicio de reconocimiento está disponible.

        Returns:
            True si el servicio está listo para procesar.
        """
        pass

    @abstractmethod
    def get_supported_languages(self) -> list[str]:
        """
        Obtiene la lista de idiomas soportados.

        Returns:
            Lista de códigos de idioma ISO 639-1 soportados.
        """
        pass

    @abstractmethod
    def get_model_info(self) -> dict:
        """
        Obtiene información sobre el modelo cargado.

        Returns:
            Diccionario con información del modelo (nombre, tamaño, etc.).
        """
        pass
