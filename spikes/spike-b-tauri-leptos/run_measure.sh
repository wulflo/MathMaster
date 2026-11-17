#!/usr/bin/env bash
# Ejecuta el spike B completo en X11 (XWayland) y activa la ventana para evitar
# el throttling de rAF que WebKit aplica a ventanas sin foco.
set -uo pipefail
cd "$(dirname "$0")"
BIN=./target/release/spike-b-tauri
XD=/tmp/opencode/xdotool-3.20160805.1
LOG=/tmp/opencode/spike_b_full.log
rm -f "$LOG"

# El frontend ya está construido en frontend/dist (ver README del spike).
GDK_BACKEND=x11 "$BIN" >"$LOG" 2>&1 &
APP_PID=$!
sleep 2.0
WID=$(LD_LIBRARY_PATH=$XD "$XD/xdotool" search --name "AnzanLab" 2>/dev/null | head -1)
if [ -n "$WID" ]; then
  LD_LIBRARY_PATH=$XD "$XD/xdotool" windowactivate --sync "$WID" >/dev/null 2>&1
  LD_LIBRARY_PATH=$XD "$XD/xdotool" windowraise "$WID" >/dev/null 2>&1
  echo "activada ventana $WID"
else
  echo "AVISO: no se encontró la ventana"
fi
wait "$APP_PID"
echo "EXIT=$?"
grep -E "SPIKE_B_" "$LOG" | tail -12
