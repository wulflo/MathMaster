#!/usr/bin/env bash
# Captura la ventana del spike a alta tasa para verificación independiente.
set -uo pipefail
cd "$(dirname "$0")"
APP=./target/release/spike-a-dioxus
LOG=/tmp/opencode/spike_a_capture.log
VID=/tmp/opencode/spike_a_capture.mkv
rm -f "$LOG" "$VID"

SPIKE_STAGE=capture WINIT_UNIX_BACKEND=x11 LD_LIBRARY_PATH="$PWD/.libs" \
  "$APP" >"$LOG" 2>&1 &
APP_PID=$!

X=""; Y=""; W=""; H=""
for _ in $(seq 1 100); do
  LINE=$(grep -m1 'SPIKE_A_GEOM' "$LOG" 2>/dev/null || true)
  if [ -n "$LINE" ]; then
    read -r _ X Y W H _SCALE <<<"$LINE"
    break
  fi
  sleep 0.1
done
if [ -z "$W" ]; then
  echo "NO_GEOM"; cat "$LOG"; kill "$APP_PID" 2>/dev/null; exit 1
fi
echo "GEOM x=$X y=$Y w=$W h=$H"
ffmpeg -hide_banner -loglevel error -y \
  -f x11grab -framerate 120 -video_size "${W}x${H}" -i ":0.0+${X},${Y}" \
  -t 20 -c:v libx264 -preset ultrafast -crf 16 -pix_fmt yuv420p "$VID"
FF=$?
wait "$APP_PID"
echo "FFMPEG_EXIT=$FF"
grep -E 'SPIKE_A_' "$LOG" | tail -3
ls -la "$VID" 2>&1
