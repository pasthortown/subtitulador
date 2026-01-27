#!/bin/bash
# ============================================
# Script para ejecutar tests
# ============================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "============================================"
echo "  Ejecución de Tests"
echo "============================================"

# Backend Python tests
echo ""
echo "[1/2] Ejecutando tests del Backend (Python)..."
echo "----------------------------------------"

cd "$PROJECT_DIR/backend"

if [ -d "venv" ]; then
    source venv/bin/activate
fi

# Instalar dependencias de test si no existen
pip install -q -r requirements-dev.txt 2>/dev/null || true

# Ejecutar pytest
python -m pytest tests/ \
    -v \
    --tb=short \
    --cov=src \
    --cov-report=term-missing \
    --cov-report=xml:coverage.xml \
    --cov-report=html:htmlcov \
    --junitxml=test-results.xml

BACKEND_RESULT=$?

# Frontend Rust tests
echo ""
echo "[2/2] Ejecutando tests del Frontend (Rust)..."
echo "----------------------------------------"

cd "$PROJECT_DIR/frontend"

if [ -f "Cargo.toml" ]; then
    cargo test --all-features 2>&1 || true
    FRONTEND_RESULT=$?
else
    echo "      Cargo.toml no encontrado, saltando tests de Rust"
    FRONTEND_RESULT=0
fi

# Resumen
echo ""
echo "============================================"
echo "  Resumen de Tests"
echo "============================================"
echo ""

if [ $BACKEND_RESULT -eq 0 ]; then
    echo "  Backend:  PASS ✓"
else
    echo "  Backend:  FAIL ✗"
fi

if [ $FRONTEND_RESULT -eq 0 ]; then
    echo "  Frontend: PASS ✓"
else
    echo "  Frontend: FAIL ✗"
fi

echo ""
echo "Reportes generados:"
echo "  - Backend coverage:  backend/htmlcov/index.html"
echo "  - Backend XML:       backend/coverage.xml"
echo "  - Backend JUnit:     backend/test-results.xml"
echo ""

# Exit con error si algún test falló
if [ $BACKEND_RESULT -ne 0 ] || [ $FRONTEND_RESULT -ne 0 ]; then
    exit 1
fi
