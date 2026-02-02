#!/bin/bash
# ============================================
# Subtitulador - Script de Instalación
# ============================================
# Uso: sudo bash install.sh
# ============================================

set -e

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

INSTALL_DIR="/opt/subtitulador"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Subtitulador - Instalación${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# -------------------------------------------
# 1. Verificar ejecución con sudo
# -------------------------------------------
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}[ERROR] Este script debe ejecutarse con sudo${NC}"
    echo "  Uso: sudo bash install.sh"
    exit 1
fi

if [ -z "$SUDO_USER" ]; then
    echo -e "${YELLOW}[WARN] No se detectó SUDO_USER, usando $(logname)${NC}"
    SUDO_USER=$(logname 2>/dev/null || echo "$USER")
fi

echo -e "${BLUE}[INFO]${NC} Usuario: $SUDO_USER"
echo -e "${BLUE}[INFO]${NC} Directorio de instalación: $INSTALL_DIR"
echo ""

# -------------------------------------------
# 2. Verificar dependencias del sistema
# -------------------------------------------
echo -e "${BLUE}[1/5]${NC} Verificando dependencias del sistema..."

MISSING_DEPS=()

if ! ldconfig -p 2>/dev/null | grep -q "libasound.so.2"; then
    MISSING_DEPS+=("libasound2-dev")
fi

if ! ldconfig -p 2>/dev/null | grep -q "libssl.so.3"; then
    if ! ldconfig -p 2>/dev/null | grep -q "libssl.so"; then
        MISSING_DEPS+=("libssl-dev")
    fi
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo -e "${YELLOW}[WARN] Dependencias faltantes: ${MISSING_DEPS[*]}${NC}"
    echo -e "${BLUE}[INFO]${NC} Instalando dependencias..."
    apt-get update -qq && apt-get install -y -qq "${MISSING_DEPS[@]}"
    echo -e "${GREEN}[OK]${NC}   Dependencias instaladas"
else
    echo -e "${GREEN}[OK]${NC}   Todas las dependencias están presentes"
fi

# -------------------------------------------
# 3. Crear estructura de directorios
# -------------------------------------------
echo -e "${BLUE}[2/5]${NC} Creando directorios..."

mkdir -p "$INSTALL_DIR/bin"
mkdir -p "$INSTALL_DIR/config"
mkdir -p "$INSTALL_DIR/share/applications"
mkdir -p "$INSTALL_DIR/share/icons"

echo -e "${GREEN}[OK]${NC}   Directorios creados"

# -------------------------------------------
# 4. Copiar archivos
# -------------------------------------------
echo -e "${BLUE}[3/5]${NC} Copiando archivos..."

# Binario
cp "$SCRIPT_DIR/bin/subtitulador" "$INSTALL_DIR/bin/subtitulador"
chmod +x "$INSTALL_DIR/bin/subtitulador"
echo -e "       Binario copiado"

# Configuración
cp "$SCRIPT_DIR/config/settings.toml" "$INSTALL_DIR/config/settings.toml"
echo -e "       Configuración copiada"

# Icono
cp "$SCRIPT_DIR/share/icons/subtitulador.svg" "$INSTALL_DIR/share/icons/subtitulador.svg"
echo -e "       Icono copiado"

# Desktop entry
cp "$SCRIPT_DIR/share/applications/subtitulador.desktop" "$INSTALL_DIR/share/applications/subtitulador.desktop"
echo -e "       Desktop entry copiado"

echo -e "${GREEN}[OK]${NC}   Archivos copiados"

# -------------------------------------------
# 5. Ajustar permisos
# -------------------------------------------
echo -e "${BLUE}[4/5]${NC} Ajustando permisos..."

# El directorio bin/ debe ser escribible por el usuario
# para que la app pueda guardar config.json
chown "$SUDO_USER:$SUDO_USER" "$INSTALL_DIR/bin"
echo -e "${GREEN}[OK]${NC}   Permisos ajustados (config.json será escribible)"

# -------------------------------------------
# 6. Registrar entrada de escritorio
# -------------------------------------------
echo -e "${BLUE}[5/5]${NC} Registrando aplicación en el sistema..."

cp "$INSTALL_DIR/share/applications/subtitulador.desktop" \
   /usr/share/applications/subtitulador.desktop

if command -v update-desktop-database &>/dev/null; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi

echo -e "${GREEN}[OK]${NC}   Aplicación registrada en el menú"

# -------------------------------------------
# Resumen
# -------------------------------------------
echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  Instalación completada${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "  Binario:    $INSTALL_DIR/bin/subtitulador"
echo -e "  Config:     $INSTALL_DIR/config/settings.toml"
echo -e "  Icono:      $INSTALL_DIR/share/icons/subtitulador.svg"
echo -e "  Desktop:    /usr/share/applications/subtitulador.desktop"
echo ""
echo -e "  Ejecutar desde terminal:"
echo -e "    ${BLUE}$INSTALL_DIR/bin/subtitulador${NC}"
echo ""
echo -e "  O buscar '${BLUE}Subtitulador${NC}' en el menú de aplicaciones."
echo ""
