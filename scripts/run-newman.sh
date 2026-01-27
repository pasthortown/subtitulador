#!/bin/bash
# ============================================
# Script para ejecutar tests Newman (Postman)
# ============================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
POSTMAN_DIR="$PROJECT_DIR/postman"
RESULTS_DIR="$PROJECT_DIR/results"

echo "============================================"
echo "  Tests Newman (Postman)"
echo "============================================"

# Crear directorio de resultados
mkdir -p "$RESULTS_DIR"

# Determinar environment
ENV="${1:-docker}"
ENV_FILE="$POSTMAN_DIR/environments/${ENV}.postman_environment.json"

if [ ! -f "$ENV_FILE" ]; then
    echo "Error: Environment '$ENV' no encontrado"
    echo "Uso: $0 [local|docker]"
    exit 1
fi

echo ""
echo "Usando environment: $ENV"
echo ""

# Verificar que newman está instalado
if ! command -v newman &> /dev/null; then
    echo "Newman no está instalado. Instalando..."
    npm install -g newman newman-reporter-htmlextra
fi

# Test Backend
echo "[1/2] Ejecutando tests del Backend..."
echo "----------------------------------------"

newman run "$POSTMAN_DIR/subtitulador-backend.postman_collection.json" \
    -e "$ENV_FILE" \
    --reporters cli,junit,htmlextra \
    --reporter-junit-export "$RESULTS_DIR/backend-results.xml" \
    --reporter-htmlextra-export "$RESULTS_DIR/backend-report.html" \
    --timeout-request 30000 \
    --delay-request 100 \
    || BACKEND_FAILED=1

# Test Translation
echo ""
echo "[2/2] Ejecutando tests de Traducción..."
echo "----------------------------------------"

newman run "$POSTMAN_DIR/subtitulador-translation.postman_collection.json" \
    -e "$ENV_FILE" \
    --reporters cli,junit,htmlextra \
    --reporter-junit-export "$RESULTS_DIR/translation-results.xml" \
    --reporter-htmlextra-export "$RESULTS_DIR/translation-report.html" \
    --timeout-request 30000 \
    --delay-request 100 \
    || TRANSLATION_FAILED=1

# Resumen
echo ""
echo "============================================"
echo "  Resumen de Tests Newman"
echo "============================================"
echo ""

if [ -z "$BACKEND_FAILED" ]; then
    echo "  Backend:     PASS ✓"
else
    echo "  Backend:     FAIL ✗"
fi

if [ -z "$TRANSLATION_FAILED" ]; then
    echo "  Traducción:  PASS ✓"
else
    echo "  Traducción:  FAIL ✗"
fi

echo ""
echo "Reportes generados:"
echo "  - Backend HTML:      results/backend-report.html"
echo "  - Backend JUnit:     results/backend-results.xml"
echo "  - Translation HTML:  results/translation-report.html"
echo "  - Translation JUnit: results/translation-results.xml"
echo ""

# Exit con error si algún test falló
if [ -n "$BACKEND_FAILED" ] || [ -n "$TRANSLATION_FAILED" ]; then
    exit 1
fi
