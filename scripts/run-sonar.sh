#!/bin/bash
# ============================================
# Script para análisis SonarQube
# ============================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
INFRA_DIR="$PROJECT_DIR/infrastructure"

echo "============================================"
echo "  Análisis SonarQube"
echo "============================================"

# Verificar que SonarQube está corriendo
echo ""
echo "[1/4] Verificando SonarQube..."

SONAR_URL="${SONAR_URL:-http://localhost:9000}"
SONAR_TOKEN="${SONAR_TOKEN:-}"

# Intentar conectar a SonarQube
if ! curl -s "$SONAR_URL/api/system/status" | grep -q "UP"; then
    echo "      SonarQube no está disponible en $SONAR_URL"
    echo "      Iniciando SonarQube..."

    cd "$INFRA_DIR"
    docker compose -f docker-compose.yml up -d
    docker compose -f docker-compose.sonar.yml up -d

    echo "      Esperando a que SonarQube inicie (puede tardar ~2 minutos)..."

    for i in {1..60}; do
        if curl -s "$SONAR_URL/api/system/status" | grep -q "UP"; then
            echo "      SonarQube está listo!"
            break
        fi
        sleep 5
        echo "      Esperando... ($i/60)"
    done
fi

# Ejecutar tests primero para generar coverage
echo ""
echo "[2/4] Ejecutando tests para generar coverage..."
"$SCRIPT_DIR/run-tests.sh" || true

# Análisis del Backend
echo ""
echo "[3/4] Analizando Backend (Python)..."
echo "----------------------------------------"

cd "$PROJECT_DIR/backend"

# Generar reporte de pylint
echo "      Generando reporte pylint..."
pip install pylint 2>/dev/null || true
pylint src/ --output-format=text > pylint-report.txt 2>&1 || true

# Generar reporte de bandit (seguridad)
echo "      Generando reporte bandit (seguridad)..."
pip install bandit 2>/dev/null || true
bandit -r src/ -f json -o bandit-report.json 2>/dev/null || true

# Ejecutar scanner
if [ -n "$SONAR_TOKEN" ]; then
    docker run --rm \
        --network infrastructure_subtitulador_net \
        -v "$PROJECT_DIR/backend:/usr/src" \
        -w /usr/src \
        sonarsource/sonar-scanner-cli \
        -Dsonar.host.url="$SONAR_URL" \
        -Dsonar.token="$SONAR_TOKEN" \
        -Dsonar.projectKey=subtitulador-backend
else
    echo "      SONAR_TOKEN no configurado. Configure con:"
    echo "      export SONAR_TOKEN=<su-token>"
    echo "      Saltando análisis..."
fi

# Análisis del Frontend
echo ""
echo "[4/4] Analizando Frontend (Rust)..."
echo "----------------------------------------"

cd "$PROJECT_DIR/frontend"

if [ -f "Cargo.toml" ] && [ -n "$SONAR_TOKEN" ]; then
    docker run --rm \
        --network infrastructure_subtitulador_net \
        -v "$PROJECT_DIR/frontend:/usr/src" \
        -w /usr/src \
        sonarsource/sonar-scanner-cli \
        -Dsonar.host.url="$SONAR_URL" \
        -Dsonar.token="$SONAR_TOKEN" \
        -Dsonar.projectKey=subtitulador-frontend
else
    echo "      Frontend no configurado o SONAR_TOKEN no disponible"
fi

echo ""
echo "============================================"
echo "  Análisis Completado"
echo "============================================"
echo ""
echo "Ver resultados en: $SONAR_URL"
echo ""
echo "Proyectos:"
echo "  - subtitulador-backend"
echo "  - subtitulador-frontend"
echo ""
