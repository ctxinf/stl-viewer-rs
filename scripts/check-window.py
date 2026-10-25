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
# Menus and dialogs own their input, including letter shortcuts.
xdo("mousemove", "--window", window, 400, 350)
scene = capture("ui-scene")
xdo("mousemove", "--window", window, 16, 10)
xdo("click", 1)
changed(scene, capture("menu-open"), "File menu did not open")
key("w")
key("Escape")
xdo("mousemove", "--window", window, 400, 350)
assert ImageChops.difference(scene, capture("menu-closed")).getbbox() is None, "Menu input changed model"
key("F1")
changed(scene, capture("help-open"), "Help did not open")
key("w")
key("Escape")
assert ImageChops.difference(scene, capture("help-closed")).getbbox() is None, "Dialog input changed model"

# A face click selects a standard view; a drag continues outside the HUD.
xdo("mousemove", "--window", window, 72, 656)
xdo("click", 1)
face = capture("cube-face")
changed(scene.crop((200, 40, 1000, 720)), face.crop((200, 40, 1000, 720)), "Face click did not change camera")
xdo("mousedown", 1)
xdo("mousemove", "--window", window, 300, 450)
xdo("mouseup", 1)
changed(face, capture("cube-drag"), "Cube drag did not orbit")
# Both origins must apply exactly the same scene controls.
for button in (1, 3):
    key("r")
    xdo("mousemove", "--window", window, 72, 656)
    xdo("mousedown", button)
    xdo("mousemove", "--window", window, 112, 676)
    xdo("mouseup", button)
    from_cube = capture(f"cube-unified-{button}")
    key("r")
    xdo("mousemove", "--window", window, 400, 350)
    xdo("mousedown", button)
    xdo("mousemove", "--window", window, 440, 370)
    xdo("mouseup", button)
    from_scene = capture(f"scene-unified-{button}")
    assert ImageChops.difference(from_cube.crop((200, 40, 1000, 720)), from_scene.crop((200, 40, 1000, 720))).getbbox() is None, "Cube and scene drag differ"
key("r")
xdo("mousemove", "--window", window, 72, 656)
xdo("click", 4)
wheel_cube = capture("cube-wheel")
key("r")
xdo("mousemove", "--window", window, 400, 350)
xdo("click", 4)
wheel_scene = capture("scene-wheel")
assert ImageChops.difference(wheel_cube.crop((200, 40, 1000, 720)), wheel_scene.crop((200, 40, 1000, 720))).getbbox() is None, "Cube and scene wheel differ"
assert wheel_scene.getpixel((16, 602)) == (49, 55, 65), "Cube backing is not opaque"
key("r")
xdo("mousemove", "--window", window, 400, 350)
scale_base = capture("scale-base")
xdo("click", 4)
scale_zoom = capture("scale-zoom")
changed(scale_base.crop((0, 720, 180, 760)), scale_zoom.crop((0, 720, 180, 760)), "Zoom percentage did not follow zoom")
key("r")
xdo("mousedown", 3)
xdo("mousemove", "--window", window, 500, 380)
xdo("mouseup", 3)
scale_pan = capture("scale-pan")
assert ImageChops.difference(scale_base.crop((0, 720, 180, 760)), scale_pan.crop((0, 720, 180, 760))).getbbox() is None, "Pan moved zoom HUD"
key("r")

# Focus loss clears the scene drag even if its release arrives elsewhere.
xdo("mousemove", "--window", window, 400, 350)
xdo("mousedown", 1)
xdo("windowfocus", 1)
xdo("mouseup", 1)
xdo("windowfocus", window)
xdo("mousemove", "--window", window, 450, 390)
assert ImageChops.difference(scene, capture("focus-restored")).getbbox() is None, "Focus loss left a drag active"

xdo("windowsize", window, 220, 200)
assert capture("narrow").size == (220, 200)
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
print(f"PASS: {blue} geometry pixels; wireframe, reset, orbit, pan, zoom, resize, menus, dialogs, cube, unified drag, zoom percentage, focus loss, narrow, idle, exit")
