"""Use Case: HealthCheck - Caso de uso para verificar estado del servicio."""

import time
from dataclasses import dataclass

from ...domain import SpeechRecognitionPort
from ..dtos import HealthResponse, ModelsResponse


@dataclass
class ServiceInfo:
    """Información del servicio."""
    version: str = "1.0.0"
    start_time: float = 0.0


class HealthCheckUseCase:
    """
    Caso de uso para verificar el estado del servicio.
    """

    def __init__(
        self,
        speech_recognition: SpeechRecognitionPort,
        service_info: ServiceInfo
    ):
        self._speech_recognition = speech_recognition
        self._service_info = service_info

    async def execute(self) -> HealthResponse:
        """
        Ejecuta el health check.

        Returns:
            HealthResponse con el estado del servicio.
        """
        is_available = await self._speech_recognition.is_available()
        model_info = self._speech_recognition.get_model_info()

        uptime = time.time() - self._service_info.start_time

        return HealthResponse(
            status="healthy" if is_available else "unhealthy",
            version=self._service_info.version,
            model=model_info.get("name", "unknown"),
            device=model_info.get("device", "cpu"),
            uptime_seconds=uptime
        )

    def get_models(self) -> ModelsResponse:
        """
        Obtiene información de los modelos.

        Returns:
            ModelsResponse con los modelos disponibles.
        """
        model_info = self._speech_recognition.get_model_info()
        languages = self._speech_recognition.get_supported_languages()

        return ModelsResponse(
            models=[
                {
                    "name": model_info.get("name", "unknown"),
                    "device": model_info.get("device", "cpu"),
                    "compute_type": model_info.get("compute_type", "int8"),
                    "languages": languages
                }
            ],
            current_model=model_info.get("name", "unknown")
        )
