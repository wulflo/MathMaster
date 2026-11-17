#!/usr/bin/env bash
# CI local de MathMaster: fmt + construcciones prohibidas + clippy + tests + deps + deny + wasm.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "== fmt"
cargo fmt --all -- --check

echo "== construcciones prohibidas"
./scripts/check-forbidden.sh

echo "== clippy"
# El host Tauri necesita webkit2gtk y el `dist/` embebido. Si faltan (p. ej. en un runner
# sin librerías de escritorio), se excluye y se declara NO VERIFICADO en ese entorno.
DESKTOP_READY=1
if [ ! -f dist/index.html ] || ! pkg-config --exists webkit2gtk-4.1; then
  DESKTOP_READY=0
  echo "AVISO: host Tauri omitido (falta dist/index.html o webkit2gtk-4.1)."
fi
if [ "$DESKTOP_READY" = 1 ]; then
  cargo clippy --workspace --all-targets -- -D warnings
else
  cargo clippy --workspace --all-targets --exclude anzan-desktop -- -D warnings
fi

echo "== tests"
if [ "$DESKTOP_READY" = 1 ]; then
  cargo test --workspace
else
  cargo test --workspace --exclude anzan-desktop
fi

echo "== regla de dependencias"
./scripts/check-deps.sh

echo "== licencias y advisories"
cargo deny check

echo "== check wasm"
TARGET=wasm32-unknown-unknown
SYSROOT="$(rustc --print sysroot)"
if [ -d "$SYSROOT/lib/rustlib/$TARGET" ]; then
  cargo check -p anzan-ui -p anzan-app --target "$TARGET"
elif [ -d /tmp/opencode/sysroot/lib/rustlib/$TARGET ]; then
  RUSTFLAGS="--sysroot=/tmp/opencode/sysroot" \
    cargo check -p anzan-ui -p anzan-app --target "$TARGET"
else
  echo "AVISO: target wasm32 no disponible en este entorno; se omite el check."
fi

echo "CI OK"
