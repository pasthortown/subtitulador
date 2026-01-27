"""Configuración de la aplicación."""

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import yaml


@dataclass
class ServerConfig:
    """Configuración del servidor."""
    host: str = "0.0.0.0"
    port: int = 8000
    workers: int = 2
    debug: bool = False


@dataclass
class WhisperConfig:
    """Configuración de Whisper."""
    model: str = "large-v3-turbo"
    device: str = "cpu"
    compute_type: str = "int8"
    language: str = "es"
    models_path: str = "/app/models"


@dataclass
class AudioConfig:
    """Configuración de audio."""
    sample_rate: int = 16000
    channels: int = 1
    min_duration_ms: int = 300
    max_duration_ms: int = 30000


@dataclass
class ApiConfig:
    """Configuración de la API."""
    version: str = "v1"
    prefix: str = "/api"
    docs_url: str = "/docs"
    openapi_url: str = "/openapi.json"


@dataclass
class LoggingConfig:
    """Configuración de logging."""
    level: str = "INFO"
    format: str = "json"
    include_timestamp: bool = True


@dataclass
class CorsConfig:
    """Configuración CORS."""
    allow_origins: list[str] = field(default_factory=lambda: ["*"])
    allow_methods: list[str] = field(default_factory=lambda: ["GET", "POST", "OPTIONS"])
    allow_headers: list[str] = field(default_factory=lambda: ["*"])


@dataclass
class Settings:
    """Configuración completa de la aplicación."""
    server: ServerConfig = field(default_factory=ServerConfig)
    whisper: WhisperConfig = field(default_factory=WhisperConfig)
    audio: AudioConfig = field(default_factory=AudioConfig)
    api: ApiConfig = field(default_factory=ApiConfig)
    logging: LoggingConfig = field(default_factory=LoggingConfig)
    cors: CorsConfig = field(default_factory=CorsConfig)

    @classmethod
    def load(cls, config_path: Optional[Path] = None) -> "Settings":
        """
        Carga la configuración desde archivo YAML y variables de entorno.

        Las variables de entorno tienen prioridad sobre el archivo.
        """
        settings = cls()

        # Cargar desde archivo si existe
        if config_path and config_path.exists():
            with open(config_path) as f:
                config_data = yaml.safe_load(f)
                settings = cls._from_dict(config_data)

        # Override con variables de entorno
        settings = cls._override_from_env(settings)

        return settings

    @classmethod
    def _from_dict(cls, data: dict) -> "Settings":
        """Crea Settings desde un diccionario."""
        return cls(
            server=ServerConfig(**data.get("server", {})),
            whisper=WhisperConfig(**data.get("whisper", {})),
            audio=AudioConfig(**data.get("audio", {})),
            api=ApiConfig(**data.get("api", {})),
            logging=LoggingConfig(**data.get("logging", {})),
            cors=CorsConfig(**data.get("cors", {}))
        )

    @classmethod
    def _override_from_env(cls, settings: "Settings") -> "Settings":
        """Override configuración con variables de entorno."""
        # Server
        if port := os.getenv("PORT"):
            settings.server.port = int(port)
        if workers := os.getenv("WORKERS"):
            settings.server.workers = int(workers)
        if debug := os.getenv("DEBUG"):
            settings.server.debug = debug.lower() in ("true", "1", "yes")

        # Whisper
        if model := os.getenv("WHISPER_MODEL"):
            settings.whisper.model = model
        if device := os.getenv("WHISPER_DEVICE"):
            settings.whisper.device = device
        if compute_type := os.getenv("WHISPER_COMPUTE_TYPE"):
            settings.whisper.compute_type = compute_type
        if language := os.getenv("DEFAULT_LANGUAGE"):
            settings.whisper.language = language
        if models_path := os.getenv("MODELS_PATH"):
            settings.whisper.models_path = models_path

        # Logging
        if log_level := os.getenv("LOG_LEVEL"):
            settings.logging.level = log_level

        return settings


# Singleton de configuración
_settings: Optional[Settings] = None


def get_settings(config_path: Optional[Path] = None) -> Settings:
    """Obtiene la configuración (singleton)."""
    global _settings
    if _settings is None:
        _settings = Settings.load(config_path)
    return _settings
