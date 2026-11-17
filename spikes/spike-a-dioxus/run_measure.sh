#!/usr/bin/env bash
# Ejecuta el spike A completo en X11 (XWayland) y activa la ventana para evitar
# el throttling de rAF que WebKit aplica a ventanas sin foco.
set -uo pipefail
cd "$(dirname "$0")"
APP=./target/release/spike-a-dioxus
XD=/tmp/opencode/xdotool-3.20160805.1
LOG=/tmp/opencode/spike_a_full.log
rm -f "$LOG"

# En esta máquina el binario enlaza libxdo.so.3 (dependencia de muda/dioxus-desktop)
# y el sistema sólo provee libxdo.so.3 en /tmp/opencode; .libs/ tiene los symlinks.
if [ -d .libs ]; then
  export LD_LIBRARY_PATH="$PWD/.libs${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
GDK_BACKEND=x11 "$APP" >"$LOG" 2>&1 &
APP_PID=$!
sleep 1.2
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
grep -E "STAGE|SPIKE_A_" "$LOG" | tail -12
