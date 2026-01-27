"""Excepciones personalizadas del sistema."""


class SubtituladorError(Exception):
    """Excepción base del sistema."""

    def __init__(self, message: str, code: str = "UNKNOWN_ERROR"):
        self.message = message
        self.code = code
        super().__init__(self.message)


class ValidationError(SubtituladorError):
    """Error de validación de datos."""

    def __init__(self, message: str, field: str = None):
        self.field = field
        super().__init__(message, code="VALIDATION_ERROR")


class TranscriptionError(SubtituladorError):
    """Error durante la transcripción."""

    def __init__(self, message: str):
        super().__init__(message, code="TRANSCRIPTION_ERROR")


class AudioFormatError(SubtituladorError):
    """Error en el formato de audio."""

    def __init__(self, message: str, expected: str = None, received: str = None):
        self.expected = expected
        self.received = received
        super().__init__(message, code="AUDIO_FORMAT_ERROR")


class ModelNotLoadedError(SubtituladorError):
    """Error cuando el modelo no está cargado."""

    def __init__(self, model_name: str):
        super().__init__(
            f"Model '{model_name}' is not loaded",
            code="MODEL_NOT_LOADED"
        )


class ServiceUnavailableError(SubtituladorError):
    """Error cuando el servicio no está disponible."""

    def __init__(self, service: str):
        super().__init__(
            f"Service '{service}' is unavailable",
            code="SERVICE_UNAVAILABLE"
        )
