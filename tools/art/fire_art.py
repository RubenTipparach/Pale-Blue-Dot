#!/usr/bin/env python3
"""The wildfire's smoke puff sheet (`openspec/changes/wildfire` 4.2).

Writes assets/textures/fire/smoke.png: 128 x 32, four 32 x 32 cells, one
puff's life from tight to fraying. Each puff is a cluster of round lobes in
a five-tone grey ramp lit from the upper left, alpha its cover faded over
the outer three texels. No outline (CLAUDE.md, "No outlines in the world").
Seeded, so the file is the same on every run.

    python3 tools/art/fire_art.py
"""

import math
import os
import random

from PIL import Image

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
OUT = os.path.join(ROOT, "assets", "textures", "fire", "smoke.png")

# Dark to light: soot grey warmed a little toward the ground's browns.
RAMP = [(52, 50, 48), (74, 71, 67), (98, 94, 88), (124, 119, 111), (150, 144, 134)]
CELL = 32


def puff(frame, rng):
    """One frame's (shade, cover) per texel."""
    grow = [0.55, 0.72, 0.86, 0.95][frame]
    fray = [0.0, 0.08, 0.2, 0.38][frame]
    lobes = []
    for k in range(7):
        a = k / 7 * math.tau + rng.uniform(-0.3, 0.3)
        r = rng.uniform(0.18, 0.32) * grow
        d = rng.uniform(0.08, 0.2) * grow
        lobes.append((0.5 + math.cos(a) * d, 0.52 + math.sin(a) * d * 0.8, r))
    lobes.append((0.5, 0.48, 0.28 * grow))
    shade = [[0.0] * CELL for _ in range(CELL)]
    cover = [[0.0] * CELL for _ in range(CELL)]
    for y in range(CELL):
        for x in range(CELL):
            u, v = (x + 0.5) / CELL, (y + 0.5) / CELL
            best = -1.0
            lit = 0.0
            for (cx, cy, r) in lobes:
                dx, dy = (u - cx) / r, (v - cy) / r
                d2 = dx * dx + dy * dy
                if d2 < 1.0:
                    h = math.sqrt(1.0 - d2)
                    # Lit from the upper left: the normal's lean toward it.
                    l = 0.5 + 0.5 * (-dx * 0.6 - dy * 0.6 + h * 0.5)
                    if h > best:
                        best, lit = h, l
            if best < 0:
                continue
            # Fraying: an old puff has holes in it.
            if fray and rng.random() < fray * (1.0 - best):
                continue
            shade[y][x] = lit
            cover[y][x] = 1.0
    # The edge fades over its outer three texels.
    out = [[0.0] * CELL for _ in range(CELL)]
    for y in range(CELL):
        for x in range(CELL):
            if cover[y][x] == 0.0:
                continue
            near = 3
            for dy in range(-3, 4):
                for dx in range(-3, 4):
                    xx, yy = x + dx, y + dy
                    if not (0 <= xx < CELL and 0 <= yy < CELL) or cover[yy][xx] == 0.0:
                        near = min(near, max(abs(dx), abs(dy)) - 1)
            out[y][x] = [0.35, 0.6, 0.85, 1.0][max(0, min(3, near))]
    return shade, out


def main():
    rng = random.Random(0x5EED)
    img = Image.new("RGBA", (CELL * 4, CELL), (0, 0, 0, 0))
    for frame in range(4):
        shade, cover = puff(frame, rng)
        for y in range(CELL):
            for x in range(CELL):
                if cover[y][x] == 0.0:
                    continue
                tone = RAMP[max(0, min(4, int(shade[y][x] * 5)))]
                img.putpixel(
                    (frame * CELL + x, y), tone + (int(255 * cover[y][x] * 0.9),)
                )
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    img.save(OUT)
    print("wrote", OUT)


if __name__ == "__main__":
    main()
