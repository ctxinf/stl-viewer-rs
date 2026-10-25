#!/usr/bin/env bash
set -euo pipefail
export WINIT_UNIX_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1
unset WAYLAND_DISPLAY
"${VIEWER_BIN:-target/release/stl-viewer-linux}" target/triangle.stl >target/x11.log 2>&1 &
pid=$!
export VIEWER_PID=$pid
trap 'kill "$pid" 2>/dev/null || true' EXIT
for i in {1..100}; do
  if ! kill -0 "$pid" 2>/dev/null; then cat target/x11.log; exit 1; fi
  window=$(xdotool search --name 'triangle.stl' 2>/dev/null | head -1 || true)
  if [[ -n "$window" ]]; then
    sleep 1
    python3 scripts/check-window.py
    wait "$pid"
    exit 0
  fi
  sleep 0.1
done
cat target/x11.log
exit 1
