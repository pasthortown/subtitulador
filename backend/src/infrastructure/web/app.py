"""Aplicación web Tornado."""

import tornado.web
import tornado.ioloop
from pathlib import Path
from typing import Optional

from .routes import get_routes
from .handlers.docs_handler import OpenAPIHandler, SwaggerUIHandler
from ..config.container import Container, initialize_container


class Application(tornado.web.Application):
    """Aplicación Tornado con inyección de dependencias."""

    def __init__(self, container: Container):
        self.container = container

        # Obtener rutas con dependencias inyectadas
        routes = get_routes(container)

        # Añadir rutas de documentación
        routes.extend([
            (r"/docs", SwaggerUIHandler),
            (r"/openapi.json", OpenAPIHandler, {"container": container}),
        ])

        settings = {
            "debug": container.settings.server.debug,
            "autoreload": container.settings.server.debug,
        }

        super().__init__(routes, **settings)


def create_app(config_path: Optional[Path] = None) -> Application:
    """
    Crea y configura la aplicación.

    Args:
        config_path: Ruta opcional al archivo de configuración.

    Returns:
        Aplicación Tornado configurada.
    """
    # Inicializar contenedor con todas las dependencias
    container = initialize_container(config_path)

    # Crear aplicación
    app = Application(container)

    return app


def run_server(config_path: Optional[Path] = None) -> None:
    """
    Ejecuta el servidor.

    Args:
        config_path: Ruta opcional al archivo de configuración.
    """
    app = create_app(config_path)
    settings = app.container.settings

    print(f"Starting server on {settings.server.host}:{settings.server.port}")
    print(f"API docs available at http://{settings.server.host}:{settings.server.port}/docs")

    app.listen(
        settings.server.port,
        address=settings.server.host
    )

    try:
        tornado.ioloop.IOLoop.current().start()
    except KeyboardInterrupt:
        print("\nShutting down...")
        app.container.shutdown()
        tornado.ioloop.IOLoop.current().stop()
