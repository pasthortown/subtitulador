"""Contenedor de inyección de dependencias."""

import time
from pathlib import Path
from dataclasses import dataclass
from typing import Optional

from ...domain import TranscriptionService
from ...application import (
    TranscribeAudioUseCase,
    HealthCheckUseCase,
    ServiceInfo
)
from ..adapters import WhisperAdapter
from .settings import Settings, get_settings


@dataclass
class Container:
    """
    Contenedor de dependencias de la aplicación.

    Implementa inyección de dependencias manual para mantener
    el control explícito sobre la creación de objetos.
    """

    settings: Settings
    whisper_adapter: WhisperAdapter
    transcription_service: TranscriptionService
    transcribe_use_case: TranscribeAudioUseCase
    health_use_case: HealthCheckUseCase
    service_info: ServiceInfo

    @classmethod
    def create(cls, config_path: Optional[Path] = None) -> "Container":
        """
        Crea el contenedor con todas las dependencias.

        Args:
            config_path: Ruta opcional al archivo de configuración.

        Returns:
            Container configurado y listo para usar.
        """
        # Cargar configuración
        settings = get_settings(config_path)

        # Crear adaptadores de salida (driven)
        whisper_adapter = WhisperAdapter(
            model_name=settings.whisper.model,
            models_path=Path(settings.whisper.models_path),
            device=settings.whisper.device,
            compute_type=settings.whisper.compute_type,
            num_workers=settings.server.workers
        )

        # Crear servicios de dominio
        transcription_service = TranscriptionService()

        # Información del servicio
        service_info = ServiceInfo(
            version="1.0.0",
            start_time=time.time()
        )

        # Crear casos de uso
        transcribe_use_case = TranscribeAudioUseCase(
            speech_recognition=whisper_adapter,
            transcription_service=transcription_service
        )

        health_use_case = HealthCheckUseCase(
            speech_recognition=whisper_adapter,
            service_info=service_info
        )

        return cls(
            settings=settings,
            whisper_adapter=whisper_adapter,
            transcription_service=transcription_service,
            transcribe_use_case=transcribe_use_case,
            health_use_case=health_use_case,
            service_info=service_info
        )

    def initialize(self) -> None:
        """Inicializa componentes que requieren carga inicial."""
        print("Initializing container...")
        self.whisper_adapter.load_model()
        print("Container initialized successfully")

    def shutdown(self) -> None:
        """Libera recursos del contenedor."""
        print("Shutting down container...")
        self.whisper_adapter.shutdown()
        print("Container shutdown complete")


# Singleton del contenedor
_container: Optional[Container] = None


def get_container(config_path: Optional[Path] = None) -> Container:
    """Obtiene el contenedor (singleton)."""
    global _container
    if _container is None:
        _container = Container.create(config_path)
    return _container


def initialize_container(config_path: Optional[Path] = None) -> Container:
    """Inicializa y retorna el contenedor."""
    container = get_container(config_path)
    container.initialize()
    return container
