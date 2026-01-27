#!/usr/bin/env python3
"""
Subtitulador Backend - Punto de entrada principal.

Servicio de transcripción de audio en tiempo real usando Whisper.
"""

import sys
from pathlib import Path

from .shared.logger import setup_logging, get_logger
from .infrastructure.web import run_server


def main():
    """Punto de entrada principal."""
    # Configurar logging
    setup_logging(level="INFO", format_type="console")
    logger = get_logger("main")

    logger.info("Starting Subtitulador Backend...")

    # Ruta de configuración
    config_path = Path(__file__).parent.parent / "config" / "settings.yaml"

    try:
        # Ejecutar servidor
        run_server(config_path if config_path.exists() else None)
    except KeyboardInterrupt:
        logger.info("Received shutdown signal")
        sys.exit(0)
    except Exception as e:
        logger.error("Fatal error", error=str(e))
        sys.exit(1)


if __name__ == "__main__":
    main()
