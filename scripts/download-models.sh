#!/bin/bash
# ============================================
# Script para descargar modelos
# ============================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MODELS_DIR="$PROJECT_DIR/models"

echo "============================================"
echo "  Descarga de Modelos"
echo "============================================"

# Crear directorios
mkdir -p "$MODELS_DIR/whisper"
mkdir -p "$MODELS_DIR/argos"

# Descargar modelo Whisper
echo ""
echo "[1/2] Descargando modelo Whisper large-v3-turbo..."
echo "      Esto puede tardar varios minutos..."

cd "$MODELS_DIR/whisper"

if [ ! -f "large-v3-turbo.pt" ]; then
    python3 -c "
import whisper
print('Descargando modelo large-v3-turbo...')
model = whisper.load_model('large-v3-turbo', download_root='.')
print('Modelo descargado correctamente.')
"
else
    echo "      Modelo Whisper ya existe, saltando..."
fi

# Los modelos de Argos/LibreTranslate se descargan automáticamente
# al iniciar el contenedor con los idiomas configurados
echo ""
echo "[2/2] Modelos de traducción (Argos)..."
echo "      Se descargarán automáticamente al iniciar LibreTranslate"
echo "      con los idiomas: es, en, pt"

echo ""
echo "============================================"
echo "  Descarga completada"
echo "============================================"
echo ""
echo "Ubicación de modelos:"
echo "  - Whisper: $MODELS_DIR/whisper/"
echo "  - Argos:   $MODELS_DIR/argos/ (se descargará al iniciar)"
echo ""
