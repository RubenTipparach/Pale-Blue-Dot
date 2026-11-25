"""The forge's painted textures (`townsfolk` design 26b): the bed of coals,
the flame over it, and the anvil's forged iron and polished face. Drawn in
code with fixed seeds, pixel art in flat tone ramps (the settlement set's
iron ramp, blued a little, and the fire's own ramp from white-yellow to red),
no outlines, sampled nearest in the game.

    python3 tools/art/forge_art.py

- `embers` (32 x 32, tiles): dark clinker lumps with glowing orange and
  yellow seams between them, a few lumps burning through.
- `forgefire` (32 x 64, mapped once, cut out by its alpha): several tongues
  of uneven height standing on its bottom row, a white-yellow core at the
  root through yellow and orange to red tips.
- `anvil` (32 x 32, tiles): forged iron, dark grey-blue, mottled from the
  forging, with hammer marks and lighter worn specks.
- `anvilface` (32 x 32, tiles): the polished steel top, brighter, with fine
  grinding streaks along its length and a few pits.
"""

import math
import random
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SET = ROOT / "assets" / "textures" / "settlement"

# Clinker, dark to light, and the glow in its seams.
CLINKER = [(30, 22, 21), (44, 31, 27), (60, 42, 33), (78, 56, 42)]
GLOW = [(150, 52, 22), (204, 88, 28), (236, 136, 42), (250, 198, 92)]
# The flame, root to tip.
FIRE = [(255, 240, 184), (252, 206, 92), (240, 140, 40), (214, 84, 28), (170, 48, 24)]
# Forged iron, the settlement set's iron ramp, a touch cool.
IRON = [(50, 51, 57), (70, 72, 79), (80, 83, 90), (92, 95, 102), (118, 121, 128), (146, 149, 155)]
STEEL = [(118, 121, 128), (140, 143, 150), (150, 153, 160), (162, 165, 171), (186, 188, 193)]


def wrap_d(a, b, n):
    d = abs(a - b) % n
    return min(d, n - d)


def embers():
    rng = random.Random(2611)
    n = 32
    seeds = [(rng.uniform(0, n), rng.uniform(0, n)) for _ in range(14)]
    hot = {i for i in range(len(seeds)) if rng.random() < 0.15}
    im = Image.new("RGB", (n, n))
    for y in range(n):
        for x in range(n):
            ds = sorted(
                (math.hypot(wrap_d(x + 0.5, sx, n), wrap_d(y + 0.5, sy, n)), i)
                for i, (sx, sy) in enumerate(seeds)
            )
            (d0, i0), (d1, _) = ds[0], ds[1]
            edge = d1 - d0
            if edge < 0.5:
                # The seam between two lumps burns.
                c = GLOW[3] if (x * 7 + y * 3) % 4 == 0 else GLOW[2]
            elif edge < 1.1:
                c = GLOW[1] if i0 in hot or (x + 2 * y) % 4 == 0 else GLOW[0]
            elif i0 in hot and d0 < 1.4:
                # A lump burning through at its heart.
                c = GLOW[1]
            else:
                # Clinker, lit from the upper left inside each lump.
                sx, sy = seeds[i0]
                lean = ((x + 0.5 - sx + n / 2) % n - n / 2) + ((y + 0.5 - sy + n / 2) % n - n / 2)
                k = 2 if lean < -1.5 else (1 if lean < 1.5 else 0)
                if d0 < 1.2:
                    k = 3 if k == 2 else k + 1
                c = CLINKER[k]
            im.putpixel((x, y), c)
    return im


def forgefire():
    rng = random.Random(2612)
    w, h = 32, 64
    # Tongues: their root's middle, half-width at the root, height (of 64
    # pixels) and lean (pixels across per pixel up), each licking up from a
    # common bed of flame.
    tongues = [
        (4.0, 4.0, 24, -0.06),
        (9.5, 5.0, 43, -0.08),
        (16.0, 6.0, 61, 0.03),
        (14.0, 3.5, 36, -0.12),
        (22.0, 5.0, 49, 0.09),
        (27.0, 4.5, 31, 0.12),
        (19.5, 3.0, 38, 0.18),
    ]
    bed = 8

    def heat(x, k):
        """How far up its tongue `(x, k)` is (0 root, 1 tip), or None."""
        best = None
        if 1 <= x <= w - 2 and k < bed:
            best = k / (bed * 4.0)
        for cx, hw, th, lean in tongues:
            if k >= th:
                continue
            f = k / th
            half = hw * (1.0 - f) ** 0.75
            mid = cx + lean * k
            if abs(x + 0.5 - mid) < half:
                v = f * 0.85 + abs(x + 0.5 - mid) / max(hw, 1.0) * 0.25
                best = v if best is None else min(best, v)
        return best

    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    for x in range(w):
        for k in range(h):
            t = heat(x, k)
            if t is None:
                continue
            # Dither the band edges a little, the same every run.
            t += (rng.random() - 0.5) * 0.1
            if t < 0.2:
                c = FIRE[0]
            elif t < 0.4:
                c = FIRE[1]
            elif t < 0.62:
                c = FIRE[2]
            elif t < 0.82:
                c = FIRE[3]
            else:
                c = FIRE[4]
            im.putpixel((x, h - 1 - k), c + (255,))
    # The core's edge: the outermost columns burn orange, not white.
    for y in range(h):
        for x in (0, w - 1):
            r, g, b, a = im.getpixel((x, y))
            if a and (r, g, b) == FIRE[0]:
                im.putpixel((x, y), FIRE[1] + (255,))
    return im


def mottle(n, rng, scale, levels):
    """A tiling low-frequency field, quantised to `levels` steps."""
    waves = [
        (rng.randint(1, 3), rng.randint(-3, 3), rng.uniform(0, math.tau), rng.uniform(0.5, 1.0))
        for _ in range(scale)
    ]
    out = []
    for y in range(n):
        row = []
        for x in range(n):
            v = sum(a * math.sin(math.tau * (fx * x + fy * y) / n + p) for fx, fy, p, a in waves)
            row.append(v)
        out.append(row)
    lo = min(min(r) for r in out)
    hi = max(max(r) for r in out)
    return [[min(levels - 1, int((v - lo) / (hi - lo + 1e-9) * levels)) for v in r] for r in out]


def anvil():
    rng = random.Random(2613)
    n = 32
    field = mottle(n, rng, 5, 3)
    im = Image.new("RGB", (n, n))
    for y in range(n):
        for x in range(n):
            im.putpixel((x, y), IRON[1 + field[y][x]])
    # Hammer marks: shallow dents, lit on their lower right where the light
    # catches the far wall, shaded on their upper left.
    for _ in range(6):
        cx, cy = rng.randrange(n), rng.randrange(n)
        rx, ry = rng.choice([(2, 1), (3, 2)])
        for dy in range(-ry, ry + 1):
            for dx in range(-rx, rx + 1):
                q = (dx / (rx + 0.5)) ** 2 + (dy / (ry + 0.5)) ** 2
                if q > 1.0:
                    continue
                p = ((cx + dx) % n, (cy + dy) % n)
                if q > 0.5:
                    c = IRON[1] if dx + dy < 0 else IRON[3]
                else:
                    c = IRON[2]
                im.putpixel(p, c)
    # Worn specks, brighter where the iron is rubbed.
    for _ in range(18):
        x, y = rng.randrange(n), rng.randrange(n)
        im.putpixel((x, y), IRON[4] if rng.random() < 0.3 else IRON[3])
    return im


def anvilface():
    rng = random.Random(2614)
    n = 32
    field = mottle(n, rng, 4, 3)
    im = Image.new("RGB", (n, n))
    for y in range(n):
        for x in range(n):
            im.putpixel((x, y), STEEL[1 + field[y][x]])
    # Grinding streaks along the face's length (x).
    for _ in range(14):
        y, x0 = rng.randrange(n), rng.randrange(n)
        for k in range(rng.randint(5, 12)):
            im.putpixel(((x0 + k) % n, y), STEEL[4] if k % 4 else STEEL[3])
    # A few pits and dents.
    for _ in range(7):
        x, y = rng.randrange(n), rng.randrange(n)
        im.putpixel((x, y), STEEL[0])
        im.putpixel(((x + 1) % n, (y + 1) % n), STEEL[4])
    return im


def main():
    made = {
        "embers": (embers(), 1.0, False),
        "forgefire": (forgefire(), 0.0, True),
        "anvil": (anvil(), 0.6, False),
        "anvilface": (anvilface(), 0.6, False),
    }
    for name, (im, _, _) in made.items():
        im.save(SET / f"{name}.png")
    manifest = SET / "manifest.ron"
    text = manifest.read_text()
    for name, (im, rep, cut) in made.items():
        extra = ", cut: true" if cut else ""
        line = (
            f'    (name: "{name}", width: {im.width}, height: {im.height}, '
            f"repeat_m: {rep:.1f}{extra}),"
        )
        if f'"{name}"' not in text:
            if name == "embers":
                text = text.replace(
                    "  ],\n)",
                    "    // `townsfolk` design 26b: the forge's coals, flame and anvil, drawn\n"
                    "    // by tools/art/forge_art.py.\n  ],\n)",
                )
            text = text.replace("  ],\n)", line + "\n  ],\n)")
    manifest.write_text(text)


if __name__ == "__main__":
    main()
