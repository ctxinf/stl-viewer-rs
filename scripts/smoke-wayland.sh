#!/usr/bin/env bash
set -euo pipefail
export XDG_RUNTIME_DIR=$(mktemp -d)
chmod 700 "$XDG_RUNTIME_DIR"
export WAYLAND_DISPLAY=stl-test LIBGL_ALWAYS_SOFTWARE=1
unset DISPLAY
weston --backend=headless-backend.so --socket="$WAYLAND_DISPLAY" --idle-time=0 >target/weston.log 2>&1 &
compositor=$!
app=''
trap '[[ -z "$app" ]] || kill "$app" 2>/dev/null || true; kill "$compositor" 2>/dev/null || true; rm -rf "$XDG_RUNTIME_DIR"' EXIT
for i in {1..100}; do [[ ! -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]] || break; sleep 0.1; done
WAYLAND_DEBUG=1 target/release/stl-viewer-linux target/triangle.stl >target/wayland.log 2>&1 &
app=$!
for i in {1..100}; do
  if ! kill -0 "$app"; then cat target/wayland.log target/weston.log; exit 1; fi
  if grep -Eq 'wl_surface.*attach\(wl_buffer' target/wayland.log; then
    echo 'PASS: Wayland window submitted its first image buffer'
    exit 0
  fi
  sleep 0.1
done
cat target/wayland.log target/weston.log
exit 1
