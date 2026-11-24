#!/usr/bin/env python3
"""Draw the smithy's forge fire and anvil textures (`townsfolk` design 26a,
"The fire and the anvil redone") into assets/textures/settlement/:

- forge_fire.png, 128 x 32: eight 16 x 32 frames of one flame tongue, root
  at the bottom row and tip at the top. Its ramp is the terrain brazier's own
  two fire colours (`planet_surface.wgsl`: white-gold 0.98, 0.66, 0.22 and
  orange 0.86, 0.28, 0.05, linear) with a deep coal red and a red-orange
  under them: red at the root, orange, yellow at the tips, every tone under
  the tonemapper's shoulder so the fire stays a fire at night. Its alpha is
  the tongue's ragged edge, which sways frame to frame; the bottom rows are
  solid, so the tongues share one base. The game steps the frames at the
  held flame's rate.
- anvil.png, 32 x 32: forged iron's blue-grey, a value up on the settlement
  `iron`, with dark hammer scale and a few bright pits. Seamless.
- anvilface.png, 32 x 32: the anvil's worn face, lighter and smoother,
  with streaks of polish along it and a few dark hammer marks. Seamless.

No outline anywhere. Deterministic: fixed seeds and integer rounding.

    python3 tools/art/forge_art.py            # write the PNGs
    python3 tools/art/forge_art.py --check    # fail if any differs
"""

import math
import random
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2] / "assets" / "textures" / "settlement"

FRAMES = 8
FW, FH = 16, 32


def srgb(linear):
    """A linear colour as 8-bit sRGB."""

    def one(c):
        c = max(0.0, min(1.0, c))
        s = 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055
        return int(round(s * 255))

    return tuple(one(c) for c in linear)


# The fire's ramp, root to tip, linear. The last two are the terrain fire's.
FIRE = [
    srgb((0.30, 0.030, 0.010)),  # coal red, the very root
    srgb((0.46, 0.060, 0.016)),  # deep red
    srgb((0.68, 0.140, 0.026)),  # red-orange
    srgb((0.86, 0.280, 0.050)),  # orange (the terrain fire's tip)
    srgb((0.93, 0.460, 0.120)),  # amber
    srgb((0.98, 0.660, 0.220)),  # yellow (the terrain fire's root)
]


def half_width(s):
    """The tongue's half width at height s (0 root, 1 tip), in a frame's
    width: solid across the bottom, then tapering to the tip. The smithy's
    geometry (`smithy::TONGUE_*`) is cut a little outside this."""
    if s < 0.4:
        return 0.5
    return 0.44 * ((1.0 - s) / 0.6) ** 0.85


def fire_frame(k):
    rng = random.Random(2600 + k)
    phase = k / FRAMES * math.tau
    tile = {}
    for y in range(FH):
        s = 1.0 - (y + 0.5) / FH
        # The tongue sways more toward its tip, a little differently each
        # frame, and its edge is ragged by a pixel either way.
        centre = 0.5 + 0.09 * s * math.sin(phase + 5.0 * s) + 0.03 * s * math.sin(2 * phase + 11.0 * s)
        w = half_width(s)
        lo = (centre - w) * FW + rng.choice((-0.6, 0.0, 0.0, 0.5))
        hi = (centre + w) * FW + rng.choice((-0.5, 0.0, 0.0, 0.6))
        for x in range(FW):
            u = x + 0.5
            if s < 0.4:
                inside = True
            else:
                inside = lo <= u <= hi
            if not inside:
                tile[(x, y)] = (0, 0, 0, 0)
                continue
            # Hotter up the tongue and toward its middle line.
            heart = 1.0 - min(1.0, abs(u / FW - centre) / max(w, 0.08))
            heat = s * 1.05 + 0.22 * heart * min(1.0, s * 2.5) + rng.uniform(-0.07, 0.07)
            step = int(heat * (len(FIRE) - 0.01))
            step = max(0, min(len(FIRE) - 1, step))
            # The root's two rows are coals: red, a few darker.
            if y >= FH - 2:
                step = rng.choice((0, 0, 1, 1, 2))
            tile[(x, y)] = FIRE[step] + (255,)
    return tile


def forge_fire():
    image = Image.new("RGBA", (FRAMES * FW, FH))
    for k in range(FRAMES):
        for (x, y), rgba in fire_frame(k).items():
            image.putpixel((k * FW + x, y), rgba)
    return image


def wrapped_noise(rng, size, cell):
    """Value noise that tiles over `size`, on a lattice `cell` px apart."""
    n = size // cell
    grid = [[rng.random() for _ in range(n)] for _ in range(n)]

    def at(x, y):
        gx, gy = x / cell, y / cell
        x0, y0 = int(gx) % n, int(gy) % n
        x1, y1 = (x0 + 1) % n, (y0 + 1) % n
        fx, fy = gx - int(gx), gy - int(gy)
        fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
        top = grid[y0][x0] * (1 - fx) + grid[y0][x1] * fx
        bottom = grid[y1][x0] * (1 - fx) + grid[y1][x1] * fx
        return top * (1 - fy) + bottom * fy

    return at


def iron(seed, ramp, lift, streaks):
    """A 32 px iron tile from `ramp` (dark to light), its tone from tiled
    noise plus `lift`, with polish `streaks` along it if asked."""
    rng = random.Random(seed)
    big = wrapped_noise(rng, 32, 8)
    small = wrapped_noise(rng, 32, 4)
    image = Image.new("RGBA", (32, 32))
    for y in range(32):
        for x in range(32):
            t = 0.55 * big(x, y) + 0.45 * small(x, y) + lift
            if streaks:
                t += 0.12 * math.sin((y + 0.35 * big(x, y) * 8) / 32 * math.tau * 3)
            t += rng.choice((0.0, 0.0, 0.0, 0.0, -0.08, 0.08))
            step = max(0, min(len(ramp) - 1, int(t * len(ramp))))
            image.putpixel((x, y), ramp[step] + (255,))
    # A few bright pits and dark hammer marks, a pixel or two each.
    for _ in range(6 if streaks else 9):
        x, y = rng.randrange(32), rng.randrange(32)
        image.putpixel((x, y), ramp[-1] + (255,))
    for _ in range(5 if streaks else 3):
        x, y = rng.randrange(32), rng.randrange(32)
        image.putpixel((x, y), ramp[0] + (255,))
        image.putpixel(((x + 1) % 32, y), ramp[1] + (255,))
    return image


# Forged iron, blue-grey, a value up on the settlement `iron` so it does not
# fall to black in shade.
ANVIL = [(58, 61, 70), (72, 76, 87), (86, 90, 102), (100, 104, 116), (116, 120, 131), (134, 138, 147)]
# The worn face: the same hue, lighter and narrower in its range.
FACE = [(92, 96, 106), (110, 114, 124), (126, 130, 139), (142, 146, 153), (160, 163, 168), (178, 180, 184)]


def textures():
    return {
        "forge_fire.png": forge_fire(),
        "anvil.png": iron(2611, ANVIL, -0.05, False),
        "anvilface.png": iron(2612, FACE, -0.02, True),
    }


def main():
    out = textures()
    if "--check" in sys.argv[1:]:
        stale = [
            name
            for name, image in out.items()
            if not (ROOT / name).exists()
            or Image.open(ROOT / name).convert("RGBA").tobytes() != image.tobytes()
        ]
        if stale:
            print("stale (run tools/art/forge_art.py): " + ", ".join(stale))
            sys.exit(1)
        print("the forge textures are current")
        return
    for name, image in out.items():
        image.save(ROOT / name)
        print(f"wrote {ROOT / name}")


if __name__ == "__main__":
    main()
