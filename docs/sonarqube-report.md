# Reporte de Calidad y Seguridad - SonarQube

**Fecha:** 2026-01-27
**Versión SonarQube:** Community Build 26.1.0.118079
**Última ejecución:** 2026-01-27 21:07 UTC

---

## Resumen Ejecutivo

| Componente | Bugs | Vulnerabilidades | Code Smells | Security Hotspots | Duplicación | LOC | Quality Gate |
|------------|------|------------------|-------------|-------------------|-------------|-----|--------------|
| **Backend (Python)** | 0 ✅ | 0 ✅ | 0 ✅ | 0 ✅ | 0.0% ✅ | 1,703 | ✅ PASSED |
| **Frontend (Rust)** | 0 ✅ | 0 ✅ | 0 ✅ | 0 ✅ | 0.0% ✅ | 2,292 | ✅ PASSED |

---

## Backend (Python)

### Métricas de Calidad (SonarQube API)

| Métrica | Valor | Estado |
|---------|-------|--------|
| Bugs | 0 | ✅ bestValue |
| Vulnerabilidades | 0 | ✅ bestValue |
| Code Smells | 0 | ✅ bestValue |
| Security Hotspots | 0 | ✅ bestValue |
| Duplicación | 0.0% | ✅ bestValue |
| Líneas de Código | 1,703 | - |

### Ratings

| Rating | Valor | Descripción |
|--------|-------|-------------|
| Reliability (Bugs) | A | Mejor calificación posible |
| Security (Vulnerabilidades) | A | Mejor calificación posible |
| Maintainability (Code Smells) | A | Mejor calificación posible |

### Issues Históricos (Resueltos)

| Regla | Archivo | Estado | Descripción |
|-------|---------|--------|-------------|
| python:S7503 | `health_check.py:50` | ✅ FIXED | Función async sin características asíncronas |

**Acción tomada:** Se removió el keyword `async` de `get_models()` y el `await` correspondiente en `http_controller.py`.

### Tests Unitarios

```
Total tests: 74
Framework: pytest + pytest-cov

Cobertura por capa:
├── Domain Layer
│   ├── entities/audio_buffer.py       87%
│   ├── entities/transcription.py      92%
│   ├── services/transcription.py     100%
│   ├── value_objects/audio_format.py  98%
│   └── value_objects/language.py      94%
├── Application Layer
│   ├── dtos/transcription_request.py  90%
│   ├── dtos/transcription_response.py 100%
│   └── use_cases/health_check.py     100%
└── Infrastructure Layer                0% (tests de integración pendientes)
```

---

## Frontend (Rust)

### Métricas de Calidad (SonarQube API)

| Métrica | Valor | Estado |
|---------|-------|--------|
| Bugs | 0 | ✅ bestValue |
| Vulnerabilidades | 0 | ✅ bestValue |
| Code Smells | 0 | ✅ bestValue |
| Security Hotspots | 0 | ✅ bestValue |
| Duplicación | 0.0% | ✅ bestValue |
| Líneas de Código | 2,292 | - |

### Ratings

| Rating | Valor | Descripción |
|--------|-------|-------------|
| Reliability (Bugs) | A | Mejor calificación posible |
| Security (Vulnerabilidades) | A | Mejor calificación posible |
| Maintainability (Code Smells) | A | Mejor calificación posible |

### Issues Históricos (Resueltos)

| Regla | Archivo | Complejidad | Estado |
|-------|---------|-------------|--------|
| rust:S3776 | `audio_capture.rs:63` | 30 → 15 | ✅ FIXED |
| rust:S3776 | `audio_capture.rs:129` | 20 → 15 | ✅ FIXED |
| rust:S3776 | `setup_wizard.rs:113` | 19 → N/A | ✅ FIXED (archivo eliminado) |
| rust:S3776 | `subtitle_app.rs:473` | 51 → 15 | ✅ FIXED |
| rust:S3776 | `subtitle_app.rs:791` | 46 → 15 | ✅ FIXED |
| rust:S3776 | `subtitle_app.rs:35` | 46 → 15 | ✅ FIXED |

**Acciones tomadas:**
- Refactorización de funciones con alta complejidad cognitiva
- Extracción de métodos helper
- Eliminación de `setup_wizard.rs` (no utilizado)
- Reducción de duplicación de código

### Tests Unitarios

```
Total tests: 36
Framework: cargo test

Tests por módulo:
├── domain::entities::audio_buffer      3 tests
├── domain::entities::calibration       6 tests
├── domain::entities::subtitle          3 tests
├── domain::services::silence_detector  1 test
├── domain::value_objects::audio_power  9 tests
├── domain::value_objects::duration    10 tests
└── domain::value_objects::language     4 tests
```

---

## Quality Gate

| Componente | Estado | URL |
|------------|--------|-----|
| Backend | ✅ PASSED | http://localhost:9000/dashboard?id=subtitulador-backend |
| Frontend | ✅ PASSED | http://localhost:9000/dashboard?id=subtitulador-frontend |

---

## Refactorizaciones Realizadas

### Backend

| Archivo | Cambio | Impacto |
|---------|--------|---------|
| `health_check.py` | Removido `async` de `get_models()` | Code smell S7503 resuelto |
| `http_controller.py` | Removido `await` de llamada | Consistencia |
| `docs_handler.py` | IP hardcodeada → URL relativa | Security hotspot resuelto |

### Frontend

| Función Original | Complejidad Antes | Complejidad Después |
|------------------|-------------------|---------------------|
| `get_primary_monitor()` | 46 | ~10 |
| `draw_status()` | 51 | ~15 |
| `draw_device_dialog()` | 46 | ~12 |
| `get_pipewire_sources()` | 30 | ~10 |
| `list_input_devices()` | 20 | ~10 |

**Mejoras logradas:**
- Code Smells: 6 → 0
- Duplicación: 3.8% → 0.0%

---

## Comandos de Verificación

### Ejecutar análisis SonarQube

```bash
# Backend
cd backend
docker run --rm --network host \
  -v "$(pwd):/usr/src" -w /usr/src \
  sonarsource/sonar-scanner-cli \
  -Dsonar.projectKey=subtitulador-backend \
  -Dsonar.sources=src \
  -Dsonar.host.url=http://localhost:9000 \
  -Dsonar.token=<TOKEN>

# Frontend
cd frontend
docker run --rm --network host \
  -v "$(pwd):/usr/src" -w /usr/src \
  sonarsource/sonar-scanner-cli \
  -Dsonar.projectKey=subtitulador-frontend \
  -Dsonar.sources=src \
  -Dsonar.host.url=http://localhost:9000 \
  -Dsonar.token=<TOKEN>
```

### Ejecutar tests

```bash
# Backend (Python)
cd backend
source venv/bin/activate
python -m pytest tests/ -v --cov=src

# Frontend (Rust)
cd frontend
cargo test
```

### Consultar métricas via API

```bash
# Backend
curl -u admin:admin "http://localhost:9000/api/measures/component?component=subtitulador-backend&metricKeys=bugs,vulnerabilities,code_smells,security_hotspots,duplicated_lines_density,ncloc"

# Frontend
curl -u admin:admin "http://localhost:9000/api/measures/component?component=subtitulador-frontend&metricKeys=bugs,vulnerabilities,code_smells,security_hotspots,duplicated_lines_density,ncloc"
```

---

## Próximos Pasos

- [x] Resolver code smells detectados
- [x] Resolver security hotspots
- [x] Implementar tests unitarios backend (74 tests)
- [x] Implementar tests unitarios frontend (36 tests)
- [ ] Aumentar cobertura backend a 80%
- [ ] Agregar tests de integración para Infrastructure layer
- [ ] Configurar CI/CD con análisis automático

---

*Reporte generado desde SonarQube Community Build 26.1.0.118079*
*Servidor: http://localhost:9000*
