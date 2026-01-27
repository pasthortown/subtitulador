"""Handlers para documentación OpenAPI/Swagger."""

import json
import tornado.web

from ...config.container import Container
from ..constants import CONTENT_TYPE_JSON

OPENAPI_SPEC = {
    "openapi": "3.0.3",
    "info": {
        "title": "Subtitulador Backend API",
        "description": "API de transcripción de audio en tiempo real usando Whisper",
        "version": "1.0.0",
        "contact": {
            "name": "Subtitulador Team"
        }
    },
    "servers": [
        {
            "url": "/",
            "description": "Current server (auto-detected)"
        }
    ],
    "paths": {
        "/api/v1/health": {
            "get": {
                "summary": "Health Check",
                "description": "Verifica el estado del servicio",
                "operationId": "healthCheck",
                "tags": ["Health"],
                "responses": {
                    "200": {
                        "description": "Servicio saludable",
                        "content": {
                            CONTENT_TYPE_JSON: {
                                "schema": {"$ref": "#/components/schemas/HealthResponse"}
                            }
                        }
                    },
                    "503": {
                        "description": "Servicio no disponible"
                    }
                }
            }
        },
        "/api/v1/models": {
            "get": {
                "summary": "List Models",
                "description": "Lista los modelos de transcripción disponibles",
                "operationId": "listModels",
                "tags": ["Models"],
                "responses": {
                    "200": {
                        "description": "Lista de modelos",
                        "content": {
                            CONTENT_TYPE_JSON: {
                                "schema": {"$ref": "#/components/schemas/ModelsResponse"}
                            }
                        }
                    }
                }
            }
        },
        "/api/v1/transcribe": {
            "post": {
                "summary": "Transcribe Audio",
                "description": "Transcribe audio enviado en base64",
                "operationId": "transcribeAudio",
                "tags": ["Transcription"],
                "requestBody": {
                    "required": True,
                    "content": {
                        CONTENT_TYPE_JSON: {
                            "schema": {"$ref": "#/components/schemas/TranscriptionRequest"}
                        }
                    }
                },
                "responses": {
                    "200": {
                        "description": "Transcripción exitosa",
                        "content": {
                            CONTENT_TYPE_JSON: {
                                "schema": {"$ref": "#/components/schemas/TranscriptionResponse"}
                            }
                        }
                    },
                    "400": {
                        "description": "Error de validación",
                        "content": {
                            CONTENT_TYPE_JSON: {
                                "schema": {"$ref": "#/components/schemas/TranscriptionResponse"}
                            }
                        }
                    },
                    "500": {
                        "description": "Error interno"
                    }
                }
            }
        },
        "/api/v1/transcribe/stream": {
            "post": {
                "summary": "Transcribe Audio Stream",
                "description": "Transcribe audio en streaming usando Server-Sent Events",
                "operationId": "transcribeAudioStream",
                "tags": ["Transcription"],
                "parameters": [
                    {
                        "name": "language",
                        "in": "query",
                        "schema": {"type": "string", "default": "es"},
                        "description": "Código de idioma ISO 639-1"
                    }
                ],
                "responses": {
                    "200": {
                        "description": "Stream de transcripciones",
                        "content": {
                            "text/event-stream": {
                                "schema": {"type": "string"}
                            }
                        }
                    }
                }
            }
        }
    },
    "components": {
        "schemas": {
            "TranscriptionRequest": {
                "type": "object",
                "required": ["audio"],
                "properties": {
                    "audio": {
                        "type": "string",
                        "description": "Audio en formato base64 (WAV, PCM float32, 16kHz mono)"
                    },
                    "language": {
                        "type": "string",
                        "default": "es",
                        "description": "Código de idioma ISO 639-1",
                        "enum": ["es", "en", "pt", "fr", "de", "it", "ja", "ko", "zh", "ru"]
                    },
                    "sample_rate": {
                        "type": "integer",
                        "default": 16000,
                        "description": "Frecuencia de muestreo del audio"
                    }
                }
            },
            "TranscriptionResponse": {
                "type": "object",
                "properties": {
                    "success": {
                        "type": "boolean",
                        "description": "Indica si la operación fue exitosa"
                    },
                    "data": {
                        "$ref": "#/components/schemas/TranscriptionData"
                    },
                    "error": {
                        "$ref": "#/components/schemas/ErrorData"
                    },
                    "timestamp": {
                        "type": "string",
                        "format": "date-time"
                    }
                }
            },
            "TranscriptionData": {
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Texto transcrito"
                    },
                    "language": {
                        "type": "string",
                        "description": "Código de idioma"
                    },
                    "confidence": {
                        "type": "number",
                        "minimum": 0,
                        "maximum": 1,
                        "description": "Nivel de confianza"
                    },
                    "duration_ms": {
                        "type": "integer",
                        "description": "Duración del audio en ms"
                    },
                    "processing_time_ms": {
                        "type": "integer",
                        "description": "Tiempo de procesamiento en ms"
                    }
                }
            },
            "ErrorData": {
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Código de error"
                    },
                    "message": {
                        "type": "string",
                        "description": "Mensaje de error"
                    },
                    "details": {
                        "type": "object",
                        "description": "Detalles adicionales"
                    }
                }
            },
            "HealthResponse": {
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["healthy", "unhealthy"]
                    },
                    "version": {
                        "type": "string"
                    },
                    "model": {
                        "type": "string"
                    },
                    "device": {
                        "type": "string"
                    },
                    "uptime_seconds": {
                        "type": "number"
                    }
                }
            },
            "ModelsResponse": {
                "type": "object",
                "properties": {
                    "models": {
                        "type": "array",
                        "items": {
                            "type": "object"
                        }
                    },
                    "current_model": {
                        "type": "string"
                    }
                }
            }
        }
    },
    "tags": [
        {"name": "Health", "description": "Endpoints de estado del servicio"},
        {"name": "Models", "description": "Información de modelos"},
        {"name": "Transcription", "description": "Endpoints de transcripción"}
    ]
}


SWAGGER_UI_HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Subtitulador API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {
            SwaggerUIBundle({
                url: "/openapi.json",
                dom_id: '#swagger-ui',
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
                layout: "BaseLayout"
            });
        };
    </script>
</body>
</html>
"""


class OpenAPIHandler(tornado.web.RequestHandler):
    """Handler para servir la especificación OpenAPI."""

    def initialize(self, container: Container):
        self.container = container

    def set_default_headers(self):
        self.set_header("Content-Type", CONTENT_TYPE_JSON)
        self.set_header("Access-Control-Allow-Origin", "*")

    def get(self):
        """GET /openapi.json"""
        self.write(json.dumps(OPENAPI_SPEC))


class SwaggerUIHandler(tornado.web.RequestHandler):
    """Handler para servir Swagger UI."""

    def get(self):
        """GET /docs"""
        self.set_header("Content-Type", "text/html")
        self.write(SWAGGER_UI_HTML)
