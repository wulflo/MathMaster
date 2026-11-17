#!/usr/bin/env bash
# Empaqueta MathMaster para Linux: AppImage y .deb.
#
# Uso:
#   ./scripts/package-desktop.sh [appimage|deb|...]
#
# Sin argumentos genera ambos bundles. Requiere cargo-tauri (tauri-cli 2.x),
# webkit2gtk-4.1 y el `dist/` de la UI Leptos, que se reconstruye aquí.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

if ! command -v cargo-tauri >/dev/null 2>&1; then
  echo "ERROR: falta cargo-tauri. Instalación: cargo install tauri-cli --version '^2'" >&2
  exit 1
fi
if ! pkg-config --exists webkit2gtk-4.1; then
  echo "ERROR: falta webkit2gtk-4.1; el host Tauri no se puede empaquetar." >&2
  exit 1
fi

BUNDLES="${1:-appimage,deb}"

echo "== build web"
./scripts/build-web.sh

echo "== bundle ($BUNDLES)"
cargo tauri build --bundles "$BUNDLES"

echo "== artefactos"
sha256sum target/release/bundle/appimage/*.AppImage target/release/bundle/deb/*.deb 2>/dev/null || true
