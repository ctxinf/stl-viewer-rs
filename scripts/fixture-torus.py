"""Generate the original MIT-licensed torus used in the README screenshot."""
import math
import struct
import sys

n, m = 64, 24

def vertex(i, j):
    u, v = 2 * math.pi * i / n, 2 * math.pi * j / m
    return ((2 + .65 * math.cos(v)) * math.cos(u),
            (2 + .65 * math.cos(v)) * math.sin(u), .65 * math.sin(v))

with open(sys.argv[1], "wb") as output:
    output.write(b"Original parametric torus - MIT".ljust(80, b"\0"))
    output.write(struct.pack("<I", n * m * 2))
    for i in range(n):
        for j in range(m):
            a, b, c, d = (vertex(k, l) for k, l in
                          [(i, j), (i+1, j), (i+1, j+1), (i, j+1)])
            for triangle in [(a, b, c), (a, c, d)]:
                output.write(struct.pack("<12fH", 0, 0, 0,
                             *(v for point in triangle for v in point), 0))
