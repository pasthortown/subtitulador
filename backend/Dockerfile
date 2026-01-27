# ============================================
# BACKEND - Servicio de Transcripción Whisper
# Con soporte GPU (CUDA) y CPU
# ============================================

# ============================================
# Stage 1: Base con CUDA y dependencias
# ============================================
FROM nvidia/cuda:12.6.3-runtime-ubuntu22.04 AS base

# Evitar prompts interactivos
ENV DEBIAN_FRONTEND=noninteractive

# Instalar Python y dependencias del sistema
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3.11 \
    python3.11-venv \
    python3.11-dev \
    python3-pip \
    ffmpeg \
    libsndfile1 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf /usr/bin/python3.11 /usr/bin/python \
    && ln -sf /usr/bin/python3.11 /usr/bin/python3

# Crear usuario no-root
RUN useradd --create-home --shell /bin/bash appuser

# Directorio de trabajo
WORKDIR /app

# ============================================
# Stage 2: Builder - Instalar dependencias Python
# ============================================
FROM base AS builder

# Instalar dependencias de compilación
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copiar requirements
COPY requirements.txt .

# Crear virtualenv e instalar dependencias
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Instalar PyTorch con CUDA primero
RUN pip install --no-cache-dir --upgrade pip && \
    pip install --no-cache-dir torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu124 && \
    pip install --no-cache-dir -r requirements.txt

# ============================================
# Stage 3: Production - Imagen final optimizada
# ============================================
FROM base AS production

# Copiar virtualenv del builder
COPY --from=builder /opt/venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"

# Copiar solo el código necesario
COPY --chown=appuser:appuser src/ /app/src/
COPY --chown=appuser:appuser config/ /app/config/

# Crear directorio para modelos
RUN mkdir -p /app/models && chown appuser:appuser /app/models

# Cambiar a usuario no-root
USER appuser

# Puerto de la aplicación
EXPOSE 8000

# Variables de entorno
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=120s --retries=3 \
    CMD curl -f http://localhost:8000/api/v1/health || exit 1

# Comando de producción
CMD ["python", "-m", "src.main"]
