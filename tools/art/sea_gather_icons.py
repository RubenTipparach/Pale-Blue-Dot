"""Sea-gathering icons (`sea-gathering` design 9): the pack's and hotbar's
16 px goods icons for what the sea floor gives, rendered from the game's
own models, plus pixel maps for the three goods that have none.

The models come from `tools/art/sea_models.json`, which the game writes from
its reef model library (`fish::places::library`):

    cargo test --profile fast -p pbd-app --lib -- --ignored dump_sea_models

Each model is drawn as the reef draws it: its triangles, its vertex colours
times its kind's first palette colour, the water life's atlas sampled
nearest at its UVs, lit from the top left, seen from a three-quarter view
above. The 64 px render is reduced to 16 px by area average (a pixel kept
where at least half is covered), its colours snapped to a ramp of at most
five of its own, and the 1 px dark outline icons carry is added round it
(CLAUDE.md: outlines are for UI icons only, never in the world).

Kelp is the kelp card the game draws (`assets/textures/water_life/
kelp_card.png`), not a model: its icon is a frond in the card's colours. Fertiliser, steamed mussels and kelp
crisps have no model; they are pixel maps in the goods' style.

    python3 tools/art/sea_gather_icons.py            # writes assets/items/goods/
    python3 tools/art/sea_gather_icons.py --sheet X  # also a contact sheet
"""

import json
import math
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "items" / "goods"
MODELS = ROOT / "tools" / "art" / "sea_models.json"
ATLAS = ROOT / "assets" / "textures" / "water_life" / "atlas.png"

OUTLINE = (22, 22, 18, 255)
BIG = 64
SMALL = 16


def srgb_to_lin(c):
    c = c / 255.0
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def lin_to_srgb(c):
    c = np.clip(c, 0.0, 1.0)
    return np.where(c <= 0.0031308, c * 12.92, 1.055 * c ** (1 / 2.4) - 0.055) * 255.0


def hex_rgb(h):
    return np.array([int(h[i : i + 2], 16) for i in (0, 2, 4)], dtype=float)


def render(model, atlas):
    """The model at BIG px, RGBA: a z-buffered rasteriser, flat lit."""
    pos = np.array(model["positions"], dtype=float).reshape(-1, 3)
    col = np.array(model["colours"], dtype=float).reshape(-1, 3)
    uv = np.array(model["uvs"], dtype=float).reshape(-1, 2)
    tint = srgb_to_lin(hex_rgb(model["palette"][0]))
    # A three-quarter view from above: turned 35 degrees, tipped 30.
    yaw, pitch = math.radians(35.0), math.radians(30.0)
    ry = np.array(
        [[math.cos(yaw), 0, math.sin(yaw)], [0, 1, 0], [-math.sin(yaw), 0, math.cos(yaw)]]
    )
    rx = np.array(
        [[1, 0, 0], [0, math.cos(pitch), -math.sin(pitch)], [0, math.sin(pitch), math.cos(pitch)]]
    )
    view = pos @ ry.T @ rx.T
    lo, hi = view[:, :2].min(axis=0), view[:, :2].max(axis=0)
    span = max(hi - lo) or 1.0
    scale = (BIG - 6) / span
    centre = (lo + hi) / 2
    sx = (view[:, 0] - centre[0]) * scale + BIG / 2
    sy = BIG / 2 - (view[:, 1] - centre[1]) * scale
    depth = view[:, 2]
    light = np.array([-0.5, 0.8, 0.45])
    light /= np.linalg.norm(light)
    light_view = light @ ry.T @ rx.T
    img = np.zeros((BIG, BIG, 4))
    zbuf = np.full((BIG, BIG), -1e9)
    aw, ah = atlas.shape[1], atlas.shape[0]
    for t in range(0, len(pos) - 2, 3):
        a, b, c = t, t + 1, t + 2
        n = np.cross(view[b] - view[a], view[c] - view[a])
        norm = np.linalg.norm(n)
        if norm < 1e-12:
            continue
        n /= norm
        if n[2] < 0:
            n = -n
        shade = 0.45 + 0.55 * max(0.0, float(n @ light_view))
        x0 = int(max(0, math.floor(min(sx[a], sx[b], sx[c]))))
        x1 = int(min(BIG - 1, math.ceil(max(sx[a], sx[b], sx[c]))))
        y0 = int(max(0, math.floor(min(sy[a], sy[b], sy[c]))))
        y1 = int(min(BIG - 1, math.ceil(max(sy[a], sy[b], sy[c]))))
        den = (sy[b] - sy[c]) * (sx[a] - sx[c]) + (sx[c] - sx[b]) * (sy[a] - sy[c])
        if abs(den) < 1e-9:
            continue
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                px, py = x + 0.5, y + 0.5
                w0 = ((sy[b] - sy[c]) * (px - sx[c]) + (sx[c] - sx[b]) * (py - sy[c])) / den
                w1 = ((sy[c] - sy[a]) * (px - sx[c]) + (sx[a] - sx[c]) * (py - sy[c])) / den
                w2 = 1 - w0 - w1
                if w0 < 0 or w1 < 0 or w2 < 0:
                    continue
                z = w0 * depth[a] + w1 * depth[b] + w2 * depth[c]
                if z <= zbuf[y, x]:
                    continue
                zbuf[y, x] = z
                u = w0 * uv[a] + w1 * uv[b] + w2 * uv[c]
                tx = min(aw - 1, max(0, int(u[0] * aw)))
                ty = min(ah - 1, max(0, int(u[1] * ah)))
                texel = srgb_to_lin(atlas[ty, tx, :3].astype(float))
                vc = w0 * col[a] + w1 * col[b] + w2 * col[c]
                rgb = texel * vc * tint * shade
                img[y, x, :3] = lin_to_srgb(rgb)
                img[y, x, 3] = 255
    return img


def reduce(img):
    """BIG to SMALL by area average, kept where half is covered."""
    k = BIG // SMALL
    blocks = img.reshape(SMALL, k, SMALL, k, 4)
    alpha = (blocks[..., 3] > 0).mean(axis=(1, 3))
    rgb = np.zeros((SMALL, SMALL, 3))
    for y in range(SMALL):
        for x in range(SMALL):
            cell = blocks[y, :, x, :, :].reshape(-1, 4)
            cover = cell[cell[:, 3] > 0]
            if len(cover):
                rgb[y, x] = cover[:, :3].mean(axis=0)
    out = np.zeros((SMALL, SMALL, 4), dtype=np.uint8)
    keep = alpha >= 0.5
    out[keep, :3] = np.clip(rgb[keep], 0, 255).astype(np.uint8)
    out[keep, 3] = 255
    return out


def snap(icon, tones=5):
    """The opaque pixels' colours snapped to at most `tones` of their own."""
    mask = icon[..., 3] > 0
    if not mask.any():
        return icon
    pix = Image.fromarray(icon[..., :3])
    q = pix.quantize(colors=tones, method=Image.Quantize.MEDIANCUT).convert("RGB")
    out = icon.copy()
    out[..., :3] = np.where(mask[..., None], np.array(q), 0)
    return out


def outline(icon):
    """A 1 px dark outline round what is drawn (icons are UI)."""
    mask = icon[..., 3] > 0
    out = icon.copy()
    h, w = mask.shape
    for y in range(h):
        for x in range(w):
            if mask[y, x]:
                continue
            near = any(
                0 <= y + dy < h and 0 <= x + dx < w and mask[y + dy, x + dx]
                for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1))
            )
            if near:
                out[y, x] = OUTLINE
    return out


PAL = {
    # sack linen, the goods' ramp
    "1": (114, 82, 68),
    "2": (171, 132, 101),
    "3": (207, 170, 129),
    "4": (234, 213, 170),
    # the crushed coral in it
    "c": (221, 87, 109),
    "d": (234, 183, 152),
    "w": (255, 254, 254),
    # twine
    "t": (65, 50, 44),
    # bowl
    "b": (93, 119, 141),
    "B": (146, 163, 178),
    # mussel shells
    "m": (31, 59, 82),
    "M": (62, 88, 110),
    "o": (250, 175, 92),
    # kelp crisps
    "g": (41, 117, 55),
    "G": (85, 150, 66),
    "H": (155, 182, 117),
    "s": (250, 250, 240),
    # kelp: the kelp card's own colours
    "K": (114, 82, 68),
    "L": (171, 132, 101),
    "N": (207, 170, 129),
    "O": (197, 109, 37),
    "P": (85, 150, 66),
    "Q": (155, 182, 117),
}

# Kelp has no model of its own (it is the kelp card), so its icon is a
# frond drawn in the card's colours.
KELP_MAP = [
    "..........NL....",
    ".........NLLK...",
    "....LN..NLLK....",
    "...NLLK.LLK.....",
    "....LLKOK.......",
    ".....KOOK..NL...",
    "......KK..NLLK..",
    "...QP.K..NLLK...",
    "..QPPPK.LLLK....",
    "...PPKK.KK......",
    "......KOK.......",
    "......KOK.......",
    "......KK........",
    ".....KK.........",
    "....LKKL........",
    "................",
]

MAPS = {
    "fertilizer": [
        "................",
        "......t..t......",
        ".....dwccd......",
        "......tttt......",
        ".....233322.....",
        "....23333332....",
        "...2333443332...",
        "...2334cc4332...",
        "...234cdwc432...",
        "...2334cc4332...",
        "...2333443322...",
        "...2233333221...",
        "...1222222211...",
        "....11111111....",
        "................",
        "................",
    ],
    "steamed-mussels": [
        "................",
        "................",
        "................",
        ".....mM..Mm.....",
        "....mMMo.oMMm...",
        "...mMMo...oMMm..",
        "..BBBBBBBBBBBBB.",
        "..bBBBBBBBBBBBb.",
        "...bBBBBBBBBBb..",
        "...bbBBBBBBBbb..",
        "....bbbbbbbbb...",
        ".....bbbbbbb....",
        "................",
        "................",
        "................",
        "................",
    ],
    "kelp-crisps": [
        "................",
        "................",
        "................",
        ".....GHg........",
        "....gGHGg..GH...",
        "...gGGsGg.gHHg..",
        "....gGGg.gGGsGg.",
        "..GH.gg...gGGg..",
        ".gHHGg..GHg.gg..",
        ".gGsGg.gHHGg....",
        "..gGg..gGsGg....",
        "...g....gGg.....",
        "................",
        "................",
        "................",
        "................",
    ],
}


def pixel_map(rows):
    arr = np.zeros((SMALL, SMALL, 4), dtype=np.uint8)
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch != ".":
                arr[y, x, :3] = PAL[ch]
                arr[y, x, 3] = 255
    return arr


def main():
    models = json.loads(MODELS.read_text())
    atlas = np.array(Image.open(ATLAS).convert("RGB"))
    icons = {}
    for code, model in models.items():
        big = render(model, atlas)
        icons[code] = outline(snap(reduce(big)))
    icons["kelp"] = outline(pixel_map(KELP_MAP))
    for code, rows in MAPS.items():
        icons[code] = outline(pixel_map(rows))
    OUT.mkdir(parents=True, exist_ok=True)
    for code, arr in icons.items():
        Image.fromarray(arr, "RGBA").save(OUT / f"{code}.png")
        print("wrote", OUT / f"{code}.png")
    if "--sheet" in sys.argv:
        dest = Path(sys.argv[sys.argv.index("--sheet") + 1])
        n = len(icons)
        cols = 8
        rows = (n + cols - 1) // cols
        sheet = Image.new("RGBA", (cols * 80, rows * 80), (60, 56, 48, 255))
        for i, (code, arr) in enumerate(icons.items()):
            img = Image.fromarray(arr, "RGBA").resize((64, 64), Image.NEAREST)
            sheet.alpha_composite(img, ((i % cols) * 80 + 8, (i // cols) * 80 + 8))
        sheet.save(dest)
        print("sheet", dest)


if __name__ == "__main__":
    main()
