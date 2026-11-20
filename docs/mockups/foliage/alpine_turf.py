"""The alpine turf candidate tile for M13's Mountains tab (`mountain-ground` design 8).

The mountains sheet has no grass, so the alpine meadow's ground is a candidate
tile, proposed for the sheet's unused (3,2). Drawn as true pixel art with no
outline, lit from the top left, alpha 255: short cool sedge green with straw
tips, soil and slate grit between the blades. 32 x 32 and seamless.

The meadow's plants are not drawn here: since M13 v6 (owner, 2026-11-19: "you
should use the foliage kit instead of making new plants") the Mountains tab
draws them with the playground's own kit (tufts coloured from this tile, the
flower stems and heads, bushes, the pine and rocks).

Writes `mountain/alpine_turf.png` and `mountain/sheet.png` (the tile tiled
3 x 3 at 4x). Nothing here writes into assets/: the game's art changes only
after the owner approves the look.

Run: python docs/mockups/foliage/alpine_turf.py
"""

from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
OUT = HERE / "mountain"


def hexc(h):
    h = h.lstrip("#")
    return (int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), 255)


# The turf's ramp: a cool alpine sedge green, a step greyer than the fields'
# pasture so it sits beside the slate, with straw tips and the sheet's soil.
TURF = [hexc(c) for c in ("#2c372b", "#3a4632", "#4b5a3b", "#5f6f45", "#7a8352", "#9a9364")]
SOIL = [hexc(c) for c in ("#3c342c", "#4d4337", "#5e5344")]
GRIT = [hexc(c) for c in ("#56606c", "#707b86", "#929ca4")]
CLEAR = (0, 0, 0, 0)

def canvas(n):
    return np.zeros((n, n, 4), np.uint8)


def put(a, x, y, c, wrap=False):
    n = a.shape[0]
    if wrap:
        x %= n
        y %= n
    if 0 <= x < n and 0 <= y < n:
        a[y, x] = c


def turf(rng):
    """The alpine turf tile: blades over a dark sward, soil and grit showing."""
    n = 32
    a = canvas(n)
    a[:, :] = TURF[1]
    # dark sward mottling
    for _ in range(90):
        put(a, rng.integers(n), rng.integers(n), TURF[0], True)
    # soil and grit between the blades
    for _ in range(14):
        x, y = rng.integers(n), rng.integers(n)
        put(a, x, y, SOIL[1], True)
        if rng.random() < 0.5:
            put(a, x + 1, y, SOIL[0], True)
    for _ in range(9):
        x, y = rng.integers(n), rng.integers(n)
        put(a, x, y, GRIT[1], True)
        put(a, x + 1, y, GRIT[0], True)
        put(a, x, y - 1, GRIT[2], True)
    # blades: short upright strokes, lit from the top left
    for _ in range(120):
        x, y = rng.integers(n), rng.integers(n)
        h = rng.integers(2, 5)
        lean = rng.choice([-1, 0, 0, 1])
        for k in range(h):
            tone = TURF[2] if k < h - 1 else TURF[3]
            put(a, x + (lean if k == h - 1 else 0), y - k, tone, True)
        if rng.random() < 0.35:
            put(a, x + lean, y - h, TURF[4] if rng.random() < 0.6 else TURF[5], True)
        put(a, x + 1, y, TURF[0], True)          # its shadow, one step down and right
    return a


def main():
    rng = np.random.default_rng(19)   # the seed and first draw of mountain_art.py (M13 v5): the same tile
    OUT.mkdir(exist_ok=True)
    tile = Image.fromarray(turf(rng), "RGBA")
    tile.save(OUT / "alpine_turf.png")
    tiled = Image.new("RGBA", (96, 96))
    for j in range(3):
        for i in range(3):
            tiled.paste(tile, (i * 32, j * 32))
    tiled.resize((384, 384), Image.NEAREST).save(OUT / "sheet.png")
    print("wrote", OUT / "alpine_turf.png", "and", OUT / "sheet.png")


if __name__ == "__main__":
    main()
