#!/usr/bin/env bash
# =============================================================================
# AULA F108 Pro GUI - Standalone Linux Build & Packaging Script
# =============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=========================================================="
echo " Building AULA F108 Pro Linux Standalone GUI..."
echo "=========================================================="

# Check Python and PyGObject
python3 -c "import gi; gi.require_version('Gtk', '3.0'); gi.require_version('WebKit2', '4.1'); from gi.repository import Gtk, WebKit2" || {
    echo "[ERROR] Missing PyGObject or WebKitGTK 4.1. Please ensure gir1.2-webkit2-4.1 is installed."
    exit 1
}

# Install PyInstaller if not found
if ! command -v pyinstaller &> /dev/null && [ ! -f "$HOME/.local/bin/pyinstaller" ]; then
    echo "[INFO] Installing PyInstaller in user environment..."
    python3 -m pip install --user --break-system-packages pyinstaller
fi

PYINSTALLER="${PYINSTALLER:-$(command -v pyinstaller || echo "$HOME/.local/bin/pyinstaller")}"

echo "[INFO] Running PyInstaller..."
"$PYINSTALLER"     --name f108-pro-gui     --noconfirm     --clean     --onedir     --add-data "ui:ui"     --add-data "assets:assets"     --collect-all gi     --collect-all PIL     --hidden-import gi.repository.Gtk     --hidden-import gi.repository.WebKit2     --hidden-import gi.repository.JavaScriptCore     app.py

# Create symlink in ~/.local/bin
mkdir -p "$HOME/.local/bin"
ln -sf "$SCRIPT_DIR/dist/f108-pro-gui/f108-pro-gui" "$HOME/.local/bin/f108-pro-gui"

# Install Desktop file
mkdir -p "$HOME/.local/share/applications"
cp "$SCRIPT_DIR/f108-pro-gui.desktop" "$HOME/.local/share/applications/"
update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

echo "=========================================================="
echo " Build successful!"
echo " Binary created: $SCRIPT_DIR/dist/f108-pro-gui/f108-pro-gui"
echo " Installed to:   $HOME/.local/bin/f108-pro-gui"
echo " Desktop entry:  $HOME/.local/share/applications/f108-pro-gui.desktop"
echo "=========================================================="
