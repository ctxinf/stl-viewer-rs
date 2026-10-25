#!/usr/bin/env bash
# Run inside a fresh distro container on the GitHub-hosted runner.
set -euo pipefail
source /etc/os-release
case "$ID" in
  ubuntu|debian)
    apt-get update
    apt-get install -y xvfb xauth xdotool imagemagick python3-pil libgl1-mesa-dri
    apt-get install -y ./dist/*.deb
    dpkg-query -W stl-viewer-linux
    ;;
  fedora)
    dnf install -y ./dist/*.rpm xorg-x11-server-Xvfb xorg-x11-xauth xdotool ImageMagick python3-pillow mesa-dri-drivers
    rpm -q stl-viewer-linux
    ;;
  arch)
    pacman -Syu --noconfirm
    pacman -S --noconfirm xorg-server-xvfb xorg-xauth xdotool imagemagick python-pillow mesa
    pacman -U --noconfirm ./dist/*.pkg.tar.zst
    pacman -Q stl-viewer-linux
    ;;
  *) exit 1 ;;
esac
mkdir -p target/install-test
cp tests/fixtures/triangle.stl target/triangle.stl
export VIEWER_BIN=/usr/bin/stl-viewer-linux LIBGL_ALWAYS_SOFTWARE=1
"$VIEWER_BIN" --help | tee target/install-test/help.txt
"$VIEWER_BIN" info tests/fixtures/triangle.stl | tee target/install-test/info.txt
grep -q '^Triangles: 1$' target/install-test/info.txt
if "$VIEWER_BIN" info tests/fixtures/missing.stl; then
  echo 'Missing input unexpectedly succeeded' >&2
  exit 1
fi
xvfb-run -a -s "-screen 0 2560x2048x24" "$VIEWER_BIN" screenshot tests/fixtures/triangle.stl -o target/install-test/export.png --width 640 --height 480
python3 - <<'PY'
from PIL import Image
with Image.open('target/install-test/export.png') as image:
    assert image.size == (640, 480)
    blue = sum(b > r * 1.5 and g > r * 1.3 and b > 50 for r, g, b in image.convert('RGB').getdata())
    assert blue > 1000, f'No model in exported image: {blue}'
PY
test -f /usr/share/applications/stl-viewer-linux.desktop
test -f /usr/share/icons/hicolor/128x128/apps/stl-viewer-linux.png
xvfb-run -a -s "-screen 0 2560x2048x24" bash scripts/smoke-x11.sh
cp target/window-test/filled.png target/install-test/window.png
