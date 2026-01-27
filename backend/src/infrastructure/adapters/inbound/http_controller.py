"""Adapter: HTTPController - Controlador HTTP para la API REST."""

import json
from typing import Any

import tornado.web
from pydantic import ValidationError

from ....application import (
    TranscribeAudioUseCase,
    HealthCheckUseCase,
    TranscriptionRequest,
    TranscriptionResponse
)


class BaseHandler(tornado.web.RequestHandler):
    """Handler base con utilidades comunes."""

    def set_default_headers(self):
        """Configura headers CORS."""
        self.set_header("Access-Control-Allow-Origin", "*")
        self.set_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.set_header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        self.set_header("Content-Type", "application/json")

    def options(self, *args, **kwargs):
        """Maneja preflight CORS."""
        self.set_status(204)
        self.finish()

    def write_json(self, data: Any, status: int = 200):
        """Escribe respuesta JSON."""
        self.set_status(status)
        if hasattr(data, 'model_dump'):
            self.write(data.model_dump_json())
        elif hasattr(data, 'dict'):
            self.write(json.dumps(data.dict()))
        else:
            self.write(json.dumps(data))

    def write_error_json(self, code: str, message: str, status: int = 400):
        """Escribe error JSON."""
        response = TranscriptionResponse.failure(code=code, message=message)
        self.write_json(response, status)


class TranscriptionHandler(BaseHandler):
    """Handler para endpoints de transcripción."""

    def initialize(self, transcribe_use_case: TranscribeAudioUseCase):
        self._transcribe_use_case = transcribe_use_case

    async def post(self):
        """
        POST /api/v1/transcribe

        Transcribe audio enviado en base64.
        """
        try:
            # Parsear request
            body = json.loads(self.request.body)
            request = TranscriptionRequest(**body)

            # Ejecutar caso de uso
            response = await self._transcribe_use_case.execute(request)

            # Retornar respuesta
            status = 200 if response.success else 400
            self.write_json(response, status)

        except json.JSONDecodeError:
            self.write_error_json(
                code="INVALID_JSON",
                message="Invalid JSON in request body",
                status=400
            )
        except ValidationError as e:
            self.write_error_json(
                code="VALIDATION_ERROR",
                message=str(e),
                status=400
            )
        except Exception as e:
            self.write_error_json(
                code="INTERNAL_ERROR",
                message=str(e),
                status=500
            )


class TranscriptionStreamHandler(BaseHandler):
    """Handler para transcripción en streaming."""

    def initialize(self, transcribe_use_case: TranscribeAudioUseCase):
        self._transcribe_use_case = transcribe_use_case

    async def post(self):
        """
        POST /api/v1/transcribe/stream

        Transcribe audio en streaming usando Server-Sent Events.
        """
        self.set_header("Content-Type", "text/event-stream")
        self.set_header("Cache-Control", "no-cache")
        self.set_header("Connection", "keep-alive")

        try:
            # Obtener parámetros
            language = self.get_argument("language", "es")

            # Generar eventos SSE
            async def audio_generator():
                # En una implementación real, esto leería del body del request
                # Por ahora, esto es un placeholder
                while True:
                    chunk = await self.request.connection.read_bytes(
                        4096,
                        partial=True
                    )
                    if not chunk:
                        break
                    yield chunk

            async for response in self._transcribe_use_case.execute_stream(
                audio_generator(),
                language
            ):
                event_data = f"data: {response.model_dump_json()}\n\n"
                self.write(event_data)
                await self.flush()

            self.write("event: done\ndata: {}\n\n")
            await self.flush()

        except Exception as e:
            error_event = f"event: error\ndata: {json.dumps({'error': str(e)})}\n\n"
            self.write(error_event)
            await self.flush()


class HealthHandler(BaseHandler):
    """Handler para health check."""

    def initialize(self, health_use_case: HealthCheckUseCase):
        self._health_use_case = health_use_case

    async def get(self):
        """
        GET /api/v1/health

        Retorna el estado del servicio.
        """
        try:
            response = await self._health_use_case.execute()
            status = 200 if response.status == "healthy" else 503
            self.write_json(response, status)
        except Exception as e:
            self.write_error_json(
                code="HEALTH_CHECK_FAILED",
                message=str(e),
                status=503
            )


class ModelsHandler(BaseHandler):
    """Handler para información de modelos."""

    def initialize(self, health_use_case: HealthCheckUseCase):
        self._health_use_case = health_use_case

    async def get(self):
        """
        GET /api/v1/models

        Retorna información de los modelos disponibles.
        """
        try:
            response = self._health_use_case.get_models()
            self.write_json(response)
        except Exception as e:
            self.write_error_json(
                code="MODELS_ERROR",
                message=str(e),
                status=500
            )
