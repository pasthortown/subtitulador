"""Definición de rutas de la API."""

from typing import List, Tuple, Any

from ..adapters import (
    TranscriptionHandler,
    TranscriptionStreamHandler,
    HealthHandler,
    ModelsHandler
)
from ..config.container import Container


def get_routes(container: Container) -> List[Tuple[str, Any, dict]]:
    """
    Obtiene las rutas de la API con dependencias inyectadas.

    Args:
        container: Contenedor de dependencias.

    Returns:
        Lista de tuplas (patrón, handler, kwargs).
    """
    api_prefix = container.settings.api.prefix
    api_version = container.settings.api.version

    return [
        # Health check
        (
            f"{api_prefix}/{api_version}/health",
            HealthHandler,
            {"health_use_case": container.health_use_case}
        ),

        # Models
        (
            f"{api_prefix}/{api_version}/models",
            ModelsHandler,
            {"health_use_case": container.health_use_case}
        ),

        # Transcription
        (
            f"{api_prefix}/{api_version}/transcribe",
            TranscriptionHandler,
            {"transcribe_use_case": container.transcribe_use_case}
        ),

        # Transcription streaming
        (
            f"{api_prefix}/{api_version}/transcribe/stream",
            TranscriptionStreamHandler,
            {"transcribe_use_case": container.transcribe_use_case}
        ),
    ]
