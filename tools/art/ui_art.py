"""The stone-and-carved-wood UI kit for Pale Blue Dot (openspec/changes/stone-and-wood-ui).

The owner, 2026-11-14: "develop ui for this game, do something like with rock and carved wood".

Every piece is pixel art drawn pixel by pixel at its native size with the balatro-pixel-art kit
(`.claude/skills/balatro-pixel-art/scripts/pixelkit.py`): every colour from Sweaft-64 (the kit's
save() refuses anything else), alpha 0 or 255, lit from the top left, a 1 px dark outline. Pieces
are drawn at native size and shown only at whole-number scales with nearest sampling.

A 9-slice piece follows the panel-textures rule: its corners are finished like a hero piece
(mitres, iron brackets, rivets, chips), its edges carry only detail that runs ALONG the edge (so
stretching them changes nothing) or, for the rough rock, detail periodic in exactly the edge's
length (so tiling them shows no seam), and its centre is one flat colour. `slices.json` records
each piece's size, its four margins and how its sides scale.

    python3 tools/art/ui_art.py                  # writes docs/mockups/ui/*.png and slices.json
    python3 tools/art/ui_art.py --sheet OUT.png  # also a 4x contact sheet
"""

import json
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
from pixelkit import SWEAFT_64, rgb, save, sheet  # noqa: E402
from PIL import Image  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "ui")

# ---------------------------------------------------------------------------------------------
# Tokens. Every colour is a Sweaft-64 entry; ramps run dark to light. These are the names the
# design (design.md, "Tokens") and the page's CSS use, and the names the Bevy palette module would
# take in place of menu.rs's INK / MINT / PANEL_FILL / EDGE.
# ---------------------------------------------------------------------------------------------
INK = "#010000"  # the outline round every piece
SEAM = "#170916"  # the dark seam between two materials, carved lines in wood
STONE = ["#011c34", "#1f3b52", "#3e586e", "#92a3b2", "#bfcad7"]  # dressed stone, slate blue (owner, 2026-11-14: "make the stone more blue")
ROCK = ["#010e1a", "#1f3b52", "#3e586e", "#5d778d", "#92a3b2"]  # rough rock, deep blue
WOOD = ["#2f1525", "#41322c", "#725244", "#ab8465", "#cfaa81"]  # carved oak
WOOD_DIM = ["#170916", "#2f1525", "#41322c", "#725244"]  # disabled or unlit wood
IRON = ["#010000", "#161612", "#2c2c25", "#434137", "#a09f91"]
BRASS = ["#733126", "#c56d25", "#fb902f", "#faaf5c"]
RUNE = ["#204f63", "#53998b", "#84e3b5", "#c1f1d8"]  # the one glowing accent: focus, selection
FACE = "#1f3b52"  # a panel's stone face, where text sits
WELL = "#011c34"  # the floor of a carved well (slots, grooves, map window)
TEXT = "#eab798"  # body text, warm bone
TEXT_DIM = "#cfaa81"  # secondary text
LABEL = "#010000"  # a label carved into an oak plaque or the compass strip
TITLE = "#faaf5c"  # carved titles, brass
HEALTH = ["#4a051e", "#a61e1f", "#d12d41", "#fa3a64"]
STAMINA = ["#733126", "#c56d25", "#fb902f", "#faaf5c"]
BREATH = ["#0c2f44", "#15578a", "#1096c3", "#74effe"]
PROGRESS = RUNE
RARITY = {
    "common": None,
    "uncommon": ["#05582d", "#559642", "#9bb675"],
    "rare": ["#15578a", "#1096c3", "#74effe"],
    "epic": ["#4f1d83", "#9e3ba2", "#c34988"],
    "legendary": ["#c56d25", "#fb902f", "#faaf5c"],
}

for _name, _v in list(globals().items()):
    if _name.isupper() and isinstance(_v, (str, list)) and _name not in ("OUT", "ROOT", "HERE"):
        for _c in [_v] if isinstance(_v, str) else _v:
            assert _c in SWEAFT_64, (_name, _c)


class Canvas:
    """An RGBA pixel grid. Colours are '#rrggbb' strings from the palette; None is transparent."""

    def __init__(self, w, h):
        self.w, self.h = w, h
        self.px = np.zeros((h, w, 4), np.uint8)

    def put(self, x, y, c):
        if 0 <= x < self.w and 0 <= y < self.h:
            if c is None:
                self.px[y, x] = 0
            else:
                self.px[y, x, :3] = rgb(c)
                self.px[y, x, 3] = 255

    def rect(self, x0, y0, x1, y1, c):
        """Fill x0..x1, y0..y1 inclusive."""
        for y in range(max(0, y0), min(self.h, y1 + 1)):
            for x in range(max(0, x0), min(self.w, x1 + 1)):
                self.put(x, y, c)

    def get(self, x, y):
        p = self.px[y, x]
        return None if p[3] == 0 else "#%02x%02x%02x" % tuple(p[:3])

    def ring(self, x0, y0, x1, y1, top, left, bottom, right, cut=False):
        """A one-pixel rectangle outline with a colour per side (top and left win the corners).
        cut=True leaves its four corner pixels alone."""
        for x in range(x0, x1 + 1):
            if cut and x in (x0, x1):
                continue
            self.put(x, y1, bottom)
            self.put(x, y0, top)
        for y in range(y0, y1 + 1):
            if cut and y in (y0, y1):
                continue
            self.put(x1, y, right)
            self.put(x0, y, left)

    def image(self):
        return Image.fromarray(self.px, "RGBA")


def rivet(c, x, y, ramp=IRON):
    """A forged rivet head, 2 x 2: lit top left, dark bottom right."""
    c.put(x, y, ramp[4])
    c.put(x + 1, y, ramp[3])
    c.put(x, y + 1, ramp[3])
    c.put(x + 1, y + 1, ramp[1])


def outline(c, cut=True):
    """1 px INK round the whole piece, its corner pixels clipped so the silhouette reads rounded."""
    c.ring(0, 0, c.w - 1, c.h - 1, INK, INK, INK, INK)
    if cut:
        for x, y in ((0, 0), (c.w - 1, 0), (0, c.h - 1), (c.w - 1, c.h - 1)):
            c.put(x, y, None)


# ---------------------------------------------------------------------------------------------
# Wood moulding: a carved profile that runs along every edge. Profile index 0 is the outside.
# ---------------------------------------------------------------------------------------------
def moulding(c, x0, y0, x1, y1, profile, lit, shade):
    """Draw nested rings from the outside in, one per profile entry. Each entry is a ramp index;
    the top and left sides take lit[i], the bottom and right shade[i] (the light is top left)."""
    for i, _ in enumerate(profile):
        c.ring(x0 + i, y0 + i, x1 - i, y1 - i, lit[i], lit[i], shade[i], shade[i])


def mitres(c, x0, y0, x1, y1, depth, colour):
    """The carved mitre joint at each corner of a moulding `depth` pixels deep: a dark diagonal."""
    for i in range(depth):
        c.put(x0 + i, y0 + i, colour)
        c.put(x1 - i, y0 + i, colour)
        c.put(x0 + i, y1 - i, colour)
        c.put(x1 - i, y1 - i, colour)


def bracket(c, x, y, sx, sy, n=6):
    """An iron corner bracket: an L of iron two pixels thick along both edges from the corner at
    (x, y), running n pixels in the directions sx, sy (+1 or -1), with a rivet on each arm."""
    for i in range(n):
        for t in range(2):
            col = IRON[3] if (t == 0) else IRON[2]
            if sx < 0 or sy < 0:
                col = IRON[2] if t == 0 else IRON[1]
            c.put(x + sx * i, y + sy * t, col)
            c.put(x + sx * t, y + sy * i, col)
    c.put(x, y, IRON[4] if sx > 0 and sy > 0 else IRON[3])
    rx = x + sx * (n - 2) - (1 if sx < 0 else 0)
    ry = y + sy * (n - 2) - (1 if sy < 0 else 0)
    rivet(c, rx, y if sy > 0 else y - 1)
    rivet(c, x if sx > 0 else x - 1, ry)


def panel_frame():
    """The menu panel: a carved oak moulding round a dressed-stone face. 48 x 48, slices 14.

    Outside in: 1 px outline; a 7 px oak moulding (a lit bead, the flat, a carved groove, the
    flat, a dark fillet) with mitred corners and an iron bracket on each corner; a 1 px seam; a
    4 px stone lip sunk into the frame (its top and left in the frame's shadow); the flat face."""
    c = Canvas(48, 48)
    lit = [WOOD[4], WOOD[3], WOOD[3], SEAM, WOOD[3], WOOD[2], WOOD[1]]
    shade = [WOOD[2], WOOD[2], WOOD[2], SEAM, WOOD[2], WOOD[1], WOOD[0]]
    moulding(c, 1, 1, 46, 46, lit, lit, shade)
    mitres(c, 1, 1, 46, 46, 7, SEAM)
    c.ring(8, 8, 39, 39, SEAM, SEAM, SEAM, SEAM)
    stone_lit = [STONE[0], STONE[1], STONE[2], STONE[2]]
    stone_shade = [STONE[3], STONE[3], STONE[2], STONE[2]]
    for i in range(4):
        c.ring(9 + i, 9 + i, 38 - i, 38 - i, stone_lit[i], stone_lit[i], stone_shade[i], stone_shade[i])
    c.rect(13, 13, 34, 34, FACE)
    outline(c)
    bracket(c, 1, 1, 1, 1)
    bracket(c, 46, 1, -1, 1)
    bracket(c, 1, 46, 1, -1)
    bracket(c, 46, 46, -1, -1)
    return c


def rock_panel():
    """The rough-rock slab, for what stands in the world (the loading screen, the map's frame):
    48 x 48, slices 14. Its silhouette is chipped; the chips in the edges repeat every 20 px, the
    edge's own length, so its sides TILE without a seam (slices.json says so), and the corners
    carry the bigger breaks."""
    W = 48
    c = Canvas(W, W)
    # Inward offset of the silhouette along an edge, index 0..47. 14..33 is the tiling edge: the
    # pattern there is one period of 20 and is 0 at both its ends, so corner meets edge cleanly.
    edge = [0] * 20
    for i, d in ((4, 1), (5, 1), (11, 1), (12, 2), (13, 1)):
        edge[i] = d
    corner_l = [3, 2, 2, 1, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0]
    corner_r = [0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 1, 2, 2, 3]
    prof = corner_l + edge + corner_r
    inside = np.zeros((W, W), bool)
    for y in range(W):
        for x in range(W):
            if (y >= prof[x] and y <= W - 1 - prof[W - 1 - x] and x >= prof[y]
                    and x <= W - 1 - prof[W - 1 - y]):
                inside[y, x] = True
    # Body: rock blocks, lit top-left by a distance-from-edge ramp, with cracks.
    for y in range(W):
        for x in range(W):
            if inside[y, x]:
                c.put(x, y, ROCK[2])
    # Edge light and shade: pixels whose upper or left neighbour is outside are lit, lower/right shaded.
    for y in range(W):
        for x in range(W):
            if not inside[y, x]:
                continue
            up = y == 0 or not inside[y - 1, x]
            lf = x == 0 or not inside[y, x - 1]
            dn = y == W - 1 or not inside[y + 1, x]
            rt = x == W - 1 or not inside[y, x + 1]
            up2 = y <= 1 or not inside[y - 2, x]
            lf2 = x <= 1 or not inside[y, x - 2]
            if dn or rt:
                c.put(x, y, ROCK[1])
            elif up or lf:
                c.put(x, y, ROCK[4])
            elif up2 or lf2:
                c.put(x, y, ROCK[3])
    # The carved face: a sunk bevel then the flat face, the same as the wooden frame's lip.
    for i, (tl, br) in enumerate(((ROCK[0], ROCK[3]), (ROCK[1], ROCK[3]), (ROCK[1], ROCK[2]))):
        c.ring(9 + i, 9 + i, 38 - i, 38 - i, tl, tl, br, br)
    c.rect(12, 12, 35, 35, FACE)
    # Strata lines in the rim: along the edges only (so they stretch or tile cleanly), broken in
    # the corners into cracks.
    for x in range(14, 34):
        if (x - 14) % 20 in (2, 3, 4, 7, 8, 15, 16, 17):
            c.put(x, 5, ROCK[1])
            c.put(x, 42, ROCK[1])
    for y in range(14, 34):
        if (y - 14) % 20 in (1, 2, 3, 9, 10, 14, 15, 16):
            c.put(5, y, ROCK[1])
            c.put(42, y, ROCK[1])
    for x, y in ((4, 4), (5, 5), (6, 5), (7, 6), (41, 3), (41, 4), (42, 5), (5, 41), (6, 41), (6, 42),
                 (40, 42), (41, 41), (42, 41), (43, 40)):
        c.put(x, y, ROCK[0])
    for x, y in ((4, 3), (40, 3), (5, 40), (40, 41)):
        c.put(x, y, ROCK[4])
    # Outline round the silhouette.
    for y in range(W):
        for x in range(W):
            if inside[y, x]:
                continue
            nb = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
            if any(0 <= a < W and 0 <= b < W and inside[b, a] for a, b in nb):
                c.put(x, y, INK)
    for y in range(W):
        for x in range(W):
            if inside[y, x] and (x in (0, W - 1) or y in (0, W - 1)):
                c.put(x, y, INK)
    return c


def slot(state="idle", rarity="common"):
    """A slot: a block of dressed stone with a well carved into it. 24 x 24; the well's floor is
    16 x 16 (4..19), where a 16 px item icon sits pixel for pixel."""
    c = Canvas(24, 24)
    body = STONE[2]
    lit, shade = STONE[3], STONE[1]
    if state == "hover":
        lit, body = STONE[4], STONE[3]
    if state == "disabled":
        lit, body, shade = STONE[2], STONE[1], STONE[0]
    c.rect(1, 1, 22, 22, body)
    c.ring(1, 1, 22, 22, lit, lit, shade, shade)
    c.ring(2, 2, 21, 21, body, body, body, body)
    # The well's rim: in shadow at the top and left, lit at the bottom and right (it is sunk).
    c.ring(3, 3, 20, 20, STONE[0], STONE[0], lit, lit)
    c.rect(4, 4, 19, 19, WELL)
    c.put(4, 4, STONE[0])
    if state == "selected":
        # A rune cut round the block and glowing: the one emissive colour in the kit.
        c.ring(2, 2, 21, 21, RUNE[2], RUNE[2], RUNE[1], RUNE[1])
        c.ring(1, 1, 22, 22, RUNE[3], RUNE[3], RUNE[2], RUNE[2])
    gem = RARITY.get(rarity)
    if gem:
        for (x, y) in ((1, 1), (21, 1), (1, 21), (21, 21)):
            c.put(x, y, gem[2])
            c.put(x + 1, y, gem[1])
            c.put(x, y + 1, gem[1])
            c.put(x + 1, y + 1, gem[0])
        c.ring(3, 3, 20, 20, gem[0], gem[0], gem[1], gem[1])
    if state == "disabled":
        for y in range(4, 20):
            for x in range(4, 20):
                if (x + y) % 2 == 0:
                    c.put(x, y, STONE[1])
    outline(c)
    return c


def plaque(state="idle", w=48, h=22):
    """A button: a carved oak plaque with a groove round its face and a rivet in each corner.
    48 x 22 (44 px tall at 2x: a touch target), slices 8. States: idle, hover, pressed, disabled,
    focus."""
    c = Canvas(w, h)
    ramp = WOOD
    if state == "disabled":
        ramp = WOOD_DIM + [WOOD_DIM[-1]]
    hi, body, sh, deep = ramp[4], ramp[3], ramp[2], ramp[1]
    if state == "idle" or state == "focus":
        hi, body, sh, deep = WOOD[4], WOOD[3], WOOD[2], WOOD[1]
    if state == "hover":
        hi, body, sh, deep = "#eab798", WOOD[4], WOOD[3], WOOD[2]
    if state == "disabled":
        hi, body, sh, deep = WOOD_DIM[3], WOOD_DIM[2], WOOD_DIM[1], WOOD_DIM[0]
    top = 2 if state == "pressed" else 1
    if state == "pressed":
        hi, body, sh, deep = WOOD[3], WOOD[3], WOOD[2], WOOD[1]
    c.rect(1, top, w - 2, h - 2, body)
    c.ring(1, top, w - 2, h - 2, hi, hi, deep, deep)
    if state == "pressed":
        c.ring(1, 1, w - 2, 1, INK, INK, INK, INK)
        c.ring(2, top + 1, w - 3, top + 1, deep, deep, deep, deep)
    c.ring(2, top + 1, w - 3, h - 3, body, body, sh, sh) if state != "pressed" else None
    # The carved groove: a dark line with a lit line below and right of it.
    g0, g1 = 4, top + 3
    c.ring(g0, g1, w - 1 - g0, h - 1 - 4, SEAM, SEAM, hi if state != "disabled" else body,
           hi if state != "disabled" else body, cut=True)
    rv = BRASS + [BRASS[3]] if state == "hover" else IRON
    if state == "disabled":
        rv = [INK, WOOD_DIM[0], WOOD_DIM[1], WOOD_DIM[2], WOOD_DIM[3]]
    for x, y in ((2, top + 1), (w - 4, top + 1), (2, h - 5), (w - 4, h - 5)):
        if rv is IRON:
            rivet(c, x, y)
        else:
            c.put(x, y, rv[3])
            c.put(x + 1, y, rv[2])
            c.put(x, y + 1, rv[2])
            c.put(x + 1, y + 1, rv[1])
    outline(c)
    if state == "pressed":
        for x in range(w):
            c.put(x, 0, None)
        c.ring(0, 1, w - 1, h - 1, INK, INK, INK, INK)
        for x, y in ((0, 1), (w - 1, 1), (0, h - 1), (w - 1, h - 1)):
            c.put(x, y, None)
    if state == "focus":
        c.ring(1, 1, w - 2, h - 2, RUNE[3], RUNE[3], RUNE[2], RUNE[2], cut=True)
    return c


def tab(active=True):
    """A tab, 40 x 18, slices 7: oak like the plaques. The open tab stands 2 px taller and has no
    foot, so it joins the frame's moulding under it; a shut tab is darker and has a foot."""
    c = Canvas(40, 18)
    if active:
        hi, body, sh = WOOD[4], WOOD[3], WOOD[2]
        y0 = 0
    else:
        hi, body, sh = WOOD[3], WOOD[2], WOOD[1]
        y0 = 2
    c.rect(1, y0 + 1, 38, 17, body)
    for x in range(1, 39):
        c.put(x, y0 + 1, hi)
    for y in range(y0 + 1, 18):
        c.put(1, y, hi)
        c.put(38, y, sh)
    for x in range(2, 38):
        c.put(x, y0, INK)
    for y in range(y0 + 1, 18):
        c.put(0, y, INK)
        c.put(39, y, INK)
    c.put(1, y0, None)
    c.put(38, y0, None)
    # The carved groove under the top edge.
    for x in range(4, 36):
        c.put(x, y0 + 3, SEAM)
        c.put(x, y0 + 4, hi)
    if not active:
        for x in range(0, 40):
            c.put(x, 17, INK)
            c.put(x, 16, SEAM)
    return c


def groove(w, h):
    """A groove cut into stone (a scroll track or a slider's channel), slices 4."""
    c = Canvas(w, h)
    c.rect(1, 1, w - 2, h - 2, WELL)
    c.ring(1, 1, w - 2, h - 2, STONE[0], STONE[0], STONE[3], STONE[3])
    outline(c)
    return c


def scroll_thumb():
    """The scroll thumb: a short oak block with an iron band at each end. 8 x 16, slices 4."""
    c = Canvas(8, 16)
    c.rect(1, 1, 6, 14, WOOD[3])
    c.ring(1, 1, 6, 14, WOOD[4], WOOD[4], WOOD[1], WOOD[1])
    for y in (2, 13):
        for x in range(1, 7):
            c.put(x, y, IRON[3] if y == 2 else IRON[2])
    outline(c)
    return c


def slider_knob():
    """The slider's knob: an oak handle capped with iron, 9 x 13."""
    c = Canvas(9, 13)
    c.rect(1, 1, 7, 11, WOOD[3])
    c.ring(1, 1, 7, 11, WOOD[4], WOOD[4], WOOD[1], WOOD[1])
    c.rect(1, 1, 7, 3, IRON[3])
    c.ring(1, 1, 7, 3, IRON[4], IRON[4], IRON[2], IRON[2])
    for y in range(5, 10, 2):
        c.put(3, y, SEAM)
        c.put(4, y, SEAM)
        c.put(5, y, SEAM)
        c.put(4, y + 1, WOOD[4])
    outline(c)
    return c


def checkbox(on):
    """A checkbox: a small well in stone; ticked, a rune is cut in it and glows. 12 x 12."""
    c = Canvas(12, 12)
    c.rect(1, 1, 10, 10, STONE[2])
    c.ring(1, 1, 10, 10, STONE[3], STONE[3], STONE[1], STONE[1])
    c.ring(2, 2, 9, 9, STONE[0], STONE[0], STONE[3], STONE[3])
    c.rect(3, 3, 8, 8, WELL)
    if on:
        for x, y in ((3, 6), (4, 7), (5, 8), (6, 7), (7, 6), (8, 5), (8, 4), (7, 5), (6, 6), (5, 7),
                     (4, 6)):
            c.put(x, y, RUNE[2])
        c.put(8, 3, RUNE[3])
        c.put(3, 5, RUNE[3])
    outline(c)
    return c


def tooltip():
    """A tooltip: a thin oak frame round the stone face, 24 x 18, slices 6."""
    c = Canvas(24, 18)
    c.ring(1, 1, 22, 16, WOOD[4], WOOD[4], WOOD[2], WOOD[2])
    c.ring(2, 2, 21, 15, WOOD[3], WOOD[3], WOOD[2], WOOD[2])
    c.ring(3, 3, 20, 14, SEAM, SEAM, SEAM, SEAM)
    c.ring(4, 4, 19, 13, STONE[0], STONE[0], STONE[2], STONE[2])
    c.rect(5, 5, 18, 12, FACE)
    outline(c)
    for x, y in ((2, 2), (20, 2), (2, 14), (20, 14)):
        c.put(x, y, IRON[3])
        c.put(x + 1, y + 1, IRON[1])
    return c


def bar_frame():
    """A bar's frame: an iron band with a dark channel and a rivet at each end. 40 x 10, slices 4.
    Its channel is 6 rows high (2..7) and runs from x 4 to 35."""
    c = Canvas(40, 10)
    c.rect(1, 1, 38, 8, IRON[2])
    c.ring(1, 1, 38, 8, IRON[4], IRON[4], IRON[1], IRON[1])
    c.rect(4, 2, 35, 7, WELL)
    c.ring(4, 2, 35, 7, IRON[0], IRON[0], IRON[3], IRON[3])
    c.rect(5, 3, 34, 6, WELL)
    rivet(c, 2, 4)
    rivet(c, 36, 4)
    outline(c)
    return c


def bar_fill(ramp):
    """A bar's fill, 2 x 4: lit top row, body, shaded foot. It stretches along x only."""
    c = Canvas(2, 4)
    for x in range(2):
        c.put(x, 0, ramp[3])
        c.put(x, 1, ramp[2])
        c.put(x, 2, ramp[2])
        c.put(x, 3, ramp[1])
    return c


def compass_strip():
    """The compass bar as a carved oak strip: 80 x 18, slices 9 top and bottom, 10 at the ends (it stretches along x only). Lighter oak than the panels, so
    the carved letters and ticks (dark) read on it; a groove along the foot carries the towns'
    diamonds; an iron band and a carved rosette at each end."""
    c = Canvas(80, 18)
    c.rect(1, 1, 78, 16, WOOD[3])
    c.ring(1, 1, 78, 16, WOOD[4], WOOD[4], WOOD[1], WOOD[1])
    c.ring(2, 2, 77, 15, WOOD[3], WOOD[3], WOOD[2], WOOD[2])
    # The groove along the foot.
    for x in range(10, 70):
        c.put(x, 12, SEAM)
        c.put(x, 13, WOOD[2])
        c.put(x, 14, WOOD[4])
    # Iron end bands and rosettes.
    for x0 in (3, 72):
        for y in range(2, 16):
            c.put(x0, y, IRON[3])
            c.put(x0 + 1, y, IRON[2])
            c.put(x0 + 4, y, IRON[2])
        c.put(x0 + 2, 7, SEAM)
        c.put(x0 + 2, 9, SEAM)
        c.put(x0 + 1, 8, SEAM)
        c.put(x0 + 3, 8, SEAM)
        c.put(x0 + 2, 8, BRASS[3])
    outline(c)
    return c


def compass_pointer():
    """The iron pointer that hangs over the strip's middle, 7 x 5."""
    c = Canvas(7, 5)
    rows = ["IIIIIII", "IhmmmdI", ".IhmdI.", "..IdI..", "...I..."]
    cols = {"I": INK, "h": IRON[4], "m": IRON[3], "d": IRON[2]}
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch != ".":
                c.put(x, y, cols[ch])
    return c


def diamond(ramp=BRASS):
    """A town's mark on the compass, 5 x 5, a brass stud."""
    c = Canvas(5, 5)
    rows = [".IaI.", "IbabI", "abcbd", "IbddI", ".IdI."]
    cols = {"I": INK, "a": ramp[3], "b": ramp[2], "c": "#fffefe", "d": ramp[1]}
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch != ".":
                c.put(x, y, cols[ch])
    return c


def map_frame():
    """The map's frame: 64 x 64, slices 22. Rough-rock corner blocks with a rune ring carved in
    each, an oak moulding along the sides between them, a brass fillet, then the map window."""
    c = Canvas(64, 64)
    lit = [WOOD[4], WOOD[3], SEAM, WOOD[3], WOOD[2], WOOD[1], SEAM]
    shade = [WOOD[2], WOOD[2], SEAM, WOOD[2], WOOD[1], WOOD[0], SEAM]
    moulding(c, 1, 1, 62, 62, lit, lit, shade)
    for i, col in enumerate((BRASS[3], BRASS[1])):
        c.ring(8 + i, 8 + i, 55 - i, 55 - i, col if i == 0 else BRASS[2], col if i == 0 else BRASS[2],
               BRASS[1], BRASS[1])
    c.ring(10, 10, 53, 53, INK, INK, INK, INK)
    c.rect(11, 11, 52, 52, WELL)
    for (x0, y0) in ((0, 0), (46, 0), (0, 46), (46, 46)):
        blk = Canvas(18, 18)
        blk.rect(1, 1, 16, 16, ROCK[2])
        blk.ring(1, 1, 16, 16, ROCK[4], ROCK[4], ROCK[1], ROCK[1])
        blk.ring(2, 2, 15, 15, ROCK[3], ROCK[3], ROCK[2], ROCK[2])
        # A carved ring with a dot: the planet's own mark.
        for a in range(0, 360, 10):
            r = 5
            x = int(round(8.5 + r * np.cos(np.radians(a))))
            y = int(round(8.5 + r * np.sin(np.radians(a))))
            blk.put(x, y, ROCK[0])
        for a in range(200, 300, 10):
            x = int(round(8.5 + 6 * np.cos(np.radians(a))))
            y = int(round(8.5 + 6 * np.sin(np.radians(a))))
            blk.put(x, y, ROCK[1])
        for a in range(20, 120, 10):
            x = int(round(8.5 + 6 * np.cos(np.radians(a))))
            y = int(round(8.5 + 6 * np.sin(np.radians(a))))
            blk.put(x, y, ROCK[4])
        blk.rect(8, 8, 9, 9, RUNE[2])
        blk.put(8, 8, RUNE[3])
        outline(blk)
        c.px[y0:y0 + 18, x0:x0 + 18][blk.px[..., 3] > 0] = blk.px[blk.px[..., 3] > 0]
    return c


def portrait_frame():
    """The speaker's portrait: a stone well with an oak rim, 32 x 32, slices 8."""
    c = Canvas(32, 32)
    c.ring(1, 1, 30, 30, WOOD[4], WOOD[4], WOOD[1], WOOD[1])
    c.ring(2, 2, 29, 29, WOOD[3], WOOD[3], WOOD[2], WOOD[2])
    c.ring(3, 3, 28, 28, SEAM, SEAM, SEAM, SEAM)
    c.ring(4, 4, 27, 27, STONE[0], STONE[0], STONE[3], STONE[3])
    c.rect(5, 5, 26, 26, WELL)
    outline(c)
    rivet(c, 2, 2)
    rivet(c, 28, 2)
    rivet(c, 2, 28)
    rivet(c, 28, 28)
    return c


def emblem():
    """The loading screen's emblem: a stone roundel with the pale blue dot set in it. 48 x 48."""
    c = Canvas(48, 48)
    cx = cy = 23.5
    for y in range(48):
        for x in range(48):
            d = np.hypot(x - cx, y - cy)
            ang = np.degrees(np.arctan2(y - cy, x - cx))
            lit = -135 - 60 < ang < -135 + 60 or ang > 135 + 30
            if d <= 22.5:
                col = ROCK[2]
                if d > 21:
                    col = ROCK[4] if lit else ROCK[1]
                elif d > 19.5:
                    col = ROCK[3] if lit else ROCK[2]
                elif d > 18.5:
                    col = ROCK[0]  # the carved ring
                elif d > 17.5:
                    col = ROCK[3] if not lit else ROCK[1]
                elif d > 11:
                    col = ROCK[2]
                elif d > 10:
                    col = ROCK[0]
                elif d > 9:
                    col = ROCK[1] if lit else ROCK[3]
                else:
                    col = WELL
                c.put(x, y, col)
    # Ticks round the ring: the hours of a day on world time.
    for k in range(12):
        a = np.radians(k * 30 - 90)
        for r in (13.5, 14.5, 15.5):
            c.put(int(round(cx + r * np.cos(a))), int(round(cy + r * np.sin(a))), ROCK[0] if k % 3 else RUNE[1])
    # The pale blue dot: a sphere in the well, lit from the top left.
    for y in range(48):
        for x in range(48):
            d = np.hypot(x - cx, y - cy)
            if d <= 6.2:
                dl = np.hypot(x - (cx - 2), y - (cy - 2))
                col = "#1096c3"
                if dl < 2.2:
                    col = "#d2f8ff"
                elif dl < 4.2:
                    col = "#74effe"
                if d > 5.2:
                    col = "#15578a"
                c.put(x, y, col)
    solid = c.px[..., 3] > 0
    for y in range(48):
        for x in range(48):
            if not solid[y, x]:
                nb = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
                if any(0 <= a < 48 and 0 <= b < 48 and solid[b, a] for a, b in nb):
                    c.put(x, y, INK)
    return c


ICONS = {
    # 11 x 11 marks carved into oak: '#' the cut (dark), '+' the lit lower-right lip of the cut.
    "close": ["...........", ".##.....##.", ".###...###+", "..###.###+.", "...#####+..",
              "....###+...", "...#####...", "..###+###..", ".###+..###.", ".##+....##+", "..+......+."],
    "back": ["...........", "....##.....", "...##+.....", "..##+......", ".#########.",
             "##########+", ".##+++++++.", "..##.......", "...##......", "....##.....", ".....+....."],
    "gear": ["....###....", ".#.#####.#.", "..#######+.", ".###+++###.", "####+..####",
             "####...####", "####+..###+", ".####+###+.", "..#######..", ".#.#####.#.", "....###+..."],
    "map": ["...........", ".##..##..#.", ".#.##..##.#", ".#..#...#.#", ".#..#...#.#",
            ".#..#...#.#", ".#..#...#.#", ".#..#...#.#", ".##..##..##", "..+++.++++.", "..........."],
    "pack": ["...#####...", "..##+++##..", "..#.....#+.", ".#########.", ".#.......#+",
             ".#..###..#+", ".#..#+#..#+", ".#..###..#+", ".#.......#+", ".#########+", "..+++++++++"],
}


def icon(name):
    rows = ICONS[name]
    c = Canvas(11, 11)
    for y, r in enumerate(rows):
        for x, ch in enumerate(r):
            if ch == "#":
                c.put(x, y, SEAM)
            elif ch == "+":
                c.put(x, y, WOOD[4])
    return c


PIECES = {}


def add(name, canvas, slices=None, sides="stretch", note=""):
    PIECES[name] = (canvas, slices, sides, note)


def build():
    add("frame", panel_frame(), 14, note="menu panel: oak moulding round a dressed-stone face")
    add("rock", rock_panel(), 14, "tile", note="rough-rock slab; sides tile (period 20 px)")
    for st in ("idle", "hover", "selected", "disabled"):
        add(f"slot-{st}", slot(st), None, note="hotbar and pack slot, 16 px icon well at 4,4")
    for r in ("uncommon", "rare", "epic", "legendary"):
        add(f"slot-{r}", slot("idle", r), None, note=f"item frame, {r}")
    for st in ("idle", "hover", "pressed", "disabled", "focus"):
        add(f"button-{st}", plaque(st), 8, note="carved oak plaque")
    add("tab-open", tab(True), 7)
    add("tab-shut", tab(False), 7)
    add("scroll-track", groove(8, 24), 4)
    add("scroll-thumb", scroll_thumb(), 4)
    add("slider-track", groove(32, 8), 3)
    add("slider-knob", slider_knob(), None)
    add("check-off", checkbox(False), None)
    add("check-on", checkbox(True), None)
    add("tooltip", tooltip(), 6)
    add("bar", bar_frame(), 4, note="channel x 4..35, y 2..7; fill drawn in x 5..34, y 3..6")
    add("fill-health", bar_fill(HEALTH), None)
    add("fill-stamina", bar_fill(STAMINA), None)
    add("fill-breath", bar_fill(BREATH), None)
    add("fill-progress", bar_fill(PROGRESS), None)
    add("compass", compass_strip(), [9, 10, 9, 10], note="slices top, right, bottom, left: it stretches along x only")
    add("compass-pointer", compass_pointer(), None)
    add("compass-town", diamond(), None)
    add("map-frame", map_frame(), 22)
    add("portrait", portrait_frame(), 8)
    add("emblem", emblem(), None)
    for n in ICONS:
        add(f"icon-{n}", icon(n), None)


def contrast(a, b):
    def lum(h):
        v = [c / 255 for c in rgb(h)]
        v = [x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4 for x in v]
        return 0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2]

    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


def main():
    build()
    os.makedirs(OUT, exist_ok=True)
    meta = {}
    for name, (cv, sl, sides, note) in PIECES.items():
        img = cv.image()
        save(img, os.path.join(OUT, f"{name}.png"), source="tools/art/ui_art.py")
        meta[name] = {"w": img.width, "h": img.height}
        if sl is not None:
            meta[name].update({"slice": sl, "sides": sides, "centre": "stretch"})
        if note:
            meta[name]["note"] = note
    tokens = {k: globals()[k] for k in ("INK", "SEAM", "FACE", "WELL", "TEXT", "TEXT_DIM", "TITLE", "LABEL")}
    ramps = {k: globals()[k] for k in ("STONE", "ROCK", "WOOD", "IRON", "BRASS", "RUNE", "HEALTH",
                                       "STAMINA", "BREATH")}
    checks = {
        "TEXT on FACE": round(contrast(TEXT, FACE), 2),
        "TEXT_DIM on FACE": round(contrast(TEXT_DIM, FACE), 2),
        "TITLE on FACE": round(contrast(TITLE, FACE), 2),
        "TEXT on WELL": round(contrast(TEXT, WELL), 2),
        "SEAM on WOOD[3] (compass letters)": round(contrast(SEAM, WOOD[3]), 2),
        "LABEL on WOOD[3] (button label)": round(contrast(LABEL, WOOD[3]), 2),
        "LABEL on WOOD[4] (hovered button label)": round(contrast(LABEL, WOOD[4]), 2),
        "TEXT_DIM on WOOD_DIM[2] (disabled label)": round(contrast(TEXT_DIM, WOOD_DIM[2]), 2),
        "RUNE[2] on WELL (focus ring)": round(contrast(RUNE[2], WELL), 2),
    }
    with open(os.path.join(OUT, "slices.json"), "w") as f:
        json.dump({"source": "tools/art/ui_art.py", "palette": "Sweaft-64", "pieces": meta,
                   "tokens": tokens, "ramps": ramps, "contrast": checks}, f, indent=1)
    for k, v in checks.items():
        print(f"{k}: {v}:1")
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1]
        items = [(n, PIECES[n][0].image()) for n in PIECES]
        sheet(items, scale=4, cols=8, bg="#1f3b52").save(out)
        print("sheet", out)
    print(len(PIECES), "pieces ->", OUT)


if __name__ == "__main__":
    main()
