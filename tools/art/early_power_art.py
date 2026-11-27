"""Textures for M33's MILLS tab (docs/mockups/mission-tree.html#mills, early-power design 12).

Copies the settlement set's textures the mills are built of (plank, timber,
fieldstone, thatch, iron, poles, grass, dirt, stone) from
assets/textures/settlement, unchanged, and draws the four the set has no
match for, 32 x 32 in the same palette and manner (a ramp lit from the top
left, no outline: these are world textures, CLAUDE.md "No outlines in the
world"):

- sailcloth: a windmill sail's linen on its lattice of laths;
- millwheel: a water wheel's paddle boards, wet at the foot, iron bands;
- riverwater: the race under the wheel, for the mockup's river;
- copperwire: a line's wire, a copper twist.

Each is sampled nearest in the mockup at the `repeat_m` written in
docs/mockups/mission-tree/tex/manifest.json (the settlement manifest's own
value for the copied ones).

    python3 tools/art/early_power_art.py
"""

import json
import random
import shutil
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "assets/textures/settlement"
OUT = ROOT / "docs/mockups/mission-tree/tex"

# name: repeat_m, from assets/textures/settlement/manifest.ron
COPIED = {
    "plank": 2.0, "timber": 1.0, "fieldstone": 2.0, "thatch": 2.0, "iron": 1.0,
    "poles": 2.0, "grass": 2.0, "dirt": 2.0, "stone": 2.0,
}


def manifest_repeat(name):
    text = (SRC / "manifest.ron").read_text()
    key = f'(name: "{name}",'
    for line in text.splitlines():
        if key in line and "repeat_m:" in line:
            return float(line.split("repeat_m:")[1].split(")")[0].strip().rstrip(","))
    return None


def ramp(base, k):
    return tuple(max(0, min(255, int(c * k))) for c in base) + (255,)


def sailcloth(rng):
    im = Image.new("RGBA", (32, 32))
    p = im.load()
    linen = (226, 214, 186)
    lath = (150, 104, 62)
    for y in range(32):
        for x in range(32):
            k = 0.94 + 0.08 * rng.random() - 0.10 * (x + y) / 62
            c = ramp(linen, k)
            if x % 16 in (0, 1) or y % 8 == 0:
                c = ramp(lath, 1.0 if (x % 16 == 0 or y % 8 == 0) else 0.8)
                if (x * 7 + y * 3) % 11 == 0:
                    c = ramp(lath, 0.7)
            p[x, y] = c
    return im


def millwheel(rng):
    im = Image.new("RGBA", (32, 32))
    p = im.load()
    wood = (164, 112, 66)
    for y in range(32):
        board = y // 8
        for x in range(32):
            k = 0.9 + 0.1 * rng.random() + 0.05 * (board % 2)
            if y % 8 == 7:
                k = 0.55
            if (x + board * 5) % 13 == 0 and rng.random() < 0.6:
                k -= 0.15  # grain
            wet = max(0.0, (y - 20) / 12)
            c = tuple(int(wood[i] * k * (1 - 0.35 * wet) + (40, 60, 70)[i] * 0.35 * wet) for i in range(3))
            if x in (4, 5, 26, 27):
                c = ramp((92, 98, 106), 1.0 if x % 2 == 0 else 0.75)[:3]  # iron band
            p[x, y] = tuple(max(0, min(255, v)) for v in c) + (255,)
    return im


def riverwater(rng):
    im = Image.new("RGBA", (32, 32))
    p = im.load()
    deep = (38, 92, 128)
    for y in range(32):
        for x in range(32):
            k = 0.92 + 0.10 * rng.random()
            c = ramp(deep, k)
            if (x + 2 * (y // 4)) % 16 in (0, 1) and y % 4 == 1:
                c = (126, 184, 206, 255)  # a ripple along the flow
            p[x, y] = c
    return im


def copperwire(rng):
    im = Image.new("RGBA", (32, 32))
    p = im.load()
    cu = (176, 104, 60)
    for y in range(32):
        for x in range(32):
            k = 0.82 + 0.28 * (((x + y) % 4) / 3)
            p[x, y] = ramp(cu, k * (0.95 + 0.06 * rng.random()))
    return im


DRAWN = {"sailcloth": (sailcloth, 2.0), "millwheel": (millwheel, 1.0), "riverwater": (riverwater, 4.0), "copperwire": (copperwire, 0.25)}

if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    manifest = {}
    for name, fallback in COPIED.items():
        shutil.copyfile(SRC / f"{name}.png", OUT / f"{name}.png")
        rep = manifest_repeat(name)
        manifest[name] = {"repeat_m": rep if rep else fallback, "from": f"assets/textures/settlement/{name}.png"}
    for name, (fn, rep) in DRAWN.items():
        fn(random.Random(name)).save(OUT / f"{name}.png")
        manifest[name] = {"repeat_m": rep, "from": "tools/art/early_power_art.py"}
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=1) + "\n")
    print(f"{len(manifest)} textures in {OUT}")
