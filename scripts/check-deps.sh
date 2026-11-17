#!/usr/bin/env bash
# Comprueba la regla de dependencias del proyecto:
#   core no depende de nadie interno
#   engine -> core
#   content -> core
#   ui -> core, engine, content
#   app -> core, engine, content, ui
# y que no haya dependencias internas inversas.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

python3 - <<'PY'
import json
import subprocess
import sys

rules = {
    "anzan-core": set(),
    "anzan-engine": {"anzan-core"},
    "anzan-content": {"anzan-core"},
    "anzan-ui": {"anzan-core", "anzan-engine", "anzan-content"},
    "anzan-app": {"anzan-core", "anzan-engine", "anzan-content", "anzan-ui"},
    "anzan-desktop": {"anzan-engine", "anzan-content"},
}

meta = json.loads(
    subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"])
)
packages = {pkg["name"]: pkg for pkg in meta["packages"]}

failures = []
for package, allowed in rules.items():
    if package not in packages:
        failures.append(f"falta el paquete {package}")
        continue
    deps = {
        dep["name"]
        for dep in packages[package]["dependencies"]
        if dep.get("kind") is None and dep["name"] in rules
    }
    if deps != allowed:
        failures.append(
            f"{package}: dependencias internas {sorted(deps)}; se esperaban {sorted(allowed)}"
        )

if failures:
    print("Regla de dependencias VIOLADA:", file=sys.stderr)
    for failure in failures:
        print(f"  - {failure}", file=sys.stderr)
    sys.exit(1)

print("Regla de dependencias OK")
PY
