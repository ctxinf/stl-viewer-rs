"""Check the zoom HUD at common DPI factors on an isolated X11 display."""
import io
import os
from pathlib import Path
import subprocess
import time
from PIL import Image

output = Path("target/window-test")
output.mkdir(parents=True, exist_ok=True)
for factor in (1, 1.5, 2):
    env = dict(os.environ, WINIT_X11_SCALE_FACTOR=str(factor),
               WINIT_UNIX_BACKEND="x11", LIBGL_ALWAYS_SOFTWARE="1")
    env.pop("WAYLAND_DISPLAY", None)
    process = subprocess.Popen(
        ["target/release/stl-viewer-linux", "target/triangle.stl"], env=env)
    try:
        deadline = time.monotonic() + 10
        while True:
            assert process.poll() is None, "Viewer exited before creating a window"
            windows = subprocess.run(
                ["xdotool", "search", "--pid", str(process.pid)],
                capture_output=True, text=True).stdout.splitlines()
            if windows:
                window = windows[-1]
                break
            assert time.monotonic() < deadline, "Window creation timed out"
            time.sleep(0.1)
        time.sleep(1)
        sizes = (None, (int(240 * factor), int(200 * factor)),
                 (int(800 * factor), int(600 * factor)))
        for size in sizes:
            if size:
                subprocess.run(["xdotool", "windowsize", window, *map(str, size)], check=True)
            time.sleep(0.3)
            data = subprocess.check_output(["import", "-window", window, "png:-"])
            image = Image.open(io.BytesIO(data)).convert("RGB")
            width, height = image.size
            ticks = image.crop((max(0, width - int(160 * factor)),
                                int(30 * factor), width, int(80 * factor)))
            white = sum(1 for r, g, b in ticks.getdata() if r > 200 and 140 < g < 230 and b < 170)
            assert white > 20, (factor, size, white)
            assert process.poll() is None
            image.save(output / f"dpi-{factor}-{width}x{height}.png")
        print(f"PASS: DPI {factor}, zoom percentage, narrow and enlarged viewport")
    finally:
        process.terminate()
        process.wait(timeout=5)
