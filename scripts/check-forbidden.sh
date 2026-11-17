#!/usr/bin/env bash
# Prohibe unwrap/expect/panic/todo/unimplemented/dbg en el código de los crates.
# (En tests también: los fallos se expresan con assert! o Result<(), _>.)
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

PATTERN='(unwrap\(|expect\(|panic!|todo!|unimplemented!|dbg!)'
if grep -RInE --include='*.rs' "$PATTERN" crates; then
  echo "ERROR: se encontraron construcciones prohibidas en crates/ (ver arriba)." >&2
  echo "En producción no hay unwrap/expect/panic; en tests usá assert! o Result<(), _>." >&2
  exit 1
fi
echo "Sin construcciones prohibidas en crates/"
