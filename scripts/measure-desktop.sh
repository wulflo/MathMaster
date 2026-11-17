#!/usr/bin/env bash
# Mide la app de escritorio real (ventana completa) con el arnés del spike B.
#
# Uso:
#   RUNS=3 ./scripts/measure-desktop.sh
#
# Genera measurements/desktop-<sello>-runN.json (crudo por corrida), un resumen
# agregado y un archivo de entorno. Requiere sesión X11 (GDK_BACKEND=x11) y, para
# activar la ventana, xdotool; sin él se mide igual pero WebKit puede limitar rAF.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/.."

RUNS="${RUNS:-3}"
if ! [[ "$RUNS" =~ ^[1-9][0-9]*$ ]]; then
  echo "ERROR: RUNS debe ser un entero positivo." >&2
  exit 1
fi

if [ "${SKIP_BUILD:-0}" != 1 ]; then
  echo "== build web"
  ./scripts/build-web.sh
  echo "== build desktop"
  cargo build -p anzan-desktop --release
fi

BIN=target/release/anzan-desktop
if [ ! -x "$BIN" ]; then
  echo "ERROR: falta $BIN; corré sin SKIP_BUILD=1." >&2
  exit 1
fi

# xdotool no está en el sistema, pero el binario del spike B sigue disponible.
XDOTOOL="${XDOTOOL:-/tmp/opencode/xdotool-3.20160805.1/xdotool}"
if ! command -v xdotool >/dev/null 2>&1; then
  if [ -x "$XDOTOOL" ]; then
    XDOTOOL_DIR="$(dirname "$XDOTOOL")"
  else
    XDOTOOL=""
    XDOTOOL_DIR=""
    echo "AVISO: sin xdotool; la ventana no se activará y rAF puede limitarse." >&2
  fi
else
  XDOTOOL="$(command -v xdotool)"
  XDOTOOL_DIR=""
fi

mkdir -p measurements
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="measurements/desktop-$STAMP"

echo "== entorno"
python3 - "$OUT-env.json" <<'PY'
import hashlib
import json
import os
import pathlib
import subprocess
import sys

def sh(command):
    return subprocess.run(command, shell=True, capture_output=True, text=True).stdout.strip()

def digest(path):
    try:
        return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()
    except OSError:
        return ""

env = {
    "fecha_utc": sh("date -u +%FT%TZ"),
    "host": sh("uname -a"),
    "cpu": sh("grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | sed 's/^ //'"),
    "gpu": sh("lspci 2>/dev/null | grep -Ei 'vga|3d|display' | head -3"),
    "sesion": os.environ.get("XDG_SESSION_TYPE", ""),
    "escritorio": os.environ.get("XDG_CURRENT_DESKTOP", ""),
    "rustc": sh("rustc --version"),
    "cargo_tauri": sh("cargo tauri --version"),
    "webkit2gtk": sh("pkg-config --modversion webkit2gtk-4.1"),
    "gtk3": sh("pkg-config --modversion gtk+-3.0"),
    "audio": sh("pactl info 2>/dev/null | grep -E 'Server Name|Default Sink' | head -2"),
    "xrandr": sh("xrandr --current 2>/dev/null | head -6"),
    "binario_sha256": digest("target/release/anzan-desktop"),
    "wasm_sha256": digest("dist/anzan_app_bg.wasm"),
}
pathlib.Path(sys.argv[1]).write_text(json.dumps(env, indent=2, ensure_ascii=False) + "\n")
print(json.dumps(env, indent=2, ensure_ascii=False))
PY

for i in $(seq 1 "$RUNS"); do
  JSON="$OUT-run$i.json"
  LOG="$OUT-run$i.log"
  echo "== corrida $i/$RUNS"
  attempt=1
  while :; do
    rm -f "$JSON"
    GDK_BACKEND=x11 ANZAN_MEASURE_OUT="$PWD/$JSON" "$BIN" --measure >"$LOG" 2>&1 &
    PID=$!
    sleep 2
    if [ -n "$XDOTOOL" ]; then
      WID="$(LD_LIBRARY_PATH="$XDOTOOL_DIR" "$XDOTOOL" search --name "MathMaster" 2>/dev/null | head -1 || true)"
      if [ -n "$WID" ]; then
        LD_LIBRARY_PATH="$XDOTOOL_DIR" "$XDOTOOL" windowactivate --sync "$WID" >/dev/null 2>&1 || true
        LD_LIBRARY_PATH="$XDOTOOL_DIR" "$XDOTOOL" windowraise "$WID" >/dev/null 2>&1 || true
        echo "   ventana $WID activada"
      else
        echo "   AVISO: no se encontró la ventana" >&2
      fi
    fi
    for _ in $(seq 1 600); do
      kill -0 "$PID" 2>/dev/null || break
      sleep 1
    done
    if kill -0 "$PID" 2>/dev/null; then
      echo "ERROR: la corrida $i excedió 600 s." >&2
      kill "$PID" 2>/dev/null || true
      exit 1
    fi
    if wait "$PID" && [ -s "$JSON" ]; then
      break
    fi
    if [ "$attempt" -ge 3 ]; then
      echo "ERROR: la corrida $i falló 3 veces; ver $LOG" >&2
      exit 1
    fi
    attempt=$((attempt + 1))
    echo "   reintento $attempt/3 de la corrida $i (sin JSON válido)" >&2
    sleep 2
  done
  echo "   $(wc -c <"$JSON") bytes en $JSON"
done

echo "== resumen"
python3 - "$OUT-summary.json" "$OUT"-run*.json <<'PY'
import json
import pathlib
import sys

summary_path = pathlib.Path(sys.argv[1])
files = [pathlib.Path(name) for name in sys.argv[2:]]
runs = []
for path in files:
    try:
        data = json.loads(path.read_text())
    except (OSError, ValueError) as error:
        print(f"ERROR: {path}: {error}")
        raise SystemExit(1)
    if not data.get("ok"):
        print(f"ERROR: {path}: {data.get('error', 'sin ok')}")
        raise SystemExit(1)
    runs.append((path.name, data))

def metricas(data, phase):
    block = data.get(phase) or {}
    return {
        "frame_real_ms": block.get("frame_real_ms"),
        "mean_ms": block.get("mean_ms"),
        "drift_pct": block.get("drift_pct"),
        "err_p50_ms": block.get("err_p50_ms"),
        "err_p95_ms": block.get("err_p95_ms"),
        "err_max_ms": block.get("err_max_ms"),
        "within_one_frame_pct": block.get("within_one_frame_pct"),
        "gaps_count": block.get("gaps_count"),
    }

detalle = []
for name, data in runs:
    cal = data.get("calibration") or {}
    drill = data.get("drill") or {}
    audio = data.get("audioStats") or {}
    detalle.append({
        "archivo": name,
        "has_focus": data.get("hasFocus"),
        "dpr": (data.get("viewport") or {}).get("dpr"),
        "calibracion_span_ms": cal.get("span_ms"),
        "calibracion_mediana_ms": cal.get("median_ms"),
        "drill_telemetry": drill.get("telemetry"),
        "drill_stimulus_ms": drill.get("stimulusMs"),
        "audio_jitter_p95_us": audio.get("jitter_p95_us"),
        "audio_jitter_max_us": audio.get("jitter_max_us"),
        "audio_clicks_started": audio.get("clicks_started"),
        "audio_late_blocks": audio.get("late_blocks"),
        "p100": metricas(data, "presentation_100ms"),
        "p200": metricas(data, "presentation_200ms"),
    })

def peor(clave, fase):
    valores = [fila[fase].get(clave) for fila in detalle if fila[fase].get(clave) is not None]
    return max(valores) if valores else None

agregado = {
    "corridas": len(detalle),
    "n_100ms_total": sum((runs[0][1].get("presentation_100ms") or {}).get("reps", 0) for _ in range(len(detalle))),
    "n_200ms_total": sum((runs[0][1].get("presentation_200ms") or {}).get("reps", 0) for _ in range(len(detalle))),
    "frame_real_100ms_min_ms": min((f["p100"]["frame_real_ms"] for f in detalle if f["p100"].get("frame_real_ms") is not None), default=None),
    "frame_real_100ms_max_ms": peor("frame_real_ms", "p100"),
    "media_100ms_min_ms": min((f["p100"]["mean_ms"] for f in detalle if f["p100"].get("mean_ms") is not None), default=None),
    "media_100ms_max_ms": peor("mean_ms", "p100"),
    "err_p95_100ms_max_ms": peor("err_p95_ms", "p100"),
    "err_p95_200ms_max_ms": peor("err_p95_ms", "p200"),
    "gaps_100ms_total": sum((f["p100"].get("gaps_count") or 0) for f in detalle),
    "gaps_200ms_total": sum((f["p200"].get("gaps_count") or 0) for f in detalle),
    "within_one_frame_100ms_min_pct": min((f["p100"]["within_one_frame_pct"] for f in detalle if f["p100"].get("within_one_frame_pct") is not None), default=None),
    "audio_jitter_p95_us_max": max((f["audio_jitter_p95_us"] or 0) for f in detalle),
    "audio_late_blocks_total": sum((f["audio_late_blocks"] or 0) for f in detalle),
}

summary = {"archivos": [r[0] for r in runs], "agregado": agregado, "detalle": detalle}
summary_path.write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n")

print(f"{'corrida':>28} {'foco':>5} {'frame100':>9} {'media100':>9} {'errP95':>7} {'±1f':>6} {'gaps':>5} {'jitter':>7} {'tarde':>6}")
for fila in detalle:
    p100 = fila["p100"]
    print(
        f"{fila['archivo']:>28} {str(fila['has_focus']):>5} "
        f"{p100.get('frame_real_ms', 0):>9.3f} {p100.get('mean_ms', 0):>9.3f} "
        f"{p100.get('err_p95_ms', 0):>7.2f} {p100.get('within_one_frame_pct', 0):>6.1f} "
        f"{p100.get('gaps_count', 0):>5} {fila['audio_jitter_p95_us'] or 0:>7} "
        f"{fila['audio_late_blocks'] or 0:>6}"
    )
print(json.dumps(agregado, indent=2, ensure_ascii=False))
print(f"resumen: {summary_path}")
PY

echo "MEASURE OK"
