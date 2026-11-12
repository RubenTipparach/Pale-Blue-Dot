"""Icons and textures for the construction mockup (docs/mockups/construction.html, M22).

Pixel art drawn in code with the balatro-pixel-art kit (`.claude/skills/balatro-pixel-art`), the
solitaire collection's style, every colour from Sweaft-64 (the kit's save() refuses anything else).
Three kinds of output, all 32 x 32:

- icons: transparent, one bold subject, 1 px outline, light from the top left, shown only at whole
  multiples of 32 px with `image-rendering: pixelated`;
- tiles: opaque and seamless, like the game's settlement textures, sampled nearest with no mipmaps
  by the page's three.js yard;
- cut-out tiles: the crane's lattice, alpha 0 or 255 only, drawn with alphaTest 0.5 so the mast
  and jib are see-through the way a lattice is.

The ground, the yard's paving, the bay's brick and slate and the office's boards are the game's own
settlement textures, copied byte for byte into docs/mockups/construction/game/, not drawn here.

    /usr/bin/python3 tools/art/construction_art.py
    /usr/bin/python3 tools/art/construction_art.py --sheet OUT.png
"""

import glob
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art")
sys.path.insert(0, os.path.join(SKILL, "scripts"))
from pixelkit import RAMPS, Sprite, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "construction")
N = 32
WHITE = "#fffefe"

R = dict(RAMPS)
R.update({
    "paint": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],          # crane and machine yellow, as industry's
    "conc": ["#434137", "#a09f91", "#bfcad7", "#fffefe"],
    "stone": ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "white": ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "dark": ["#010e1a", "#011c34", "#1f3b52", "#3e586e", "#5d778d"],
    "print": ["#2c2c25", "#434137", "#a09f91", "#bfcad7"],
    "bp": ["#191954", "#15578a", "#1096c3", "#74effe", "#d2f8ff"],
    "hot": ["#a61e1f", "#fe6420", "#fb902f", "#faaf5c", WHITE],
    "teal": ["#0c2f44", "#204f63", "#53998b", "#84e3b5"],
})


def glint(sp, pts):
    sp.plots(pts, WHITE)


# ---------------------------------------------------------------- icons

def assembly_bay(sp):
    roof = sp.poly([(2, 12), (16, 4), (30, 12)])
    walls = sp.rect(3, 12, 29, 29)
    sp.part(walls, R["red"], "rim", base=2)
    for x in (6, 10, 22, 26):
        sp.fill(sp.rect(x, 13, x, 28) & walls, R["red"][1])
    sp.part(roof, R["stone"], "rim", base=3)
    door = sp.rect(9, 15, 23, 29)
    sp.part(door, R["dark"], "flat", base=1)
    sp.part(sp.rect(9, 15, 23, 17), R["paint"], "rim", base=2)           # the bridge crane's beam
    sp.part(sp.rect(15, 18, 16, 21), R["steel"], "flat", base=1)         # its hook line
    sp.part(sp.rect(12, 22, 20, 27), R["wood"], "rim", base=2)           # a crate under it
    sp.fill(sp.rect(12, 24, 20, 24), R["wood"][0])
    sp.fill(sp.rect(5, 14, 7, 16) | sp.rect(25, 14, 27, 16), R["gold"][3])  # lit windows
    glint(sp, [(15, 6), (16, 5)])
    sp.outline()


def hex_printer(sp):
    # the printed layer: a hex outline of grey walls
    hexo = sp.poly([(8, 20), (12, 15), (20, 15), (24, 20), (20, 26), (12, 26)])
    hexi = sp.poly([(11, 20), (14, 17), (18, 17), (21, 20), (18, 24), (14, 24)])
    sp.part(hexo & ~hexi, R["print"], "rim", base=2)
    sp.part(sp.rect(2, 28, 29, 29), R["soot"], "flat", base=1)            # rails
    for x in (3, 26):
        sp.part(sp.rect(x, 5, x + 3, 28), R["paint"], "cyl_v", base=2)    # legs
    sp.part(sp.rect(2, 3, 29, 6), R["paint"], "rim", base=2)              # bridge
    sp.part(sp.rect(13, 7, 18, 10), R["dark"], "rim", base=3)             # carriage
    sp.part(sp.rect(15, 11, 16, 15), R["steel"], "cyl_v", base=2)         # ram
    sp.fill(sp.rect(6, 4, 25, 4) & (np.arange(N)[None, :] % 4 == 0), R["ink"][1])
    glint(sp, [(4, 3), (5, 3)])
    sp.outline()
    sp.part(sp.rect(14, 15, 17, 17), R["hot"], "inset", glow=True)        # the hot nozzle
    sp.plot(15, 16, WHITE, glow=True)


def tower_crane(sp):
    ys = np.arange(N)[:, None]
    xs = np.arange(N)[None, :]
    mast = sp.rect(5, 8, 11, 29)
    sp.part(mast, R["paint"], "rim", base=2)
    sp.fill(mast & ((((ys - xs) % 7) == 0) | (((ys + xs) % 7) == 0)) & (xs > 5) & (xs < 11), R["paint"][0])
    jib = sp.rect(1, 4, 31, 8)
    sp.part(jib, R["paint"], "rim", base=3)
    sp.fill(jib & (((xs + ys) % 5) == 0) & (ys > 4) & (ys < 8), R["paint"][0])
    sp.part(sp.poly([(6, 4), (8, 0), (10, 4)]), R["paint"], "flat", base=2)   # the top
    sp.part(sp.rect(1, 9, 5, 14), R["conc"], "rim", base=2)              # counterweight
    sp.part(sp.rect(11, 9, 15, 13), R["glass"], "rim", base=3)           # the cab
    sp.part(sp.rect(22, 9, 26, 10), R["dark"], "flat", base=2)           # trolley
    sp.part(sp.rect(24, 11, 24, 17), R["steel"], "flat", base=1)         # line
    sp.part(sp.rect(19, 18, 29, 25), R["wood"], "rim", base=2)           # the load
    sp.fill(sp.rect(20, 21, 28, 21), R["wood"][0])
    sp.part(sp.rect(3, 29, 13, 31), R["conc"], "flat", base=1)           # footing
    glint(sp, [(2, 5), (3, 5)])
    sp.outline()
    sp.plot(8, 0, R["red"][3], glow=True)
    sp.plot(31, 3, R["red"][3], glow=True)


def gantry_crane(sp):
    for x in (3, 25):
        leg = sp.poly([(x + 2, 6), (x + 4, 6), (x + 6, 29), (x, 29)])
        sp.part(leg, R["teal"], "rim", base=2)
    beam = sp.rect(2, 3, 29, 7)
    sp.part(beam, R["paint"], "rim", base=2)
    sp.fill(beam & (np.arange(N)[None, :] % 4 == 1), R["paint"][0])
    sp.part(sp.rect(12, 8, 19, 10), R["dark"], "rim", base=3)
    sp.part(sp.rect(15, 11, 16, 16), R["steel"], "flat", base=1)
    hook = sp.rect(13, 17, 18, 19) | sp.rect(15, 20, 16, 22)
    sp.part(hook, R["paint"], "rim", base=2)
    sp.part(sp.rect(2, 29, 29, 30), R["soot"], "flat", base=1)
    glint(sp, [(3, 3), (4, 3)])
    sp.outline()


def crate(sp):
    top = sp.poly([(7, 4), (29, 4), (25, 9), (3, 9)])
    front = sp.rect(3, 10, 24, 29)
    side = sp.poly([(25, 10), (29, 5), (29, 24), (25, 29)])
    sp.part(front, R["wood"], "rim", base=2)
    for y in (15, 21):
        sp.fill(sp.rect(4, y, 23, y), R["wood"][0])
    sp.fill(sp.line([(5, 27), (22, 12)], 2) & front, R["wood"][3])
    sp.part(side, R["wood"], "flat", base=1)
    sp.part(top, R["wood"], "flat", base=3)
    for (x0, y0, x1, y1) in [(3, 10, 5, 12), (22, 10, 24, 12), (3, 27, 5, 29), (22, 27, 24, 29)]:
        sp.part(sp.rect(x0, y0, x1, y1), R["steel"], "flat", base=2, seam=False)
    sp.fill(sp.rect(10, 17, 17, 19) & front, R["paint"][3])
    glint(sp, [(8, 5), (9, 5)])
    sp.outline()


def printstone(sp):
    # a hex-topped block of printed stone, its beads in layers
    top = sp.poly([(4, 9), (10, 4), (22, 4), (28, 9), (22, 14), (10, 14)])
    body = sp.poly([(4, 9), (10, 14), (22, 14), (28, 9), (28, 25), (22, 30), (10, 30), (4, 25)])
    sp.part(body & ~top, R["print"], "rim", base=2)
    ys = np.arange(N)[:, None]
    sp.fill(body & ~top & (ys % 3 == 0) & (ys > 14), R["print"][1])
    sp.fill(body & ~top & (np.arange(N)[None, :] >= 16) & (np.arange(N)[None, :] <= 22), R["print"][1])
    sp.part(top, R["print"], "flat", base=3)
    sp.fill(sp.poly([(9, 9), (12, 6), (20, 6), (23, 9), (20, 12), (12, 12)]), R["conc"][2])
    glint(sp, [(11, 6), (12, 5)])
    sp.outline()


def wall_panel(sp):
    panel = sp.poly([(6, 3), (27, 6), (27, 30), (6, 27)])
    sp.part(panel, R["print"], "rim", base=2)
    win = sp.poly([(11, 9), (21, 10), (21, 18), (11, 17)])
    sp.part(win, R["glass"], "rim", base=2)
    sp.fill(sp.line([(16, 10), (16, 17)], 1) & win, R["glass"][0])
    edge = sp.poly([(3, 4), (6, 3), (6, 27), (3, 28)])
    sp.part(edge, R["print"], "flat", base=1)
    sp.part(sp.rect(13, 0, 18, 2), R["steel"], "rim", base=2)             # lifting eye
    glint(sp, [(12, 10), (13, 10)])
    sp.outline()


def mast(sp):
    pole = sp.rect(14, 2, 18, 30)
    sp.part(pole, R["wood"], "cyl_v", base=2)
    yard = sp.rect(4, 8, 28, 10)
    sp.part(yard, R["wood"], "cyl_h", base=2)
    sp.part(sp.rect(13, 3, 19, 5), R["steel"], "rim", base=2)
    sp.part(sp.rect(13, 25, 19, 27), R["steel"], "rim", base=2)
    sp.part(sp.rect(4, 11, 5, 14) | sp.rect(27, 11, 28, 14), R["bone"], "flat", base=1)
    glint(sp, [(15, 6), (15, 7)])
    sp.outline()


def wing_pair(sp):
    left = sp.poly([(1, 11), (14, 7), (14, 22), (2, 22)])
    right = sp.poly([(18, 7), (31, 11), (30, 22), (18, 22)])
    sp.part(left, R["white"], "rim", base=3, depth=2)
    sp.part(right, R["white"], "rim", base=3, depth=2)
    sp.part(sp.poly([(1, 11), (5, 10), (5, 22), (2, 22)]), R["red"], "rim", base=2)
    sp.part(sp.poly([(27, 10), (31, 11), (30, 22), (27, 22)]), R["red"], "rim", base=2)
    sp.fill(sp.rect(6, 19, 13, 19) | sp.rect(19, 19, 26, 19), R["white"][1])
    sp.part(sp.rect(13, 5, 19, 24), R["paint"], "cyl_v", base=2)          # the spar box
    sp.part(sp.rect(12, 25, 20, 28), R["wood"], "rim", base=2)            # its cradle
    glint(sp, [(7, 10), (8, 10)])
    sp.outline()


def engine(sp):
    block = sp.rect(7, 9, 26, 26)
    sp.part(block, R["steel"], "rim", base=2)
    for y in (11, 14, 17, 20, 23):
        sp.fill(sp.rect(8, y, 25, y), R["steel"][0])
    sp.part(sp.rect(10, 4, 23, 8), R["paint"], "rim", base=2)             # rocker cover
    hub = sp.circle(6, 17, 4)
    sp.part(hub, R["red"], "sphere", base=2)
    blade = sp.poly([(4, 3), (8, 3), (7, 31), (5, 31)])
    sp.part(blade & ~hub, R["dark"], "rim", base=3)
    sp.part(sp.rect(24, 27, 29, 29), R["soot"], "flat", base=1)
    glint(sp, [(11, 5), (12, 5)])
    sp.outline()


def hull_section(sp):
    outer = sp.poly([(2, 4), (30, 4), (28, 16), (22, 26), (16, 29), (10, 26), (4, 16)])
    inner = sp.poly([(6, 4), (26, 4), (24, 15), (20, 22), (16, 24), (12, 22), (8, 15)])
    sp.part(outer & ~inner, R["wood"], "rim", base=2)
    ys = np.arange(N)[:, None]
    sp.fill(outer & ~inner & (ys % 4 == 0), R["wood"][0])
    for x in (9, 16, 23):
        sp.part(sp.rect(x, 5, x + 1, 22) & inner, R["wood"], "flat", base=3)
    sp.part(sp.rect(15, 26, 17, 31), R["dark"], "rim", base=2)            # keel
    sp.part(sp.rect(2, 3, 30, 5), R["red"], "flat", base=2)
    glint(sp, [(4, 6), (5, 7)])
    sp.outline()


def blueprint(sp):
    page = sp.poly([(3, 4), (29, 4), (29, 22), (23, 28), (3, 28)])
    sp.part(page, R["bp"], "flat", base=1)
    xs = np.arange(N)[None, :]
    ys = np.arange(N)[:, None]
    sp.fill(page & ((xs % 6 == 0) | (ys % 6 == 0)), R["bp"][0])
    hexo = sp.poly([(9, 16), (12, 11), (19, 11), (22, 16), (19, 21), (12, 21)])
    hexi = sp.poly([(11, 16), (13, 13), (18, 13), (20, 16), (18, 19), (13, 19)])
    sp.fill(hexo & ~hexi, R["bp"][4])
    sp.fill(sp.rect(15, 13, 16, 19), R["bp"][3])
    curl = sp.poly([(23, 28), (29, 22), (23, 22)])
    sp.part(curl, R["bp"], "flat", base=3)
    sp.fill(sp.rect(5, 6, 14, 7), R["bp"][3])
    sp.outline()


def print_head(sp):
    sp.part(sp.rect(8, 2, 23, 9), R["paint"], "rim", base=2)
    sp.part(sp.rect(11, 10, 20, 17), R["dark"], "rim", base=3)
    sp.part(sp.poly([(12, 18), (19, 18), (17, 23), (14, 23)]), R["steel"], "rim", base=2)
    sp.part(sp.rect(2, 27, 29, 30), R["print"], "rim", base=2)
    sp.fill(sp.rect(3, 29, 28, 29), R["print"][1])
    sp.part(sp.rect(3, 4, 7, 6), R["steel"], "cyl_h", base=2)             # the feed hose
    glint(sp, [(9, 3), (10, 3)])
    sp.outline()
    sp.part(sp.rect(14, 24, 17, 27), R["hot"], "inset", glow=True)
    sp.plot(15, 25, WHITE, glow=True)


ICONS = {
    "assembly-bay": assembly_bay, "hex-printer": hex_printer, "tower-crane": tower_crane,
    "gantry-crane": gantry_crane, "crate": crate, "printstone": printstone, "wall-panel": wall_panel,
    "mast": mast, "wing-pair": wing_pair, "engine": engine, "hull-section": hull_section,
    "blueprint": blueprint, "print-head": print_head,
}


# ---------------------------------------------------------------- tiles (32 x 32, seamless)

YS, XS = np.mgrid[0:N, 0:N]
SPECK = np.random.default_rng(11).random((N, N)) < 0.06


def full(sp):
    return sp.rect(0, 0, N - 1, N - 1)


def tex_printstone(sp):
    """Printed stone: a bead every 4 px (the head lays it 12.5 cm a pass on a 1 m repeat), each bead
    lit on top and shaded under, with a little grain."""
    ramp = R["print"]
    sp.fill(full(sp), ramp[2])
    v = YS % 4
    sp.fill(v == 0, ramp[3])
    sp.fill(v == 3, ramp[1])
    sp.fill(SPECK & (v == 1), ramp[1])
    sp.fill(SPECK & (v == 2) & (XS % 3 == 0), ramp[3])


def tex_printstone_top(sp):
    ramp = R["print"]
    sp.fill(full(sp), ramp[2])
    sp.fill(SPECK, ramp[1])
    sp.fill(np.roll(SPECK, 7, axis=1) & ~SPECK, ramp[3])


def tex_hazard(sp):
    stripe = ((XS + YS) // 8) % 2 == 0
    sp.fill(stripe, R["paint"][3])
    sp.fill(~stripe, R["ink"][1])
    sp.fill(stripe & (((XS + YS) % 16) == 0), WHITE)


def tex_paint(sp):
    """Crane and gantry paint: yellow plates with a seam every 16 px and a rivet line."""
    ramp = R["paint"]
    sp.fill(full(sp), ramp[2])
    sp.fill(SPECK, ramp[1])
    u, v = XS % 16, YS % 16
    sp.fill(v == 0, ramp[0])
    sp.fill(v == 1, ramp[3])
    sp.fill((u == 15) & (v > 1), ramp[1])
    sp.fill((v == 8) & (u % 4 == 2), ramp[0])


def tex_teal(sp):
    """The printer's frame: teal machine paint, bolted plates."""
    ramp = R["teal"]
    sp.fill(full(sp), ramp[2])
    sp.fill(SPECK, ramp[1])
    u, v = XS % 16, YS % 16
    sp.fill((u == 0) | (v == 0), ramp[0])
    sp.fill((u == 1) | (v == 1), ramp[3])
    for rx, ry in [(3, 3), (12, 3), (3, 12), (12, 12)]:
        sp.fill((u == rx) & (v == ry), ramp[3])
        sp.fill((u == rx + 1) & (v == ry + 1), ramp[0])


def tex_crate(sp):
    ramp = R["wood"]
    sp.fill(full(sp), ramp[2])
    v = YS % 8
    sp.fill(v == 0, ramp[0])
    sp.fill(v == 1, ramp[3])
    sp.fill(SPECK & (v > 1), ramp[1])
    frame = (XS <= 2) | (XS >= 29) | (YS <= 2) | (YS >= 29)
    sp.fill(frame, ramp[1])
    sp.fill((XS == 0) | (YS == 0), ramp[0])
    sp.fill(((XS == 1) | (YS == 1)) & frame, ramp[3])
    diag = (np.abs(XS - YS) <= 1) & ~frame
    sp.fill(diag, ramp[3])
    sp.fill((XS - YS == 2) & ~frame, ramp[0])
    for cx, cy in [(1, 1), (30, 1), (1, 30), (30, 30)]:
        sp.fill((np.abs(XS - cx) <= 1) & (np.abs(YS - cy) <= 1), R["steel"][2])


def tex_container(sp):
    """A steel box's corrugation, rust red, a rib every 4 px."""
    ramp = R["rust"]
    u = XS % 4
    sp.fill(u == 0, ramp[0])
    sp.fill(u == 1, ramp[3])
    sp.fill(u == 2, ramp[2])
    sp.fill(u == 3, ramp[1])
    sp.fill(SPECK & (u == 2), ramp[1])


def tex_counterweight(sp):
    ramp = R["conc"]
    sp.fill(full(sp), ramp[1])
    sp.fill(SPECK, ramp[0])
    u, v = XS % 16, YS % 16
    sp.fill((u == 0) | (v == 0), ramp[0])
    sp.fill(((u == 1) | (v == 1)) & (u != 0) & (v != 0), ramp[2])


def plates(sp, ramp, cell=16):
    """Riveted plates, as the ship-crafting hulls: a seam on the plate's top and left, a lit line
    inside it, a shadow line on its bottom and right, a rivet near each corner."""
    base = len(ramp) - 2
    sp.fill(full(sp), ramp[base])
    sp.fill(SPECK, ramp[base - 1])
    u, v = XS % cell, YS % cell
    sp.fill((u == 1) | (v == 1), ramp[base + 1])
    sp.fill((u == cell - 1) | (v == cell - 1), ramp[base - 1])
    sp.fill((u == 0) | (v == 0), ramp[0])
    for rx, ry in [(3, 3), (cell - 4, 3), (3, cell - 4), (cell - 4, cell - 4)]:
        sp.fill((u == rx) & (v == ry), ramp[base + 1])


def tex_white(sp):
    plates(sp, R["white"])


def tex_red(sp):
    plates(sp, R["red"])


def tex_lattice(sp):
    """The crane's lattice, a cut-out: chords down both sides and across, a diagonal each panel.
    Alpha is 0 between the members, so the page draws it with alphaTest and both sides."""
    ramp = R["paint"]
    u, v = XS % 32, YS % 32
    chord = (u <= 3) | (u >= 28)
    rung = (v <= 2)
    diag = np.abs(u - v) <= 1
    m = chord | rung | diag
    sp.fill(m, ramp[2])
    sp.fill(m & ((u == 0) | (v == 0) | (u == 28)), ramp[3])
    sp.fill(m & ((u == 3) | (u == 31) | (v == 2)), ramp[0])
    sp.fill(diag & (u - v == 1), ramp[3])
    sp.fill(diag & (u - v == -1), ramp[1])


TEXTURES = {
    "tex-printstone": tex_printstone, "tex-printstone-top": tex_printstone_top, "tex-hazard": tex_hazard,
    "tex-paint": tex_paint, "tex-teal": tex_teal, "tex-crate": tex_crate, "tex-container": tex_container,
    "tex-counterweight": tex_counterweight, "tex-white": tex_white, "tex-red": tex_red,
}
CUTOUTS = {"tex-lattice": tex_lattice}


def main():
    os.makedirs(OUT, exist_ok=True)
    icons, texes = [], []
    for name, draw in ICONS.items():
        sp = Sprite(N, N)
        draw(sp)
        img = sp.image()
        save(img, os.path.join(OUT, name + ".png"), source="balatro-pixel-art pixelkit")
        icons.append((name, img))
    for name, draw in TEXTURES.items():
        sp = Sprite(N, N)
        draw(sp)
        img = sp.image()
        assert img.getchannel("A").getextrema() == (255, 255), name + " has holes"
        save(img.convert("RGB"), os.path.join(OUT, name + ".png"), source="balatro-pixel-art pixelkit")
        texes.append((name, img))
    for name, draw in CUTOUTS.items():
        sp = Sprite(N, N)
        draw(sp)
        img = sp.image()
        save(img, os.path.join(OUT, name + ".png"), source="balatro-pixel-art pixelkit")
        texes.append((name, img))
    print("wrote", len(icons), "icons and", len(texes), "tiles to", OUT)
    if "--sheet" in sys.argv:
        from PIL import Image
        path = sys.argv[sys.argv.index("--sheet") + 1]
        refs = sorted(p for p in glob.glob(os.path.join(SKILL, "references", "*.png")) if not p.endswith("_contact.png"))
        refs = [(os.path.basename(p)[:-4], Image.open(p).convert("RGBA")) for p in refs]
        refs = [r for r in refs if r[1].size == (56, 56)][-6:]
        tiled = []
        for name, img in texes:
            t = Image.new("RGBA", (64, 64))
            for ox in (0, 32):
                for oy in (0, 32):
                    t.paste(img, (ox, oy))
            tiled.append((name, t))
        sheet(refs + icons + tiled, scale=3, cols=8).save(path)
        print("sheet", path)


if __name__ == "__main__":
    main()
