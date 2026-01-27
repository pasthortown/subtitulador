# Arquitectura del Sistema

## Visión General

El sistema de subtitulación en tiempo real está compuesto por tres componentes principales:

1. **Frontend (Rust)** - Captura de audio y visualización
2. **Backend (Python)** - Transcripción con Whisper
3. **Traducción (LibreTranslate)** - Servicio de traducción

## Diagrama de Arquitectura

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND (Rust)                                │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────────────────┐  │
│  │ Captura  │───▶│ Detector │───▶│ Cliente  │───▶│ Ventana Subtítulos   │  │
│  │  Audio   │    │ Silencio │    │   HTTP   │    │ (Transparente)       │  │
│  └──────────┘    └──────────┘    └──────────┘    └──────────────────────┘  │
└───────────────────────────┬─────────────────────────────────────────────────┘
                            │
                            ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                         DOCKER NETWORK (192.168.97.0/24)                  │
│                                                                           │
│  ┌─────────────────────────────┐    ┌─────────────────────────────────┐  │
│  │    BACKEND (Python)         │    │    TRADUCCIÓN (LibreTranslate)  │  │
│  │    192.168.97.10:8000       │    │    192.168.97.11:5000           │  │
│  │                             │    │                                 │  │
│  │  ┌───────────────────────┐  │    │  ┌───────────────────────────┐  │  │
│  │  │   API (Tornado)       │  │    │  │   API REST                │  │  │
│  │  │   /api/v1/transcribe  │  │    │  │   /translate              │  │  │
│  │  └───────────┬───────────┘  │    │  │   /languages              │  │  │
│  │              │              │    │  └───────────────────────────┘  │  │
│  │  ┌───────────▼───────────┐  │    │                                 │  │
│  │  │   Whisper Engine      │  │    │  Volume: ./models/argos        │  │
│  │  │   (CPU Optimizado)    │  │    │                                 │  │
│  │  └───────────────────────┘  │    └─────────────────────────────────┘  │
│  │                             │                                          │
│  │  Volume: ./models/whisper   │                                          │
│  └─────────────────────────────┘                                          │
└───────────────────────────────────────────────────────────────────────────┘
```

## Arquitectura Hexagonal

Tanto el frontend como el backend siguen una arquitectura hexagonal (ports & adapters):

### Capas

```
┌─────────────────────────────────────────────────────────────┐
│                     INFRAESTRUCTURA                         │
│  (Adaptadores: HTTP, Base de datos, APIs externas)         │
├─────────────────────────────────────────────────────────────┤
│                      APLICACIÓN                             │
│  (Casos de uso, DTOs, Orquestación)                        │
├─────────────────────────────────────────────────────────────┤
│                        DOMINIO                              │
│  (Entidades, Value Objects, Servicios, Puertos)            │
└─────────────────────────────────────────────────────────────┘
```

### Backend - Estructura

```
backend/src/
├── domain/                 # Núcleo del negocio
│   ├── entities/          # AudioBuffer, Transcription
│   ├── value_objects/     # Language, AudioFormat
│   ├── services/          # TranscriptionService
│   └── ports/             # Interfaces (SpeechRecognitionPort)
│
├── application/           # Casos de uso
│   ├── use_cases/        # TranscribeAudio, HealthCheck
│   └── dtos/             # Request/Response DTOs
│
└── infrastructure/        # Adaptadores
    ├── adapters/
    │   ├── inbound/      # HTTP Controllers
    │   └── outbound/     # WhisperAdapter
    ├── config/           # Settings, Container DI
    └── web/              # Tornado App, Routes
```

### Frontend - Estructura (DDD)

```
frontend/src/
├── domain/                # Núcleo del negocio
│   ├── entities/         # AudioBuffer, Subtitle
│   ├── value_objects/    # Language, Duration
│   ├── services/         # SilenceDetector, Calibration
│   ├── events/           # Domain Events
│   └── ports/            # Interfaces
│
├── application/          # Casos de uso
│   ├── use_cases/       # StartTranscription, etc.
│   ├── dtos/            # Request/Response
│   └── services/        # Orchestrator
│
├── infrastructure/       # Adaptadores
│   └── adapters/
│       ├── inbound/     # CPAL Audio, egui UI
│       └── outbound/    # HTTP Clients
│
└── presentation/         # UI
    └── ui/              # SubtitleWindow, etc.
```

## Flujo de Datos

1. **Captura de Audio**
   - CPAL captura audio del micrófono
   - Se acumula en buffer hasta detectar silencio

2. **Detección de Silencio**
   - Basada en calibración previa (ruido vs voz)
   - Umbral = ruido + 0.3 × (voz - ruido)
   - Se procesa tras 750ms de silencio

3. **Transcripción**
   - Frontend envía audio base64 al Backend
   - Whisper procesa y retorna texto

4. **Traducción**
   - Si el idioma destino es diferente
   - Se envía a LibreTranslate

5. **Visualización**
   - Texto aparece en ventana transparente
   - Sistema FIFO con fade out

## Red Docker

| Servicio | IP | Puerto |
|----------|-----|--------|
| Backend | 192.168.97.10 | 8000 |
| Translation | 192.168.97.11 | 5000 |
| SonarQube | 192.168.97.20 | 9000 |
| PostgreSQL | 192.168.97.21 | 5432 |

## Volúmenes

Los modelos se almacenan en volúmenes externos para:
- Persistencia entre reinicios
- Facilidad de backup
- Portabilidad a otras máquinas

```
models/
├── whisper/     # Modelos Whisper (~3GB)
└── argos/       # Modelos Argos/LibreTranslate (~300MB)
```
