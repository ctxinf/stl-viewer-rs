"""Check only the viewer's window on an isolated X11 test display."""
import os
from pathlib import Path
import subprocess
import time
from PIL import Image, ImageChops


def xdo(*args):
    return subprocess.check_output(["xdotool", *map(str, args)], text=True).strip()


window = xdo("search", "--name", "triangle.stl").splitlines()[0]
xdo("windowfocus", "--sync", window)
output = Path("target/window-test")
output.mkdir(parents=True, exist_ok=True)


def capture(name):
    time.sleep(0.25)
    path = output / f"{name}.png"
    subprocess.run(["import", "-window", window, str(path)], check=True)
    with Image.open(path) as image:
        return image.convert("RGB")


def key(name):
    xdo("key", name)


def changed(before, after, reason):
    assert ImageChops.difference(before, after).getbbox(), reason


filled = capture("filled")
blue = sum(1 for r, g, b in filled.getdata() if b > r * 1.5 and g > r * 1.3 and b > 50)
assert blue > 5000, f"Model was not rendered: {blue} model pixels"
key("w")
changed(filled, capture("wireframe"), "Wireframe did not change rendering")
key("w")
key("r")
assert ImageChops.difference(filled, capture("reset")).getbbox() is None

for button, name in [(1, "orbit"), (3, "pan")]:
    xdo("mousemove", "--window", window, 400, 350)
    xdo("mousedown", button)
    xdo("mousemove", "--window", window, 480, 390)
    xdo("mouseup", button)
    changed(filled, capture(name), f"{name} did not change rendering")
    key("r")
    time.sleep(0.2)

xdo("click", 4)
changed(filled, capture("zoom"), "Zoom did not change rendering")
key("r")
xdo("windowsize", window, 800, 600)
resized = capture("resized")
assert resized.size == (800, 600), resized.size

# An idle viewer should block on events rather than spend CPU redrawing.
pid = int(os.environ["VIEWER_PID"])
def cpu_ticks():
    fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    return int(fields[11]) + int(fields[12])
time.sleep(0.5)
start = cpu_ticks()
time.sleep(1)
assert cpu_ticks() - start <= max(3, os.sysconf("SC_CLK_TCK") // 20), "Idle redraw consumes CPU"
key("Escape")
print(f"PASS: {blue} geometry pixels; wireframe, reset, orbit, pan, zoom, resize, idle, exit")
