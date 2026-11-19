"""Mountain flora art for M13's Mountains tab (`mountain-ground` design 8, 9).

The owner, 2026-11-19: "nice, can you add more stuff to the mountain biomes?
add stuff to foliage detail playground".

Draws, as true pixel art with no outline (world art: CLAUDE.md "No outlines in
the world"), lit from the top left, alpha 0 or 255, every colour a step of the
mountain's own ground ramp or a low-saturation bloom:

* the alpine turf candidate (32 x 32, seamless): short cool sedge green with
  straw tips, soil and slate grit between the blades. Proposed for the
  mountains sheet's unused (3,2).
* the meadow's sprites (16 x 16): three tufts off the turf's ramp, gentian,
  edelweiss, alpine avens, moss campion, a glacier buttercup, a sedge clump,
  today's dead shrub, and two seamless cushion tops (pink and white).
* three low shrubs (32 x 32): dwarf juniper, heather, bilberry.

Writes `mountain/mountain_art.png` (128 x 128, a 16 px grid; the page reads
each sprite by its cell, see SPRITES below) and `mountain/sheet.png`, a
contact sheet at 4x. Nothing here writes into assets/: the game's art changes
only after the owner approves the look.

Run: python docs/mockups/foliage/mountain_art.py
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

# Cell index -> what is drawn there, in 16 px cells of the 128 px sheet.
# Read by foliage.html's Mountains tab (MT_SPRITE).
SPRITES = {
    "turf": (0, 0, 2),      # 32 x 32 at cells (0,0)-(1,1)
    "tuft0": (2, 0, 1), "tuft1": (3, 0, 1), "tuft2": (4, 0, 1),
    "gentian": (5, 0, 1), "edelweiss": (6, 0, 1), "avens": (7, 0, 1),
    "campion": (2, 1, 1), "buttercup": (3, 1, 1), "sedge": (4, 1, 1),
    "cushion_pink": (5, 1, 1), "cushion_white": (6, 1, 1), "deadshrub": (7, 1, 1),
    "juniper": (0, 2, 2), "heather": (2, 2, 2), "bilberry": (4, 2, 2),
}


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


def tuft(rng, dry=False):
    a = canvas(16)
    blades = 9
    for i in range(blades):
        x0 = 3 + int(i * 10 / blades) + rng.integers(0, 2)
        h = rng.integers(6, 12)
        lean = rng.uniform(-0.45, 0.45)
        for k in range(h):
            x = int(round(x0 + lean * k * 0.6))
            y = 15 - k
            t = k / h
            tone = TURF[1] if t < 0.3 else TURF[2] if t < 0.6 else TURF[3] if t < 0.85 else (TURF[5] if dry else TURF[4])
            put(a, x, y, tone)
    return a


def stem_leaves(a, rng, x, top, leaf=(TURF[2], TURF[3])):
    for y in range(top, 16):
        put(a, x, y, TURF[2])
    for dx, dy in ((-1, 14), (1, 13), (-2, 15), (2, 15), (-1, 12)):
        put(a, x + dx, dy, leaf[0] if dx < 0 else leaf[1])


def bloom(a, cx, cy, ramp, eye=None, shape="star"):
    dark, mid, light = ramp
    if shape == "bell":
        # a gentian trumpet: narrow at the base, open at the mouth
        for y, (x0, x1) in zip(range(cy - 3, cy + 2), ((-2, 2), (-2, 2), (-1, 1), (-1, 1), (0, 0))):
            for x in range(x0, x1 + 1):
                tone = light if (x < 0 and y < cy) else mid if x <= 0 else dark
                put(a, cx + x, y, tone)
        put(a, cx - 1, cy - 4, light)
        put(a, cx + 1, cy - 4, mid)
        return
    pts = [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)]
    if shape == "star":
        pts += [(-2, 0), (2, 0), (0, -2), (0, 2), (-1, -1), (1, 1)]
    for dx, dy in pts:
        tone = light if (dx < 0 or dy < 0) else mid
        if abs(dx) == 2 or abs(dy) == 2:
            tone = mid if (dx < 0 or dy < 0) else dark
        put(a, cx + dx, cy + dy, tone)
    if eye:
        put(a, cx, cy, eye)


def flower(kind, rng):
    a = canvas(16)
    if kind == "gentian":
        ramp = (hexc("#41477f"), hexc("#565d9e"), hexc("#767db6"))
        stem_leaves(a, rng, 6, 10)
        stem_leaves(a, rng, 10, 11)
        bloom(a, 6, 8, ramp, shape="bell")
        bloom(a, 10, 9, ramp, shape="bell")
    elif kind == "edelweiss":
        grey = (hexc("#76826f"), hexc("#97a08d"))
        for y in range(9, 16):
            put(a, 7, y, grey[0])
        for dx, dy in ((-1, 14), (1, 13), (-2, 15), (2, 14)):
            put(a, 7 + dx, dy, grey[1] if dx < 0 else grey[0])
        bloom(a, 7, 7, (hexc("#bfc3b5"), hexc("#d6d9cc"), hexc("#e8eae0")), eye=hexc("#b9ad74"))
    elif kind == "avens":
        stem_leaves(a, rng, 5, 9)
        stem_leaves(a, rng, 10, 8)
        y = (hexc("#a8913f"), hexc("#c3a94f"), hexc("#d8c26b"))
        bloom(a, 5, 7, y, eye=hexc("#8a7b3e"), shape="round")
        bloom(a, 10, 6, y, eye=hexc("#8a7b3e"), shape="round")
    elif kind == "campion":
        # moss campion: a low mat of pink stars
        for x in range(2, 14):
            for yy in range(12, 16):
                if rng.random() < 0.8:
                    put(a, x, yy, TURF[2] if (x + yy) % 3 else TURF[3])
        p = (hexc("#9a6577"), hexc("#b58193"), hexc("#cc9dab"))
        for cx, cy in ((4, 12), (8, 11), (11, 13), (6, 14)):
            bloom(a, cx, cy, p, shape="round")
    elif kind == "buttercup":
        stem_leaves(a, rng, 7, 10, leaf=(TURF[1], TURF[2]))
        bloom(a, 7, 8, (hexc("#c9c7bd"), hexc("#dedcd2"), hexc("#ecebe4")), eye=hexc("#c6a64a"), shape="star")
    elif kind == "deadshrub":
        # today's dead shrub on snow: bare grey-brown twigs
        tw = (hexc("#4a4038"), hexc("#6a5d50"))
        def branch(x, y, dx, n):
            for k in range(n):
                put(a, int(round(x + dx * k)), y - k, tw[k % 2 == 0])
        branch(7, 15, 0.0, 8)
        branch(7, 12, -0.6, 6)
        branch(8, 11, 0.7, 6)
        branch(6, 14, -0.9, 4)
        branch(9, 13, 0.4, 7)
    elif kind == "sedge":
        for i in range(11):
            x0 = 3 + i
            h = rng.integers(5, 11)
            lean = rng.uniform(-0.6, 0.6)
            for k in range(h):
                x = int(round(x0 + lean * k * 0.5))
                t = k / h
                tone = TURF[2] if t < 0.4 else TURF[4] if t < 0.75 else TURF[5]
                put(a, x, 15 - k, tone)
    return a


def cushion(dot):
    """A cushion plant's top, seamless 16 px: a tight green mat with blooms."""
    rng = np.random.default_rng(7 if dot == "pink" else 8)
    a = canvas(16)
    a[:, :] = TURF[2]
    for _ in range(70):
        put(a, rng.integers(16), rng.integers(16), TURF[3] if rng.random() < 0.6 else TURF[1], True)
    ramp = (hexc("#a06b7e"), hexc("#bd8a9b")) if dot == "pink" else (hexc("#cfd1c6"), hexc("#e6e7df"))
    for _ in range(16):
        x, y = rng.integers(16), rng.integers(16)
        put(a, x, y, ramp[1], True)
        put(a, x + 1, y, ramp[0], True)
    return a


def mound(rng, n, h, ramp, accent=None, accent_p=0.0, stems=None):
    """A low shrub as a 32 px card: a lumpy mound from the bottom, lit top left."""
    a = canvas(n)
    cx = n / 2
    for x in range(n):
        dx = (x - cx) / (n * 0.48)
        if abs(dx) >= 1:
            continue
        top = n - 1 - int(h * (1 - dx * dx) ** 0.7 + rng.integers(-1, 2))
        for y in range(max(top, 0), n):
            depth = (y - top) / max(n - top, 1)
            side = dx
            tone = ramp[3] if (depth < 0.18 and side < 0.2) else ramp[2] if depth < 0.4 and side < 0.5 else ramp[1] if depth < 0.75 else ramp[0]
            if rng.random() < 0.18:
                tone = ramp[max(0, ramp.index(tone) - 1)]
            put(a, x, y, tone)
    if stems:
        for _ in range(6):
            x = rng.integers(4, n - 4)
            for y in range(n - 3, n):
                put(a, x, y, stems)
    if accent:
        ys, xs = np.nonzero(a[:, :, 3])
        for i in range(len(xs)):
            if rng.random() < accent_p and ys[i] < n - 2:
                put(a, xs[i], ys[i], accent[0])
                put(a, xs[i] - 1, ys[i] - 1, accent[1])
    return a


def main():
    rng = np.random.default_rng(19)
    OUT.mkdir(exist_ok=True)
    sheet = np.zeros((128, 128, 4), np.uint8)

    def place(name, img):
        cx, cy, _ = SPRITES[name]
        h, w = img.shape[:2]
        sheet[cy * 16:cy * 16 + h, cx * 16:cx * 16 + w] = img

    place("turf", turf(rng))
    place("tuft0", tuft(rng))
    place("tuft1", tuft(rng))
    place("tuft2", tuft(rng, dry=True))
    for k in ("gentian", "edelweiss", "avens", "campion", "buttercup", "sedge", "deadshrub"):
        place(k, flower(k, rng))
    place("cushion_pink", cushion("pink"))
    place("cushion_white", cushion("white"))
    juniper = [hexc(c) for c in ("#26352f", "#334639", "#435a48", "#577058")]
    heather = [hexc(c) for c in ("#34402f", "#46523a", "#5a5a46", "#6e6a52")]
    bilberry = [hexc(c) for c in ("#2f4429", "#3f5a33", "#52713f", "#6a8a4c")]
    place("juniper", mound(rng, 32, 13, juniper, accent=(hexc("#59678a"), hexc("#7480a0")), accent_p=0.025, stems=hexc("#4a3a2c")))
    place("heather", mound(rng, 32, 11, heather, accent=(hexc("#86607a"), hexc("#a27d95")), accent_p=0.22, stems=hexc("#4a3a2c")))
    place("bilberry", mound(rng, 32, 15, bilberry, accent=(hexc("#2d3256"), hexc("#4a5180")), accent_p=0.05, stems=hexc("#55452f")))

    Image.fromarray(sheet, "RGBA").save(OUT / "mountain_art.png")
    # contact sheet at 4x over a slate ground, the turf tiled 3 x 3 beside it
    big = Image.new("RGBA", (128 * 4 + 96 * 4 + 16, 128 * 4), (51, 62, 80, 255))
    big.alpha_composite(Image.fromarray(sheet, "RGBA").resize((512, 512), Image.NEAREST), (0, 0))
    t = Image.fromarray(sheet[0:32, 0:32], "RGBA")
    tiled = Image.new("RGBA", (96, 96))
    for j in range(3):
        for i in range(3):
            tiled.paste(t, (i * 32, j * 32))
    big.alpha_composite(tiled.resize((384, 384), Image.NEAREST), (528, 0))
    big.save(OUT / "sheet.png")
    print("wrote", OUT / "mountain_art.png", "and", OUT / "sheet.png")


if __name__ == "__main__":
    main()
