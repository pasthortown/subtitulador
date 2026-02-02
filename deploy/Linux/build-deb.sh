#!/bin/bash
# ============================================
# Subtitulador - Generador de paquete .deb
# ============================================
# Uso: bash build-deb.sh
# ============================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKG_NAME="subtitulador-frontend"
VERSION="1.0.0"
ARCH="amd64"
PKG_DIR="${SCRIPT_DIR}/${PKG_NAME}_${VERSION}_${ARCH}"
DEB_FILE="${SCRIPT_DIR}/${PKG_NAME}_${VERSION}_${ARCH}.deb"

# Colores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}  Subtitulador - Build .deb${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# -------------------------------------------
# 1. Verificar que existen los archivos fuente
# -------------------------------------------
echo -e "${BLUE}[1/4]${NC} Verificando archivos fuente..."

for f in \
    "$SCRIPT_DIR/bin/subtitulador" \
    "$SCRIPT_DIR/config/settings.toml" \
    "$SCRIPT_DIR/share/applications/subtitulador.desktop" \
    "$SCRIPT_DIR/share/icons/subtitulador.svg"; do
    if [ ! -f "$f" ]; then
        echo -e "${RED}[ERROR] No se encontró: $f${NC}"
        exit 1
    fi
done

echo -e "${GREEN}[OK]${NC}   Archivos fuente verificados"

# -------------------------------------------
# 2. Crear estructura del paquete
# -------------------------------------------
echo -e "${BLUE}[2/4]${NC} Creando estructura del paquete..."

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/opt/subtitulador/bin"
mkdir -p "$PKG_DIR/opt/subtitulador/config"
mkdir -p "$PKG_DIR/opt/subtitulador/share/applications"
mkdir -p "$PKG_DIR/opt/subtitulador/share/icons"
mkdir -p "$PKG_DIR/usr/share/applications"

# Copiar archivos
cp "$SCRIPT_DIR/bin/subtitulador"                          "$PKG_DIR/opt/subtitulador/bin/"
cp "$SCRIPT_DIR/config/settings.toml"                      "$PKG_DIR/opt/subtitulador/config/"
cp "$SCRIPT_DIR/share/icons/subtitulador.svg"              "$PKG_DIR/opt/subtitulador/share/icons/"
cp "$SCRIPT_DIR/share/applications/subtitulador.desktop"   "$PKG_DIR/opt/subtitulador/share/applications/"
cp "$SCRIPT_DIR/share/applications/subtitulador.desktop"   "$PKG_DIR/usr/share/applications/"

# Permisos de archivos
chmod 755 "$PKG_DIR/opt/subtitulador/bin/subtitulador"
chmod 644 "$PKG_DIR/opt/subtitulador/config/settings.toml"
chmod 644 "$PKG_DIR/opt/subtitulador/share/icons/subtitulador.svg"
chmod 644 "$PKG_DIR/opt/subtitulador/share/applications/subtitulador.desktop"
chmod 644 "$PKG_DIR/usr/share/applications/subtitulador.desktop"

echo -e "${GREEN}[OK]${NC}   Estructura creada"

# -------------------------------------------
# 3. Generar archivos DEBIAN
# -------------------------------------------
echo -e "${BLUE}[3/4]${NC} Generando metadatos del paquete..."

# Calcular tamaño instalado (en KB)
INSTALLED_SIZE=$(du -sk "$PKG_DIR" | cut -f1)

# DEBIAN/control
cat > "$PKG_DIR/DEBIAN/control" << EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libasound2 (>= 1.2), libssl3 (>= 3.0)
Installed-Size: ${INSTALLED_SIZE}
Maintainer: Subtitulador Team
Description: Subtitulador en Tiempo Real
 Aplicación de transcripción y traducción de audio en tiempo real
 utilizando Whisper y LibreTranslate. Interfaz gráfica nativa
 construida con egui/eframe.
Homepage: https://github.com/subtitulador
EOF

# DEBIAN/postinst
cat > "$PKG_DIR/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e

# Permitir que el usuario que ejecuta la app pueda escribir config.json
# Se otorga escritura al grupo para cualquier usuario del sistema
chmod 777 /opt/subtitulador/bin

# Actualizar base de datos de aplicaciones de escritorio
if command -v update-desktop-database &>/dev/null; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi

echo "Subtitulador instalado en /opt/subtitulador/"
echo "Ejecutar: /opt/subtitulador/bin/subtitulador"
EOF
chmod 755 "$PKG_DIR/DEBIAN/postinst"

# DEBIAN/postrm
cat > "$PKG_DIR/DEBIAN/postrm" << 'EOF'
#!/bin/bash
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    # Limpiar config.json generado en runtime
    rm -f /opt/subtitulador/bin/config.json

    # Limpiar directorios vacíos
    rmdir /opt/subtitulador/bin 2>/dev/null || true
    rmdir /opt/subtitulador/config 2>/dev/null || true
    rmdir /opt/subtitulador/share/applications 2>/dev/null || true
    rmdir /opt/subtitulador/share/icons 2>/dev/null || true
    rmdir /opt/subtitulador/share 2>/dev/null || true
    rmdir /opt/subtitulador 2>/dev/null || true

    # Actualizar base de datos de aplicaciones
    if command -v update-desktop-database &>/dev/null; then
        update-desktop-database /usr/share/applications 2>/dev/null || true
    fi
fi
EOF
chmod 755 "$PKG_DIR/DEBIAN/postrm"

echo -e "${GREEN}[OK]${NC}   Metadatos generados"

# -------------------------------------------
# 4. Construir el .deb
# -------------------------------------------
echo -e "${BLUE}[4/4]${NC} Construyendo paquete .deb..."

dpkg-deb --build --root-owner-group "$PKG_DIR" "$DEB_FILE"

# Limpiar directorio temporal
rm -rf "$PKG_DIR"

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  Paquete generado exitosamente${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "  Archivo: ${BLUE}${DEB_FILE}${NC}"
echo ""
echo -e "  Instalar:"
echo -e "    sudo dpkg -i ${PKG_NAME}_${VERSION}_${ARCH}.deb"
echo ""
echo -e "  Desinstalar:"
echo -e "    sudo dpkg -r ${PKG_NAME}"
echo ""
