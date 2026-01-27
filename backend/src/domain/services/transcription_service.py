"""Domain Service: TranscriptionService - Lógica de negocio para transcripción."""

from ..entities import AudioBuffer, Transcription
from ..value_objects import Language


class TranscriptionService:
    """
    Servicio de dominio para la lógica de transcripción.

    Contiene reglas de negocio puras sin dependencias de infraestructura.
    """

    MIN_AUDIO_DURATION_MS = 300
    MAX_AUDIO_DURATION_MS = 30000

    def validate_audio_buffer(self, audio_buffer: AudioBuffer) -> list[str]:
        """
        Valida un buffer de audio para transcripción.

        Args:
            audio_buffer: Buffer a validar.

        Returns:
            Lista de errores de validación (vacía si es válido).
        """
        errors = []

        if audio_buffer.is_empty:
            errors.append("Audio buffer is empty")

        if audio_buffer.duration_ms < self.MIN_AUDIO_DURATION_MS:
            errors.append(
                f"Audio duration too short: {audio_buffer.duration_ms}ms "
                f"(minimum: {self.MIN_AUDIO_DURATION_MS}ms)"
            )

        if audio_buffer.duration_ms > self.MAX_AUDIO_DURATION_MS:
            errors.append(
                f"Audio duration too long: {audio_buffer.duration_ms}ms "
                f"(maximum: {self.MAX_AUDIO_DURATION_MS}ms)"
            )

        if not audio_buffer.audio_format.is_whisper_compatible():
            errors.append(
                f"Audio format not compatible with Whisper: "
                f"{audio_buffer.audio_format}"
            )

        return errors

    def is_valid_language(self, language_code: str) -> bool:
        """
        Verifica si un código de idioma es válido.

        Args:
            language_code: Código ISO 639-1 del idioma.

        Returns:
            True si el idioma es soportado.
        """
        try:
            Language(language_code)
            return True
        except ValueError:
            return False

    def calculate_expected_processing_time(
        self,
        audio_duration_ms: int,
        is_cpu: bool = True
    ) -> int:
        """
        Estima el tiempo de procesamiento esperado.

        Args:
            audio_duration_ms: Duración del audio en ms.
            is_cpu: True si se usa CPU (más lento).

        Returns:
            Tiempo estimado de procesamiento en ms.
        """
        # Factor de procesamiento (ratio tiempo procesamiento / duración audio)
        # CPU es aproximadamente 0.5-1.0x tiempo real para large-v3-turbo
        factor = 0.7 if is_cpu else 0.1
        return int(audio_duration_ms * factor)

    def should_process_realtime(
        self,
        processing_time_ms: int,
        audio_duration_ms: int
    ) -> bool:
        """
        Determina si el procesamiento fue en tiempo real.

        Args:
            processing_time_ms: Tiempo de procesamiento real.
            audio_duration_ms: Duración del audio procesado.

        Returns:
            True si el procesamiento fue en tiempo real (< 1x).
        """
        if audio_duration_ms == 0:
            return False
        return processing_time_ms < audio_duration_ms

    def create_empty_transcription(
        self,
        language: Language,
        audio_duration_ms: int = 0
    ) -> Transcription:
        """
        Crea una transcripción vacía (para silencios o errores).

        Args:
            language: Idioma de la transcripción.
            audio_duration_ms: Duración del audio (si aplica).

        Returns:
            Transcription vacía.
        """
        return Transcription(
            text="",
            language=language,
            confidence=0.0,
            audio_duration_ms=audio_duration_ms,
            processing_time_ms=0
        )

    def normalize_transcription_text(self, text: str) -> str:
        """
        Normaliza el texto de una transcripción.

        Args:
            text: Texto a normalizar.

        Returns:
            Texto normalizado.
        """
        if not text:
            return ""

        # Limpiar espacios múltiples
        text = " ".join(text.split())

        # Capitalizar primera letra
        if text:
            text = text[0].upper() + text[1:]

        return text.strip()
