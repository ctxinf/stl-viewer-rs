"""Regenerate the original one-triangle CI fixture (MIT).

Usage: python3 scripts/fixture.py tests/fixtures/triangle.stl
"""
import struct
import sys
with open(sys.argv[1], "wb") as f:
    f.write(b"STL Viewer test".ljust(80, b"\0"))
    f.write(struct.pack("<I12fH", 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0))
