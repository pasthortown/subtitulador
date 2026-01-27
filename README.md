# Subtitulador en Tiempo Real

Sistema de transcripción y traducción de voz en tiempo real con arquitectura de microservicios.

## Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│                     FRONTEND (Rust + egui)                      │
│           Captura audio → Detecta silencio → Muestra UI         │
└─────────────────────────────┬───────────────────────────────────┘
                              │
        ┌─────────────────────┴─────────────────────┐
        ▼                                           ▼
┌───────────────────────┐                 ┌───────────────────────┐
│  BACKEND (Python)     │                 │  TRADUCCIÓN           │
│  192.168.97.10:8000   │                 │  192.168.97.11:5000   │
│  Tornado + Whisper    │                 │  LibreTranslate       │
└───────────────────────┘                 └───────────────────────┘
```

## Componentes

| Componente | Tecnología | Descripción |
|------------|------------|-------------|
| **Frontend** | Rust + egui | Captura audio, detección silencio, UI subtítulos |
| **Backend** | Python + Tornado | Transcripción con Whisper (CPU) |
| **Traducción** | LibreTranslate | Traducción es/en/pt |
| **Calidad** | SonarQube | Análisis estático y OWASP |

## Requisitos

- Docker y Docker Compose
- Rust 1.70+ (para frontend)
- 8GB+ RAM (para modelo Whisper)

## Instalación

### 1. Descargar modelos

```bash
./scripts/download-models.sh
```

### 2. Configurar variables de entorno

```bash
cp infrastructure/.env.example infrastructure/.env
# Editar según necesidad
```

### 3. Levantar servicios

```bash
cd infrastructure
docker compose up -d
```

### 4. Compilar frontend

```bash
cd frontend
cargo build --release
```

## Uso

### Ejecutar frontend

```bash
cd frontend
cargo run --release
```

O con variables de entorno:

```bash
BACKEND_URL=http://192.168.97.10:8000 \
TRANSLATION_URL=http://192.168.97.11:5000 \
INPUT_LANGUAGE=es \
OUTPUT_LANGUAGE=pt \
cargo run --release
```

### Probar APIs

```bash
# Health check backend
curl http://192.168.97.10:8000/api/v1/health

# Idiomas disponibles traducción
curl http://192.168.97.11:5000/languages
```

### Documentación API

- **Swagger UI**: http://192.168.97.10:8000/docs
- **OpenAPI JSON**: http://192.168.97.10:8000/openapi.json

## Red Docker

| Servicio | IP | Puerto |
|----------|-----|--------|
| Backend | 192.168.97.10 | 8000 |
| Translation | 192.168.97.11 | 5000 |
| SonarQube | 192.168.97.20 | 9000 |

## Scripts

| Script | Descripción |
|--------|-------------|
| `scripts/download-models.sh` | Descarga modelos Whisper |
| `scripts/run-tests.sh` | Ejecuta tests unitarios |
| `scripts/run-newman.sh` | Valida APIs con Postman/Newman |
| `scripts/run-sonar.sh` | Ejecuta análisis SonarQube |

## Estructura del Proyecto

```
subtitulador/
├── backend/                 # Servicio de transcripción (Python)
│   ├── src/
│   │   ├── domain/         # Entidades, Value Objects, Puertos
│   │   ├── application/    # Casos de uso, DTOs
│   │   └── infrastructure/ # Adaptadores, Web, Config
│   ├── tests/
│   └── Dockerfile
│
├── frontend/               # Aplicación de subtítulos (Rust)
│   ├── src/
│   │   ├── domain/        # Entidades, VOs, Servicios (DDD)
│   │   ├── application/   # Orquestador
│   │   ├── infrastructure/# Clientes HTTP, Config
│   │   └── presentation/  # UI egui
│   └── Cargo.toml
│
├── infrastructure/         # Docker y configuración
│   ├── docker-compose.yml
│   ├── docker-compose.sonar.yml
│   └── .env.example
│
├── postman/               # Colecciones para testing
├── scripts/               # Scripts de utilidad
├── models/                # Volúmenes para modelos ML
└── docs/                  # Documentación
```

## Configuración

### Variables de entorno (Frontend)

| Variable | Default | Descripción |
|----------|---------|-------------|
| `BACKEND_URL` | http://192.168.97.10:8000 | URL del backend |
| `TRANSLATION_URL` | http://192.168.97.11:5000 | URL de traducción |
| `INPUT_LANGUAGE` | es | Idioma de entrada |
| `OUTPUT_LANGUAGE` | pt | Idioma de salida |
| `SAMPLE_RATE` | 16000 | Frecuencia de muestreo |

### Variables de entorno (Backend)

| Variable | Default | Descripción |
|----------|---------|-------------|
| `WHISPER_MODEL` | large-v3-turbo | Modelo Whisper |
| `WHISPER_DEVICE` | cpu | Dispositivo (cpu/cuda) |
| `DEFAULT_LANGUAGE` | es | Idioma por defecto |
| `LOG_LEVEL` | INFO | Nivel de logging |

## Testing

### Tests unitarios

```bash
# Backend
cd backend
pip install -r requirements-dev.txt
pytest tests/ -v --cov=src

# Frontend
cd frontend
cargo test
```

### Tests de API (Newman)

```bash
./scripts/run-newman.sh docker
```

### Análisis de calidad

```bash
./scripts/run-sonar.sh
```

## Idiomas Soportados

| Código | Idioma |
|--------|--------|
| es | Español |
| en | Inglés |
| pt | Portugués |
| fr | Francés |
| de | Alemán |
| it | Italiano |

## Licencia

Uso personal/educativo.
