"""Configuración de logging estructurado."""

import logging
import sys
from typing import Optional

import structlog


def setup_logging(level: str = "INFO", format_type: str = "json") -> None:
    """
    Configura el logging estructurado.

    Args:
        level: Nivel de logging (DEBUG, INFO, WARNING, ERROR, CRITICAL).
        format_type: Formato de salida ('json' o 'console').
    """
    # Configurar nivel
    log_level = getattr(logging, level.upper(), logging.INFO)

    # Procesadores comunes
    shared_processors = [
        structlog.contextvars.merge_contextvars,
        structlog.processors.add_log_level,
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.UnicodeDecoder(),
    ]

    if format_type == "json":
        # Formato JSON para producción
        processors = shared_processors + [
            structlog.processors.format_exc_info,
            structlog.processors.JSONRenderer()
        ]
    else:
        # Formato consola para desarrollo
        processors = shared_processors + [
            structlog.dev.ConsoleRenderer(colors=True)
        ]

    # Configurar structlog
    structlog.configure(
        processors=processors,
        wrapper_class=structlog.stdlib.BoundLogger,
        context_class=dict,
        logger_factory=structlog.stdlib.LoggerFactory(),
        cache_logger_on_first_use=True,
    )

    # Configurar logging estándar
    logging.basicConfig(
        format="%(message)s",
        stream=sys.stdout,
        level=log_level,
    )


def get_logger(name: Optional[str] = None) -> structlog.stdlib.BoundLogger:
    """
    Obtiene un logger estructurado.

    Args:
        name: Nombre del logger (opcional).

    Returns:
        Logger estructurado.
    """
    return structlog.get_logger(name)


# Logger por defecto
logger = get_logger("subtitulador")
