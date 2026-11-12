"""Mockup-only pixel art for the foliage playground (docs/mockups/foliage.html).

Pixel art drawn in code with the balatro-pixel-art kit
(.claude/skills/balatro-pixel-art/scripts/pixelkit.py), the solitaire
collection's style: a tiny native canvas, chunky shapes, 3 to 5 tone ramps lit
from the top left, seams between parts, a 1 px outline, single-pixel glints,
alpha 0 or 255, every colour from Sweaft-64. The owner, 2026-11-12: "the card
game rubens solitaire collection uses a different art style, can you adopt that
art style instead?"

Only what the game has no texture for is drawn here (the owner, the same day:
"why redraw tiles we already have texture for?"). Ground, bark, leaves,
needles, grass, rock, litter, reed and the alien sheets are the game's own
assets/tilesets/atlas.png and assets/textures/settlement/, copied byte for byte
into docs/mockups/foliage/game/; leaf, needle and frond card shapes are cut
from those tiles by a pixel mask in the page's shader. What is left:

- tiles, 32 x 32 like the game's settlement textures: bark_birch, bark_palm;
  16 x 16: cap_red, cap_brown (mushroom caps);
- cards, 16 x 16 on a clear ground: flower_red, flower_yellow, flower_blue,
  flower_white, flower_glow;
- impostors, 32 x 48: imp_<species>, the far-LOD tree standing on the bottom edge, and
  imp_<oak|birch|willow>_bare, the same trees in winter: a dark crown of twigs;
- icons, 56 x 56: icon_<species>, the same tree on a mound, for the biome picker.

    python tools/art/foliage_art.py                 # writes docs/mockups/foliage/*.png
    python tools/art/foliage_art.py --sheet S.png   # also a 4x contact sheet with the kit's references
"""

import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts")
sys.path.insert(0, KIT)
from pixelkit import Sprite, RAMPS, INK, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "foliage")

# Ramps beyond the kit's, dark to light, every entry Sweaft-64.
LEAF = ["#003227", "#05582d", "#297537", "#559642", "#9bb675"]
PINE = ["#010e1a", "#003227", "#05582d", "#297537"]
BIRCHLEAF = ["#297537", "#559642", "#9bb675", "#c1f1d8"]
ACACIA = ["#2c2c25", "#05582d", "#297537", "#9bb675"]
WILLOW = ["#003227", "#297537", "#559642", "#9bb675"]
BARK = ["#2e0d06", "#41322c", "#725244", "#ab8465"]
PALE = ["#5d778d", "#92a3b2", "#bfcad7", "#fffefe"]
KELP = ["#2c2c25", "#434137", "#05582d", "#297537", "#9bb675"]
CACTUS = ["#003227", "#05582d", "#297537", "#559642", "#9bb675"]


# ---------- tiles ----------------------------------------------------------------------------------

def tile_birch():
    sp = Sprite(32, 32)
    sp.fill(sp.rect(0, 0, 31, 31), "#bfcad7")
    rnd = random.Random(4)
    for x in range(0, 32, 4):       # faint vertical grain
        sp.fill(sp.rect(x, 0, x, 31), "#fffefe" if x % 8 else "#92a3b2")
    for _ in range(9):              # the black lenticels, kept off the edges so the tile wraps
        x, y, w = rnd.randrange(1, 24), rnd.randrange(1, 30), rnd.randrange(3, 7)
        sp.fill(sp.rect(x, y, x + w, y + 1), INK)
        sp.fill(sp.rect(x + 1, y, x + w - 1, y), "#2c2c25")
    return sp


def tile_palm():
    sp = Sprite(32, 32)
    for k in range(8):              # stacked leaf scars, lit on top
        y = k * 4
        sp.fill(sp.rect(0, y, 31, y + 3), "#725244" if k % 2 else "#ab8465")
        sp.fill(sp.rect(0, y, 31, y), "#cfaa81")
        sp.fill(sp.rect(0, y + 3, 31, y + 3), "#41322c")
    for x in range(2, 32, 6):
        sp.plots([(x, y) for y in range(1, 32, 4)], "#41322c")
    return sp


def tile_cap(body, spot, deep):
    sp = Sprite(16, 16)
    sp.fill(sp.rect(0, 0, 15, 15), body)
    for x, y in ((2, 2), (9, 3), (5, 8), (12, 10), (1, 12), (8, 13)):
        sp.fill(sp.rect(x, y, x + 1, y + 1), spot)
        sp.plot(x + 1, y + 2, deep)
    return sp


# ---------- flower heads ---------------------------------------------------------------------------

def flower(petal_ramp, centre_ramp, n=5, r_petal=3, ring=4, glow=False):
    sp = Sprite(16, 16)
    for k in range(n):
        a = -math.pi / 2 + 2 * math.pi * k / n
        cx, cy = 8 + math.cos(a) * ring, 8 + math.sin(a) * ring
        sp.part(sp.circle(cx, cy, r_petal), petal_ramp, "sphere")
    sp.part(sp.circle(8, 8, 2), centre_ramp, "sphere")
    sp.plot(7, 7, "#fffefe")
    sp.outline(petal_ramp[0])
    return sp


def flower_glow():
    sp = Sprite(16, 16)
    sp.part(sp.circle(8, 8, 5), RAMPS["cyan"], "inset", glow=True)
    sp.fill(sp.rect(8, 1, 8, 15), "#d2f8ff", glow=True)
    sp.fill(sp.rect(1, 8, 15, 8), "#d2f8ff", glow=True)
    sp.plot(6, 6, "#fffefe", glow=True)
    return sp


# ---------- trees: one function per species, on a 32 x 48 unit grid ------------------------------

class G:
    """Draws a 32 x 48 design onto a sprite at an offset and a whole-number-free scale."""

    def __init__(self, sp, ox=0, oy=0, k=1.0):
        self.sp, self.ox, self.oy, self.k = sp, ox, oy, k

    def X(self, x):
        return self.ox + x * self.k

    def Y(self, y):
        return self.oy + y * self.k

    def circle(self, x, y, r):
        return self.sp.circle(self.X(x), self.Y(y), max(1, r * self.k))

    def ellipse(self, x0, y0, x1, y1):
        return self.sp.ellipse(self.X(x0), self.Y(y0), self.X(x1), self.Y(y1))

    def rect(self, x0, y0, x1, y1):
        return self.sp.rect(self.X(x0), self.Y(y0), self.X(x1), self.Y(y1))

    def poly(self, pts):
        return self.sp.poly([(self.X(x), self.Y(y)) for x, y in pts])

    def taper(self, x0, y0, x1, y1, w0, w1=0):
        return self.sp.taper(self.X(x0), self.Y(y0), self.X(x1), self.Y(y1), max(1, w0 * self.k), w1 * self.k)

    def line(self, pts, w):
        return self.sp.line([(self.X(x), self.Y(y)) for x, y in pts], max(1, round(w * self.k)))

    def plot(self, x, y, c, glow=False):
        self.sp.plot(round(self.X(x)), round(self.Y(y)), c, glow)


def t_oak(g):
    g.sp.part(g.taper(16, 47, 16, 26, 6, 4), BARK, "cyl_v")
    g.sp.part(g.line([(16, 32), (9, 24)], 2) | g.line([(16, 30), (24, 23)], 2), BARK, "rim")
    for x, y, r in ((9, 22, 7), (23, 22, 7), (16, 25, 6)):
        g.sp.part(g.circle(x, y, r), LEAF[:4], "sphere")
    g.sp.part(g.circle(16, 13, 10), LEAF, "sphere")
    g.plot(12, 7, "#c1f1d8")


def t_birch(g):
    g.sp.part(g.rect(14, 16, 17, 47), PALE, "cyl_v")
    for y in (20, 26, 33, 40, 45):
        g.sp.fill(g.rect(14, y, 16, y), INK)
    g.sp.part(g.ellipse(7, 2, 25, 30), BIRCHLEAF, "sphere")
    g.sp.part(g.ellipse(13, 16, 23, 30), BIRCHLEAF[:3], "sphere")
    g.plot(11, 7, "#fffefe")


def t_pine(g):
    g.sp.part(g.rect(15, 38, 17, 47), BARK, "cyl_v")
    for k in range(5):
        yb = 42 - k * 8
        w = 15 - k * 3
        g.sp.part(g.poly([(16 - w, yb), (16 + w, yb), (16, yb - 13)]), PINE, "rim")
    g.plot(15, 4, "#84e3b5")


def t_kapok(g):
    g.sp.part(g.taper(16, 47, 16, 14, 7, 4), ["#41322c", "#725244", "#a09f91", "#cfaa81"], "cyl_v")
    for s in (-1, 1):
        g.sp.part(g.poly([(16 + s * 2, 38), (16 + s * 9, 47), (16 + s * 2, 47)]), ["#41322c", "#725244", "#a09f91"], "rim")
        g.sp.part(g.line([(16, 20), (16 + s * 11, 13)], 2), BARK, "rim")
    g.sp.part(g.ellipse(0, 7, 31, 17), LEAF[:4], "sphere")
    g.sp.part(g.ellipse(6, 2, 26, 12), LEAF, "sphere")
    g.plot(10, 4, "#c1f1d8")


def t_palm(g):
    pts = [(15, 47), (15, 38), (16, 28), (17, 18), (18, 12)]
    g.sp.part(g.line(pts, 3), BARK, "rim")
    for a in (-2.9, -2.3, -1.6, -0.9, -0.3, 0.3, 2.7):
        ex, ey = 18 + math.cos(a) * 12, 12 + math.sin(a) * 6 + 7
        g.sp.part(g.taper(18, 12, ex, ey, 4, 1), LEAF[:4], "rim")
    g.sp.part(g.circle(18, 13, 2), BARK, "sphere")
    g.plot(17, 8, "#c1f1d8")


def t_acacia(g):
    g.sp.part(g.taper(16, 47, 16, 30, 4, 3), BARK, "cyl_v")
    g.sp.part(g.line([(16, 31), (8, 18)], 2) | g.line([(16, 31), (24, 17)], 2), BARK, "rim")
    g.sp.part(g.ellipse(0, 10, 31, 19), ACACIA, "sphere")
    g.sp.part(g.ellipse(6, 7, 26, 14), ACACIA, "sphere")
    g.plot(10, 9, "#c1f1d8")


def t_willow(g):
    g.sp.part(g.taper(16, 47, 16, 22, 6, 4), BARK, "cyl_v")
    g.sp.part(g.ellipse(3, 6, 29, 22), WILLOW, "sphere")
    for k, x in enumerate(range(3, 30, 4)):
        g.sp.part(g.rect(x, 15, x + 2, 37 - abs(k - 3) * 3), WILLOW[:3], "cyl_v")
    g.plot(10, 9, "#c1f1d8")


def t_cactus(g):
    g.sp.part(g.rect(13, 10, 19, 47), CACTUS, "cyl_v")
    g.sp.part(g.rect(6, 26, 13, 30) | g.rect(6, 16, 10, 30), CACTUS, "cyl_v")
    g.sp.part(g.rect(19, 22, 25, 26) | g.rect(22, 13, 26, 26), CACTUS, "cyl_v")
    g.sp.part(g.circle(16, 9, 2), RAMPS["red"], "sphere")
    g.plot(14, 14, "#c1f1d8")


def t_glowcap(g):
    g.sp.part(g.rect(13, 20, 18, 47), RAMPS["bone"], "cyl_v")
    g.sp.part(g.poly([(1, 22), (4, 12), (10, 6), (16, 4), (22, 6), (28, 12), (31, 22)]), RAMPS["violet"], "sphere")
    g.sp.part(g.rect(3, 22, 29, 23), ["#230c44", "#4f1d83"], "flat")
    for x, y in ((8, 14), (15, 9), (22, 12), (12, 18), (24, 18)):
        g.sp.fill(g.rect(x, y, x + 1, y + 1), "#74effe")
        g.plot(x, y, "#d2f8ff")


def t_crystal(g):
    for x, top, w, ramp in ((9, 18, 4, RAMPS["violet"]), (23, 14, 4, RAMPS["violet"]), (16, 2, 6, RAMPS["cyan"]), (5, 30, 3, RAMPS["cyan"]), (27, 30, 3, RAMPS["cyan"])):
        g.sp.part(g.poly([(x, top), (x + w, top + w * 1.5), (x + w * 0.7, 47), (x - w * 0.7, 47), (x - w, top + w * 1.5)]), ramp, "inset")
    g.plot(15, 6, "#fffefe")
    g.plot(22, 17, "#fffefe")


def t_kelp(g):
    for k, x0 in enumerate((9, 16, 23)):
        pts = [(x0 + 3 * math.sin(i * 0.8 + k), 47 - i * 4) for i in range(11 - k)]
        g.sp.part(g.line(pts, 3), KELP, "rim")
        for i in range(2, len(pts), 3):
            px, py = pts[i]
            g.sp.part(g.circle(px + 3, py, 2), RAMPS["brass"], "sphere")


def bare(seed, trunk_w=4, trunk_top=26, ramp=None):
    """A broadleaf tree in winter for the far ring: a dark crown of twigs, no leaves, no outline
    (a 1 px twig beside an outline would vanish; these are the twigs)."""
    ramp = ramp or ["#2e0d06", "#41322c", "#725244"]
    sp = Sprite(32, 48)
    rnd = random.Random(seed)
    sp.part(sp.taper(16, 47, 16, trunk_top, trunk_w, max(2, trunk_w - 2)), ramp, "cyl_v", seam=False)

    def br(x, y, a, L, w, d):
        x2, y2 = x + math.cos(a) * L, y + math.sin(a) * L
        sp.fill(sp.line([(x, y), (x2, y2)], w), ramp[1] if w > 1 else ramp[0])
        if d == 0:
            return
        for s in (-1, 1):
            br(x2, y2, a + s * (0.35 + rnd.random() * 0.3), L * 0.68, max(1, w - 1), d - 1)
    for a in (-2.2, -1.57, -0.9):
        br(16, trunk_top + 2, a + (rnd.random() - 0.5) * 0.2, 9, 2, 3)
    sp.plot(15, trunk_top + 6, ramp[2])
    return sp


SPECIES = {
    "oak": (t_oak, RAMPS["green"]), "birch": (t_birch, RAMPS["green"]), "pine": (t_pine, RAMPS["paper"]),
    "kapok": (t_kapok, RAMPS["olive"]), "palm": (t_palm, RAMPS["olive"]), "acacia": (t_acacia, RAMPS["sand"]),
    "willow": (t_willow, ["#2c2c25", "#434137", "#725244"]), "cactus": (t_cactus, RAMPS["sand"]),
    "glowcap": (t_glowcap, ["#230c44", "#4f1d83", "#69608b"]), "crystal": (t_crystal, ["#3e3551", "#69608b", "#bfcad7"]),
    "kelp": (t_kelp, RAMPS["glass"][:4]),
}


def impostor(fn):
    sp = Sprite(32, 48)
    fn(G(sp))
    sp.outline()
    return sp


def icon(fn, ground):
    sp = Sprite(56, 56)
    sp.part(sp.ellipse(6, 46, 49, 54), ground, "sphere")
    fn(G(sp, ox=12, oy=4, k=1.0))
    sp.outline()
    return sp


def build():
    items = []
    items += [("bark_birch", tile_birch()), ("bark_palm", tile_palm()),
              ("cap_red", tile_cap("#d12d41", "#fffefe", "#a61e1f")), ("cap_brown", tile_cap("#a8633e", "#e69d69", "#725244"))]
    items += [("flower_red", flower(RAMPS["red"], RAMPS["soot"], 5, 3, 4)),
              ("flower_yellow", flower(RAMPS["gold"][:4], RAMPS["rust"], 8, 2, 4)),
              ("flower_blue", flower(RAMPS["blue"], RAMPS["gold"][:4], 5, 3, 4)),
              ("flower_white", flower(RAMPS["paper"], RAMPS["gold"][:4], 10, 2, 5)),
              ("flower_glow", flower_glow())]
    for name, (fn, ground) in SPECIES.items():
        items.append(("imp_" + name, impostor(fn)))
        items.append(("icon_" + name, icon(fn, ground)))
    # the deciduous species' winter impostors: bare twig crowns
    items += [("imp_oak_bare", bare(1, 6, 26)), ("imp_birch_bare", bare(2, 4, 16, ["#5d778d", "#92a3b2", "#bfcad7"])),
              ("imp_willow_bare", bare(3, 6, 22))]
    return items


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    items = build()
    for name, sp in items:
        save(sp.image(), os.path.join(OUT, name + ".png"), source="pixelkit")
    print("wrote", len(items), "pngs to", OUT)
    if "--sheet" in argv:
        from PIL import Image
        out = argv[argv.index("--sheet") + 1]
        imgs = [(n, sp.image()) for n, sp in items]
        refs = os.path.join(KIT, "..", "references")
        for r in ("archivist", "brine_hive", "glass_seraph"):
            p = os.path.join(refs, r + ".png")
            if os.path.exists(p):
                imgs.append(("(ref) " + r, Image.open(p).convert("RGBA")))
        sheet(imgs, scale=4, cols=8, cell=(56 * 4, 56 * 4)).save(out)
        print("wrote", out)


if __name__ == "__main__":
    main(sys.argv[1:])
