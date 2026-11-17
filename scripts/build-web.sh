#!/usr/bin/env bash
# Compila la app web y prepara el sitio estático en dist/.
#
# Uso:
#   ./scripts/build-web.sh
#
# Requisitos: cargo, wasm32-unknown-unknown y wasm-bindgen 0.2.130.
# En esta máquina no hay rustup ni target wasm32; el script recurre al sysroot local
# de /tmp/opencode/sysroot si hace falta (ver spikes/README.md).
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

TARGET=wasm32-unknown-unknown
PROFILE=release
SYSROOT="$(rustc --print sysroot)"

if [ ! -d "$SYSROOT/lib/rustlib/$TARGET" ] && [ -d /tmp/opencode/sysroot/lib/rustlib/$TARGET ]; then
  export RUSTFLAGS="${RUSTFLAGS:-} --sysroot=/tmp/opencode/sysroot"
fi

echo "== build wasm"
cargo build -p anzan-app --target "$TARGET" --profile "$PROFILE"

rm -rf dist
mkdir -p dist

WASM_BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
if ! command -v "$WASM_BINDGEN" >/dev/null 2>&1; then
  WASM_BINDGEN=/tmp/opencode/wb/wasm-bindgen-0.2.130-x86_64-unknown-linux-musl/wasm-bindgen
fi

echo "== wasm-bindgen"
"$WASM_BINDGEN" "target/$TARGET/$PROFILE/anzan_app.wasm" --target web --out-dir dist --out-name anzan_app

cp web/index.html dist/index.html
cp web/manifest.webmanifest dist/manifest.webmanifest
cp web/service-worker.js dist/service-worker.js
cp web/icon.svg dist/icon.svg
echo "== dist listo"
ls -la dist
