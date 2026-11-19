"""Candidate sand tiles for `sand-tiles` (M13's Sand tab), drawn in code.

The owner, 2026-11-19: "can you redo sand texture? its got a lot of these
weird blue shells. also desert sand and beach sand should be different", and
"beach would have shells and stuff".

Draws, as true 32 px pixel art with no outline (world tiles: CLAUDE.md "No
outlines in the world"), seamless where the game tiles them:

* beach (0,0) dry shell sand: pale warm sand, a few small shells and shell
  bits in cream, white and soft pink/coral, a pebble, driftwood grit; no teal.
* beach (1,0) dry over wet sand side, (2,0) wet sand: the same, damp.
* desert (0,0) pale sand: warmer orange-ochre, fine wind-sorted grain, faint
  ripples, no shells. desert (1,0): that sand over the sheet's own strata.
  desert (2,0), the ochre grit, is kept as it is.

Writes, in this folder:
  sand_tiles.png   128 x 128: rows beach today, beach new, desert today,
                   desert new; columns the sheet's tiles (0,0) (1,0) (2,0) (3,0).
                   "Today" is copied texel for texel out of assets/tilesets/atlas.png.
  sheet.png        a contact sheet: each tile tiled 4 x 4, at 3x, today above new.

Nothing here writes into assets/: the game's art changes only after the owner
approves the look. Run: python docs/mockups/sand/sand_tiles.py
"""

from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
ATLAS = ROOT / "assets" / "tilesets" / "atlas.png"
N = 32
SLOT = {"beach": 2, "desert": 3}


def hexc(h):
    h = h.lstrip("#")
    return np.array([int(h[i : i + 2], 16) for i in (0, 2, 4)], dtype=np.uint8)


def today(sheet, tx, ty=0):
    a = np.array(Image.open(ATLAS).convert("RGB"))
    s = SLOT[sheet]
    x0, y0 = (s % 4) * 128 + tx * N, (s // 4) * 128 + ty * N
    return a[y0 : y0 + N, x0 : x0 + N].copy()


def value_noise(rng, cells, n=N):
    """Seamless value noise on an n-texel tile: a cells x cells lattice, wrapped."""
    g = rng.random((cells, cells))
    t = np.arange(n) * cells / n
    i0 = np.floor(t).astype(int)
    f = t - i0
    f = f * f * (3 - 2 * f)
    i1 = (i0 + 1) % cells
    a = g[np.ix_(i0, i0)]
    b = g[np.ix_(i0, i1)]
    c = g[np.ix_(i1, i0)]
    d = g[np.ix_(i1, i1)]
    fy, fx = f[:, None], f[None, :]
    return (a * (1 - fx) + b * fx) * (1 - fy) + (c * (1 - fx) + d * fx) * fy


def grain(rng, ramp, base, h, w=N, warm=None, speck=0.0):
    """Pixel sand: a ramp index field from wrapped noise plus single-texel flecks.

    `ramp` runs dark to light; `base` is the index most texels sit on. The
    noise only nudges a texel one step, so the tile stays low in contrast.
    """
    n = value_noise(rng, 4, max(h, w))[:h, :w] * 0.6 + value_noise(rng, 8, max(h, w))[:h, :w] * 0.4
    r = rng.random((h, w))
    idx = np.full((h, w), base)
    idx[(n > 0.62) & (r < 0.55)] = base + 1
    idx[(n < 0.38) & (r < 0.55)] = base - 1
    f = rng.random((h, w))
    idx[f < 0.045] = base - 1
    idx[f > 0.97] = base + 1
    g = rng.random((h, w))
    idx[g < 0.012] = max(base - 2, 0)
    idx[g > 0.992] = min(base + 2, len(ramp) - 1)
    out = np.array(ramp)[np.clip(idx, 0, len(ramp) - 1)]
    if warm is not None:
        out[rng.random((h, w)) < speck] = warm
    return out


# ---------------------------------------------------------------- beach ----
# Sand ramp off the beach sheet's own top (#e2cb98 its commonest texel).
BEACH = [hexc(c) for c in ("#cdb383", "#d8bf8e", "#e2cb98", "#e9d4a4", "#efdcb0")]
# Damp sand, off the sheet's wet sand (2,0), #9b7b57.
DAMP = [hexc(c) for c in ("#88694a", "#927353", "#9b7b57", "#a58560", "#b0916b")]
SHADE = hexc("#d3ba88")  # a shell's shadow on dry sand: one sand step down, no outline
SHADE_DAMP = hexc("#907152")
# Each shell its own short ramp: light, mid, dark (all warm; no teal, no blue).
SHELLS = {
    "cream": [hexc("#fffaeb"), hexc("#f4e6c6"), hexc("#cdb68c")],
    "white": [hexc("#ffffff"), hexc("#f1ece2"), hexc("#cfc1a6")],
    "pink": [hexc("#fbe0d4"), hexc("#eeb7a2"), hexc("#cc937c")],
    "coral": [hexc("#f8cdb4"), hexc("#e59d80"), hexc("#c0806a")],
}
PEBBLE = [hexc("#cbb89c"), hexc("#b3a084"), hexc("#9c8a70")]
GRIT = hexc("#a4865f")  # driftwood and weed grit

# Sprites: L M D the shell's ramp, s a shadow texel on the sand, . nothing.
SCALLOP = [".LLL.", "LLMLM", "LMLMD", ".MLD.", "..Ds"]
CLAM = [".LLL.", "LLMMD", ".DDDs"]
SPIRAL = [".LL.", "LMLM", "LLDM", ".MDs"]
CONE = ["LL...", "LML..", ".MLM.", "..MDs"]
CONE2 = [".L", "LM", "MD", ".s"]
BIT_H = ["LLM", ".ss"]
BIT_V = ["L", "M", "s"]
BIT_D = ["LM", "Ms"]
SPECK = ["L"]
PEB = [".LM", "LMD", ".Ds"]


def stamp(img, sprite, x, y, ramp, shade, wrap_y=True):
    h, w = img.shape[:2]
    for j, row in enumerate(sprite):
        for i, ch in enumerate(row):
            if ch == ".":
                continue
            yy = y + j
            if wrap_y:
                yy %= h
            elif not 0 <= yy < h:
                continue
            xx = (x + i) % w
            img[yy, xx] = shade if ch == "s" else ramp["LMD".index(ch)]


def scatter(rng, img, items, shade, top=0, bottom=N, wrap_y=True, gap=6):
    """Place sprites at least `gap` texels apart, wrapping round the tile."""
    placed = []
    h, w = img.shape[:2]
    for sprite, ramp in items:
        for _ in range(200):
            x = int(rng.integers(0, w))
            y = int(rng.integers(top, bottom - (0 if wrap_y else len(sprite))))
            cx, cy = (x + len(sprite[0]) / 2) % w, y + len(sprite) / 2

            def d(p):
                dx = abs(p[0] - cx)
                dy = abs(p[1] - cy)
                return min(dx, w - dx), (min(dy, h - dy) if wrap_y else dy)

            if all(max(*d(p)) >= gap for p in placed):
                placed.append((cx, cy))
                stamp(img, sprite, x, y, ramp, shade, wrap_y)
                break


def beach_items(rng, damp=False):
    s = SHELLS
    if damp:
        # Under the damp sand the shells are fewer, broken and duller.
        dull = {k: [np.clip(c.astype(int) - 38, 0, 255).astype(np.uint8) for c in v] for k, v in s.items()}
        return [(BIT_H, dull["cream"]), (BIT_V, dull["pink"]), (SPECK, dull["white"]),
                (BIT_D, dull["coral"]), (SPECK, dull["cream"]), (PEB, PEBBLE)]
    return [
        (SCALLOP, s["pink"]), (CLAM, s["white"]), (SPIRAL, s["cream"]), (CONE, s["coral"]),
        (SCALLOP, s["white"]), (CLAM, s["coral"]), (CONE2, s["white"]), (SPIRAL, s["pink"]),
        (BIT_H, s["white"]), (BIT_V, s["cream"]), (BIT_D, s["pink"]), (BIT_H, s["coral"]),
        (SPECK, s["white"]), (SPECK, s["cream"]), (SPECK, s["pink"]), (SPECK, s["white"]),
        (PEB, PEBBLE),
    ]


def beach_top(seed=11):
    rng = np.random.default_rng(seed)
    img = grain(rng, BEACH, 2, N, warm=GRIT, speck=0.008)
    scatter(rng, img, beach_items(rng), SHADE, gap=5)
    return img


def beach_wet(seed=23):
    rng = np.random.default_rng(seed)
    img = grain(rng, DAMP, 2, N, warm=hexc("#7d6044"), speck=0.012)
    scatter(rng, img, beach_items(rng, damp=True), SHADE_DAMP, gap=8)
    return img


def edge(rng, mean, amp):
    """A wavy lower edge, periodic across the tile (so the side tiles seamlessly)."""
    x = np.arange(N)
    p1, p2 = rng.random() * 6.28, rng.random() * 6.28
    e = mean + amp * (0.6 * np.sin(2 * np.pi * x / 32 + p1) + 0.4 * np.sin(2 * np.pi * x / 8 + p2))
    return np.round(e + rng.integers(-1, 2, N) * 0.5).astype(int)


def beach_side(seed=31):
    rng = np.random.default_rng(seed)
    img = beach_wet(seed + 1)
    dry = grain(rng, BEACH, 2, N, warm=GRIT, speck=0.008)
    scatter(rng, dry, beach_items(rng)[8:15], SHADE, bottom=8, wrap_y=False, gap=6)
    e = edge(rng, 9, 1.6)
    for x in range(N):
        img[: e[x], x] = dry[: e[x], x]
        img[e[x], x] = BEACH[0]          # the dry sand's foot, a step down
        img[e[x] + 1, x] = DAMP[1]       # the damp line under it
    return img


# --------------------------------------------------------------- desert ----
# Warmer orange-ochre than today's #ebc586, off the desert sheet's own colours
# (#d59a58 and #e4c78e in biomes.json).
DESERT = [hexc(c) for c in ("#c4824a", "#d39556", "#dea464", "#e6b373", "#edc184")]
RUST = hexc("#b9693f")
QUARTZ = hexc("#f2cf98")


def desert_top(seed=41):
    rng = np.random.default_rng(seed)
    img = grain(rng, DESERT, 2, N)
    # Faint wind ripples: four crests down the tile and one turn across it,
    # so the pattern wraps; warped by wrapped noise so they are not ruled lines.
    y, x = np.mgrid[0:N, 0:N]
    warp = (value_noise(rng, 4) - 0.5) * 0.9
    ph = (4 * y + 1 * x) / N + warp
    f = ph - np.floor(ph)                # 0..1 across one ripple
    r = rng.random((N, N))
    lee = (f > 0.80) & (f < 0.93)        # the steep lee face, in shade
    crest = (f > 0.72) & (f <= 0.80)     # the crest, catching the light
    img[lee & (r < 0.85)] = DESERT[1]
    img[lee & (r < 0.18)] = DESERT[0]
    img[crest & (r < 0.45)] = DESERT[3]
    s = rng.random((N, N))
    img[s < 0.010] = RUST
    img[s > 0.993] = QUARTZ
    return img


def desert_side(seed=53):
    """The sheet's own stratified side, its pale top band re-sanded in the new sand.

    Today's strata are kept texel for texel; only the band of today's pale sand
    over them (every texel above the first strata texel in its column) takes
    the new top, so the side meets the new cap in its colour.
    """
    img = today("desert", 1, 0)
    top = desert_top(seed - 12)  # the same sand as the cap
    luma = img.astype(int) @ np.array([299, 587, 114]) // 1000
    for x in range(N):
        y = 0
        while y < N and luma[y, x] > 180:
            y += 1
        img[:y, x] = top[:y, x]
    return img


def check_seamless(img, name, axes=(0, 1)):
    """No hard seam: the step across the wrap is no bigger than steps inside."""
    a = img.astype(int)
    for ax in axes:
        inside = np.abs(np.diff(a, axis=ax)).sum(-1).mean()
        wrap = np.abs(a.take(0, axis=ax) - a.take(-1, axis=ax)).sum(-1).mean()
        assert wrap < inside * 1.8 + 6, f"{name}: seam on axis {ax} ({wrap:.1f} vs {inside:.1f})"


def no_blue(img, name):
    """No texel whose blue beats its red, or that leans teal: the owner's 'blue shells'."""
    a = img.astype(int)
    bad = (a[..., 2] > a[..., 0]) | (a[..., 1] > a[..., 0] + 4)
    assert not bad.any(), f"{name}: {bad.sum()} blue or teal texels"


def main():
    new = {
        ("beach", 0): beach_top(),
        ("beach", 1): beach_side(),
        ("beach", 2): beach_wet(),
        ("beach", 3): today("beach", 3, 0),
        ("desert", 0): desert_top(),
        ("desert", 1): desert_side(),
        ("desert", 2): today("desert", 2, 0),
        ("desert", 3): today("desert", 3, 0),
    }
    for (sheet, tx), img in new.items():
        if tx in (0, 2):
            check_seamless(img, f"{sheet} ({tx},0)")
        elif tx == 1:
            check_seamless(img, f"{sheet} ({tx},0)", axes=(1,))
        if tx < 3 and not (sheet == "desert" and tx == 2):
            no_blue(img, f"{sheet} ({tx},0)")
    out = np.zeros((128, 128, 3), np.uint8)
    for row, (sheet, kind) in enumerate([("beach", "t"), ("beach", "n"), ("desert", "t"), ("desert", "n")]):
        for tx in range(4):
            img = today(sheet, tx) if kind == "t" else new[(sheet, tx)]
            out[row * N : row * N + N, tx * N : tx * N + N] = img
    Image.fromarray(out).save(HERE / "sand_tiles.png")
    # Contact sheet: each tile 4 x 4 at 3x, today then new, beach then desert.
    sc, gap = 3, 6
    cell = N * 4 * sc
    sheet_img = Image.new("RGB", (4 * cell + 5 * gap, 4 * cell + 5 * gap), (21, 26, 18))
    for row in range(4):
        for tx in range(4):
            t = Image.fromarray(out[row * N : row * N + N, tx * N : tx * N + N])
            tiled = Image.new("RGB", (N * 4, N * 4))
            for j in range(4):
                for i in range(4):
                    tiled.paste(t, (i * N, j * N))
            tiled = tiled.resize((cell, cell), Image.NEAREST)
            sheet_img.paste(tiled, (gap + tx * (cell + gap), gap + row * (cell + gap)))
    sheet_img.save(HERE / "sheet.png")
    # Mean colours and the tuft ramps a repaint would move (planet_terrain.rs test).
    for row, label in enumerate(["beach today", "beach new", "desert today", "desert new"]):
        top = out[row * N : row * N + N, 0:N].reshape(-1, 3).astype(float) / 255
        lin = np.where(top <= 0.04045, top / 12.92, ((top + 0.055) / 1.055) ** 2.4).mean(0)
        print(f"{label:13s} top mean #{''.join('%02x' % round(255 * v ** (1 / 2.2)) for v in lin)}")


if __name__ == "__main__":
    main()
