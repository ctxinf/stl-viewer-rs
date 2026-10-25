"""Compare CLI startup + STL reading on local models; never publish model paths.

Usage: python3 scripts/benchmark-startup.py FILE.stl [FILE.stl ...]
Requires a release build, Flatpak Bambu Studio, and Blender on PATH.
"""
import statistics
import subprocess
import sys
import time
from pathlib import Path

viewer = Path(__file__).resolve().parents[1] / 'target/release/stl-viewer-linux'
for index, argument in enumerate(sys.argv[1:], 1):
    model = Path(argument).resolve()
    commands = {
        'STL Viewer': [str(viewer), 'info', str(model)],
        'Bambu Studio': ['flatpak', 'run', 'com.bambulab.BambuStudio', '--info', str(model)],
        'Blender': ['blender', '--background', '--factory-startup',
                    '--python-exit-code', '1', '--python-expr',
                    f'import bpy; bpy.ops.wm.stl_import(filepath={str(model)!r})'],
    }
    print(f'Sample {index}: {model.stat().st_size} bytes', flush=True)
    for name, command in commands.items():
        samples = []
        for _ in range(3):
            start = time.perf_counter()
            subprocess.run(command, cwd='/tmp', capture_output=True,
                           check=True, timeout=30)
            samples.append((time.perf_counter() - start) * 1000)
        print(f'{name}: {statistics.median(samples):.1f} ms (median of 3)', flush=True)
