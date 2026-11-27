#!/usr/bin/env python3
"""The pixel textures of Survival's weapons and stations (`survival-mode`
design 16 to 18, tasks 4.10): the approved weapons mockup's own textures
(docs/mockups/survival/weapons.js, survey G10), ported texel for texel, and
the tanning rack's hide.

Each is 16 or 32 px in the game's palette, sampled nearest, written to
assets/textures/settlement/ beside the towns' set with its manifest row (the
weapons are mapped once over each part, `repeat_m: 0`):

| Texture | Size | What | Mockup |
| --- | --- | --- | --- |
| weapon_shaft | 32 | a spear's shaft along v, its fibre binding at the top | `shaftT` |
| weapon_stone | 16 | knapped stone: a spear's and an arrow's head | `stoneT` |
| weapon_bronze | 16 | cast bronze: the hunting axe's head, the sword's guard and pommel | `bronzeT` |
| weapon_iron | 16 | the same cast in iron's ramp (the iron tier) | `bronzeT`, iron's colour |
| weapon_leather | 16 | a sword's grip wrap | `leatherT` |
| weapon_bow | 32 | the bow's limb along u, its leather grip in the middle | `bowT` |
| weapon_feather | 16 | an arrow's fletching | `featherT` |
| weapon_wood | 32 | an arrow's shaft | `woodT` |
| weapon_haft | 32 | the hunting axe's handle, its leather wrap down one edge | the axe's handle |
| weapon_blade_bronze | 16 | a sword's blade: its fuller and bright edges | the sword's blade |
| weapon_blade_iron | 16 | the same in iron | |
| hide | 32 | a hide laced on the tanning rack | M33's hide icon's ramp |

    python3 tools/art/weapon_textures.py            # write them
    python3 tools/art/weapon_textures.py --check    # fail if any differs
"""

import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "textures" / "settlement"
M32 = 0xFFFFFFFF


def imul(a, b):
    return (a * b) & M32


def h(x, y, s=0):
    """The mockup's `S.hash`, bit for bit (JavaScript's |0, Math.imul, >>>)."""
    v = x * 374761393 + y * 668265263 + s * 2246822519
    v &= M32
    v = imul(v ^ (v >> 13), 1274126177)
    return ((v ^ (v >> 16)) & M32) / 4294967296.0


def clamp(v):
    return max(0, min(255, int(round(v))))


def wood(x, y):
    g = 0.78 if (x * 3 + y // 4) % 9 < 2 else 1.0
    v = (150 + 30 * h(x, y >> 2, 1)) * g
    return (v, v * 0.7, v * 0.44)


def shaft(x, y):
    if y < 4:
        b = 1.0 if (x + y) % 4 < 2 else 0.8
        return (206 * b, 180 * b, 112 * b)
    g = 0.8 if x % 6 < 1 else 1.0
    v = (156 + 26 * h(x, y >> 3, 2)) * g
    return (v, v * 0.7, v * 0.45)


def stone(x, y):
    f = h(x >> 1, y >> 1, 3)
    v = 120 + 70 * f + (30 if h(x, y, 4) > 0.85 else 0)
    return (v * 0.95, v * 0.95, v)


def cast(colour):
    def tex(x, y):
        f = h(x >> 2, y >> 2, 5)
        d = 0.9 if h(x, y, 6) > 0.92 else 1.0
        v = (0.94 + 0.08 * f) * d
        return tuple(c * v for c in colour)

    return tex


BRONZE = (226, 160, 82)
IRON = (176, 186, 198)


def leather(x, y):
    band = 0.7 if (x + y * 2) % 8 < 2 else 1.0
    v = (0.9 + 0.2 * h(x, y, 7)) * band
    return (120 * v, 76 * v, 44 * v)


def bow(x, y):
    if 14 <= x <= 18:
        b = 0.75 if (y + x) % 3 == 0 else 1.0
        return (118 * b, 74 * b, 42 * b)
    g = 0.8 if (y * 5 + x // 3) % 7 < 1 else 1.0
    v = (170 + 30 * h(x >> 2, y, 8)) * g
    return (v, v * 0.72, v * 0.46)


def feather(x, y):
    v = 0.7 if x == 8 else 0.95 + 0.1 * h(x, y >> 1, 9)
    return (236 * v, 230 * v, 216 * v)


def haft(x, y):
    if x < 7:
        return (96, 60, 36) if (x + y) % 3 == 0 else (112, 72, 42)
    v = 160 + 20 * h(x >> 2, y, 13)
    return (v, v * 0.7, v * 0.45)


def blade(colour):
    def tex(x, y):
        f = 0.84 if x in (7, 8) else 1.12 if (x < 2 or x > 13) else 1.0
        v = (0.95 + 0.06 * h(x >> 2, y >> 2, 14)) * f
        return tuple(c * v for c in colour)

    return tex


BLADE_BRONZE = (236, 180, 100)
BLADE_IRON = (190, 198, 210)


def hide(x, y):
    """A tanned-in-the-making hide: M33's hide ramp (176, 120, 72 over 127,
    80, 48), mottled, a darker flesh-side blotch and a few hair flecks."""
    f = h(x >> 1, y >> 1, 21)
    v = 0.88 + 0.18 * f
    if h(x >> 2, y >> 2, 22) > 0.78:
        v *= 0.82
    if h(x, y, 23) > 0.93:
        v *= 1.12
    return (176 * v, 120 * v, 72 * v)


TEXTURES = {
    "weapon_shaft": (32, shaft),
    "weapon_stone": (16, stone),
    "weapon_bronze": (16, cast(BRONZE)),
    "weapon_iron": (16, cast(IRON)),
    "weapon_leather": (16, leather),
    "weapon_bow": (32, bow),
    "weapon_feather": (16, feather),
    "weapon_wood": (32, wood),
    "weapon_haft": (32, haft),
    "weapon_blade_bronze": (16, blade(BLADE_BRONZE)),
    "weapon_blade_iron": (16, blade(BLADE_IRON)),
    "hide": (32, hide),
}


def render(n, fn):
    im = Image.new("RGBA", (n, n))
    px = im.load()
    for y in range(n):
        for x in range(n):
            r, g, b = fn(x, y)
            px[x, y] = (clamp(r), clamp(g), clamp(b), 255)
    return im


def main():
    check = "--check" in sys.argv
    bad = []
    for name, (n, fn) in TEXTURES.items():
        im = render(n, fn)
        path = OUT / f"{name}.png"
        if check:
            if not path.exists() or Image.open(path).convert("RGBA").tobytes() != im.tobytes():
                bad.append(path)
            continue
        im.save(path)
    if bad:
        for path in bad:
            print(f"{path} differs from tools/art/weapon_textures.py")
        sys.exit(1)
    print(f"{len(TEXTURES)} textures {'match' if check else 'written'}")


if __name__ == "__main__":
    main()
