"""Module icons and hull textures for the ship-crafting mockup (docs/mockups/ship-crafting.html).

Pixel art drawn in code with the balatro-pixel-art kit (`.claude/skills/balatro-pixel-art`), the
solitaire collection's style, every colour from Sweaft-64 (the kit's save() refuses anything else).
Two kinds of output:

- module icons: 32 x 32, transparent, one bold subject, 1 px outline, light from the top left;
  the page shows them only at whole multiples of 32 px with `image-rendering: pixelated`;
- hull textures: 32 x 32 tiles that repeat seamlessly, opaque, no outline, like the game's own
  settlement textures (assets/textures/settlement/), which the page's three.js ships and
  shipyard sample with NearestFilter and no mipmaps.

The shipyard's apron and the hangar walls are not drawn here: they wear the game's own
granite and iron textures, copied byte for byte into docs/mockups/ship-crafting/game/.

    python tools/art/ship-crafting_art.py
    python tools/art/ship-crafting_art.py --sheet OUT.png
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

OUT = os.path.join(ROOT, "docs", "mockups", "ship-crafting")
N = 32

R = dict(RAMPS)
R.update({
    "white": ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "dark": ["#010e1a", "#011c34", "#1f3b52", "#3e586e", "#5d778d"],
    "orange": ["#733126", "#c56d25", "#fb902f", "#faaf5c"],
    "hazard": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "tank": ["#4a051e", "#a61e1f", "#d12d41", "#fa3a64"],
    "plume": ["#15578a", "#1096c3", "#74effe", "#d2f8ff", "#fffefe"],
})
GLINT = "#fffefe"


def glint_star(sp, x, y, ramp):
    sp.plot(x, y, GLINT, glow=True)
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], ramp[3], glow=True)


# ---------------------------------------------------------------- module icons

def cockpit(sp):
    hull = sp.poly([(2, 27), (4, 17), (11, 9), (20, 6), (27, 8), (30, 14), (30, 27)])
    sp.part(hull, R["white"], "rim", base=3, depth=2)
    canopy = sp.poly([(6, 18), (12, 11), (20, 8), (26, 10), (27, 15), (18, 17)])
    sp.part(canopy, R["cyan"], "rim", base=2)
    sp.fill(sp.line([(11, 15), (16, 11)], 2) & canopy, R["cyan"][4])
    sp.part(sp.rect(3, 21, 29, 23), R["hazard"], "rim", base=2)
    sp.part(sp.rect(3, 25, 29, 28), R["dark"], "flat", base=2)
    sp.plots([(12, 11), (13, 10)], GLINT)
    sp.outline()


def cargo(sp):
    top = sp.poly([(7, 3), (29, 3), (25, 8), (3, 8)])
    front = sp.rect(3, 9, 24, 29)
    side = sp.poly([(25, 9), (29, 4), (29, 24), (25, 29)])
    sp.part(front, R["orange"], "rim", base=2)
    for x in (8, 13, 18):
        sp.fill(sp.rect(x, 10, x, 28), R["orange"][1])
    sp.part(side, R["orange"], "flat", base=1)
    sp.part(top, R["orange"], "flat", base=3)
    band = sp.rect(4, 18, 23, 22)
    ys, xs = np.mgrid[0:N, 0:N]
    sp.fill(band & (((xs + ys) // 2) % 2 == 0), R["ink"][1])
    sp.fill(band & (((xs + ys) // 2) % 2 == 1), R["gold"][3])
    sp.plots([(8, 4), (9, 4)], GLINT)
    sp.outline()


def thruster(sp):
    mount = sp.rect(9, 2, 22, 8)
    sp.part(mount, R["white"], "rim", base=3)
    bell = sp.poly([(10, 9), (21, 9), (27, 21), (4, 21)])
    sp.part(bell, R["dark"], "cyl_v", base=3)
    sp.part(sp.rect(4, 19, 27, 21), R["steel"], "rim", base=2)
    sp.plots([(10, 3), (11, 3)], GLINT)
    sp.outline()
    plume = sp.poly([(6, 22), (25, 22), (19, 30), (16, 31), (12, 30)])
    sp.part(plume, R["plume"], "inset", glow=True)


def reactor(sp):
    for y in (3, 25):
        sp.part(sp.rect(5, y, 26, y + 4), R["steel"], "cyl_h")
    core = sp.rect(8, 8, 23, 24)
    sp.part(core, R["dark"], "cyl_v", base=2)
    for x in (8, 23):
        sp.part(sp.rect(x - 1, 8, x + 1, 24), R["white"], "cyl_v")
    sp.plots([(6, 4), (7, 4)], GLINT)
    sp.outline()
    sp.part(sp.rect(11, 10, 20, 22), R["plume"], "inset", glow=True)
    sp.plot(13, 12, GLINT, glow=True)


def fuel(sp):
    tank = sp.rect(7, 5, 24, 29) & ~sp.rect(7, 5, 7, 6) & ~sp.rect(24, 5, 24, 6)
    sp.part(tank, R["tank"], "cyl_v", base=2)
    sp.part(sp.rect(7, 14, 24, 19), R["white"], "cyl_v", base=3)
    sp.part(sp.rect(13, 1, 18, 4), R["steel"], "rim", base=2)
    flame = sp.poly([(15, 15), (18, 18), (15, 19), (13, 18)])
    sp.part(flame, R["flame"], "flat", base=2, seam=False)
    sp.plots([(9, 7), (9, 8), (10, 7)], GLINT)
    sp.outline()


def shield(sp):
    base = sp.rect(7, 24, 24, 29)
    sp.part(base, R["steel"], "rim", base=2)
    sp.part(sp.rect(13, 18, 18, 23), R["white"], "cyl_v")
    dish = sp.ellipse(6, 13, 25, 21)
    sp.part(dish, R["white"], "sphere", base=3)
    sp.part(sp.ellipse(11, 14, 20, 18), R["blue"], "rim", base=2)
    sp.outline()
    arc = sp.ellipse(1, 1, 30, 30) & ~sp.ellipse(3, 3, 28, 28) & sp.rect(0, 0, 31, 14)
    sp.fill(arc, R["blue"][3], glow=True)
    sp.fill(sp.ellipse(1, 1, 30, 30) & ~sp.ellipse(2, 2, 29, 29) & sp.rect(0, 0, 31, 6), R["cyan"][4], glow=True)
    sp.plot(15, 15, GLINT, glow=True)


def mining_laser(sp):
    body = sp.taper(3, 28, 19, 12, 10, 10)
    sp.part(body, R["hazard"], "rim", base=2)
    for t in (0.3, 0.55):
        x, y = 3 + 16 * t, 28 - 16 * t
        sp.fill(sp.taper(x - 3, y - 3, x + 3, y + 3, 2, 2) & body, R["ink"][1])
    barrel = sp.taper(17, 14, 25, 6, 6, 5)
    sp.part(barrel, R["steel"], "rim", base=2)
    sp.part(sp.taper(24, 7, 27, 4, 6, 6), R["dark"], "rim", base=2)
    sp.plots([(7, 21), (8, 20)], GLINT)
    sp.outline()
    sp.part(sp.taper(27, 4, 31, 0, 3, 2), R["flame"], "inset", glow=True)
    glint_star(sp, 29, 2, R["flame"])


def habitation(sp):
    hull = sp.rect(2, 7, 29, 25) & ~sp.rect(2, 7, 2, 8) & ~sp.rect(29, 7, 29, 8) & ~sp.rect(2, 24, 2, 25) & ~sp.rect(29, 24, 29, 25)
    sp.part(hull, R["white"], "cyl_h", base=3)
    for x in (6, 13, 20):
        sp.part(sp.rect(x, 11, x + 4, 15), R["gold"], "rim", base=3)
    sp.part(sp.rect(3, 19, 28, 20), R["blue"], "flat", base=2)
    sp.part(sp.ellipse(22, 21, 27, 24), R["green"], "rim")
    sp.plots([(4, 9), (5, 9)], GLINT)
    sp.outline()


def gyro(sp):
    ring = sp.ellipse(1, 9, 30, 23) & ~sp.ellipse(4, 12, 27, 20)
    back = ring & sp.rect(0, 0, 31, 15)
    front = ring & sp.rect(0, 16, 31, 31)
    sp.part(back, R["gold"], "rim", base=1)
    disc = sp.circle(16, 15, 9)
    sp.part(disc, R["white"], "sphere", base=3)
    sp.part(sp.circle(16, 15, 3), R["dark"], "rim", base=2)
    sp.part(front, R["gold"], "rim", base=3)
    sp.plots([(11, 9), (12, 8)], GLINT)
    sp.outline()


def hull_plate(sp):
    plate = sp.rect(3, 3, 28, 28)
    sp.part(plate, R["white"], "rim", base=3, depth=2)
    sp.fill(sp.rect(15, 4, 15, 27), R["white"][1])
    sp.fill(sp.rect(4, 15, 27, 15), R["white"][1])
    for x in (6, 12, 19, 25):
        for y in (6, 12, 19, 25):
            sp.plot(x, y, R["white"][1])
            sp.plot(x - 1, y - 1, R["white"][4])
    sp.plots([(4, 4), (5, 4), (4, 5)], GLINT)
    sp.outline()


def landing_gear(sp):
    sp.part(sp.rect(6, 2, 25, 6), R["white"], "rim", base=3)
    strut = sp.taper(16, 7, 16, 22, 5, 5)
    sp.part(strut, R["steel"], "cyl_v", base=2)
    sp.part(sp.taper(16, 7, 16, 15, 7, 7), R["dark"], "cyl_v", base=3)
    sp.part(sp.taper(10, 8, 15, 18, 3, 3), R["hazard"], "rim", base=2)
    foot = sp.poly([(5, 24), (27, 24), (29, 29), (3, 29)])
    sp.part(foot, R["dark"], "rim", base=3)
    sp.part(sp.rect(5, 24, 26, 25), R["hazard"], "flat", base=2)
    sp.plots([(7, 3), (8, 3)], GLINT)
    sp.outline()


ICONS = {
    "cockpit": cockpit, "cargo": cargo, "thruster": thruster, "reactor": reactor, "fuel": fuel,
    "shield": shield, "mining-laser": mining_laser, "habitation": habitation, "gyro": gyro,
    "hull-plate": hull_plate, "landing-gear": landing_gear,
}


# ---------------------------------------------------------------- hull textures (32 x 32 tiles)
# Each one repeats seamlessly: every pattern's period divides 32.

YS, XS = np.mgrid[0:N, 0:N]
SPECK = np.random.default_rng(7).random((N, N)) < 0.05  # fixed grain, the same every run


def tile():
    return Sprite(N, N)


def plates(sp, ramp, cell=16, rivets=True):
    """Riveted plates: a seam on the plate's top and left, a lit line inside it, a shadow line on
    its bottom and right, rivets near the corners, and a sparse dither of grain."""
    full = sp.rect(0, 0, N - 1, N - 1)
    base = len(ramp) - 2
    sp.fill(full, ramp[base])
    sp.fill(full & SPECK, ramp[base - 1])
    u, v = XS % cell, YS % cell
    sp.fill((u == 1) | (v == 1), ramp[base + 1])
    sp.fill((u == cell - 1) | (v == cell - 1), ramp[base - 1])
    sp.fill((u == 0) | (v == 0), ramp[0])
    if rivets:
        for rx, ry in [(3, 3), (cell - 4, 3), (3, cell - 4), (cell - 4, cell - 4)]:
            sp.fill((u == rx) & (v == ry), ramp[base + 1])
            sp.fill((u == rx + 1) & (v == ry + 1), ramp[1])


def tex_hull(sp):
    plates(sp, R["white"])


def tex_hull_dark(sp):
    plates(sp, R["dark"])


def tex_cargo(sp):
    ramp = R["orange"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[2])
    u = XS % 8
    sp.fill(u == 0, ramp[0])
    sp.fill(u == 1, ramp[3])
    sp.fill((u == 6) | (u == 7), ramp[1])
    band = (YS >= 22) & (YS <= 29)
    sp.fill(band & (((XS + YS) // 4) % 2 == 0), R["ink"][1])
    sp.fill(band & (((XS + YS) // 4) % 2 == 1), R["gold"][3])
    sp.fill((YS == 21) | (YS == 30), R["ink"][1])


def tex_reactor(sp):
    ramp = R["dark"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[1])
    v = YS % 16
    sp.fill(v == 0, ramp[0])
    sp.fill(v == 1, ramp[3])
    bar = (v >= 6) & (v <= 9) & ((XS % 16) >= 3) & ((XS % 16) <= 12)
    sp.fill(bar, R["plume"][2])
    sp.fill(bar & (v == 7), R["plume"][3])
    sp.fill(bar & (v == 9), R["plume"][1])
    sp.fill(((XS % 16) == 1) & ((v == 4) | (v == 12)), ramp[4])


def tex_thruster(sp):
    ramp = R["dark"]
    v = YS % 4
    sp.fill(v == 0, ramp[0])
    sp.fill(v == 1, ramp[4])
    sp.fill(v == 2, ramp[3])
    sp.fill(v == 3, ramp[2])
    sp.fill((XS % 16 == 0) & (v != 0), ramp[1])


def tex_fuel(sp):
    tank = R["tank"]
    sp.fill(sp.rect(0, 0, 31, 31), tank[2])
    sp.fill(YS % 32 == 0, tank[0])
    sp.fill(YS % 32 == 1, tank[3])
    band = (YS >= 12) & (YS <= 19)
    sp.fill(band, R["white"][3])
    sp.fill(YS == 12, R["white"][4])
    sp.fill((YS == 11) | (YS == 20), tank[0])
    sp.fill(band & ((XS % 16) >= 6) & ((XS % 16) <= 9) & (YS >= 14) & (YS <= 17), R["flame"][1])
    sp.fill(SPECK & ~band & (YS > 1), tank[1])


def hex_edges(cell_w=16, row_h=8):
    """A wrapped hexagon lattice (period 32 in x and y): True on the cell edges."""
    centres = []
    for r in range(N // row_h):
        for c in range(N // cell_w):
            centres.append((c * cell_w + (cell_w // 2 if r % 2 else 0), r * row_h))
    d = np.full((N, N, len(centres)), 1e9)
    for i, (cx, cy) in enumerate(centres):
        for ox in (-N, 0, N):
            for oy in (-N, 0, N):
                dd = np.hypot(XS + 0.5 - cx - ox, (YS + 0.5 - cy - oy) * 1.7)
                d[..., i] = np.minimum(d[..., i], dd)
    s = np.sort(d, axis=2)
    return (s[..., 1] - s[..., 0]) < 1.6


def tex_shield(sp):
    ramp = R["blue"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[1])
    sp.dither(sp.rect(0, 0, 31, 31) & ((YS % 8) < 3), ramp[1], ramp[2], 0.5)
    sp.fill(hex_edges(), ramp[3])


def tex_laser(sp):
    stripe = ((XS + YS) // 4) % 2 == 0
    sp.fill(stripe, R["gold"][3])
    sp.fill(~stripe, R["ink"][1])
    sp.fill(stripe & (((XS + YS) % 8) == 0), R["gold"][4])
    sp.fill(~stripe & (((XS + YS) % 8) == 7), R["ink"][0])


def tex_hab(sp):
    plates(sp, R["white"], cell=32, rivets=False)
    for x0 in (4, 18):
        w = sp.rect(x0, 6, x0 + 9, 13)
        sp.fill(w, R["gold"][3])
        sp.fill(sp.rect(x0, 6, x0 + 9, 6) | sp.rect(x0, 6, x0, 13), R["gold"][4])
        sp.fill(sp.rect(x0, 13, x0 + 9, 13) | sp.rect(x0 + 9, 6, x0 + 9, 13), R["gold"][1])
    sp.fill(sp.rect(0, 22, 31, 23), R["blue"][2])
    sp.fill(sp.rect(0, 24, 31, 24), R["blue"][0])


def tex_glass(sp):
    ramp = R["cyan"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[1])
    sp.dither(sp.rect(0, 0, 31, 31), ramp[1], ramp[0], 0.25)
    d = (XS + YS) % 32
    sp.fill((d == 4) | (d == 5), ramp[3])
    sp.fill(d == 11, ramp[2])
    sp.fill(((XS + YS) % 32 == 5) & (XS % 8 == 0), ramp[4])


def tex_girder(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["night"][1])
    rust = R["rust"]
    u, v = XS % 32, YS % 32
    diag = (np.abs(u - v) <= 1) | (np.abs(u + v - 31) <= 1)
    sp.fill(diag, rust[2])
    sp.fill(diag & ((u - v) == -1), rust[3])
    frame = (u <= 2) | (u >= 29) | (v <= 2) | (v >= 29)
    sp.fill(frame, rust[2])
    sp.fill((u == 0) | (v == 0), rust[3])
    sp.fill((u == 2) & (v > 2) & (v < 29) | (v == 2) & (u > 2) & (u < 29), rust[1])
    sp.fill(((u == 31) | (v == 31)), rust[0])
    for rx, ry in [(1, 1), (30, 1), (1, 30), (30, 30), (15, 15), (16, 16)]:
        sp.fill((u == rx) & (v == ry), rust[3])


TEXTURES = {
    "tex-hull": tex_hull, "tex-hull-dark": tex_hull_dark, "tex-cargo": tex_cargo, "tex-reactor": tex_reactor,
    "tex-thruster": tex_thruster, "tex-fuel": tex_fuel, "tex-shield": tex_shield, "tex-laser": tex_laser,
    "tex-hab": tex_hab, "tex-glass": tex_glass, "tex-girder": tex_girder,
}


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
        sp = tile()
        draw(sp)
        img = sp.image()
        assert img.getchannel("A").getextrema() == (255, 255), name + " has holes"
        save(img.convert("RGB"), os.path.join(OUT, name + ".png"), source="balatro-pixel-art pixelkit")
        texes.append((name, img))
    print("wrote", len(icons), "icons and", len(texes), "textures to", OUT)
    if "--sheet" in sys.argv:
        from PIL import Image
        path = sys.argv[sys.argv.index("--sheet") + 1]
        refs = sorted(p for p in glob.glob(os.path.join(SKILL, "references", "*.png")) if not p.endswith("_contact.png"))
        refs = [(os.path.basename(p)[:-4], Image.open(p).convert("RGBA")) for p in refs]
        refs = [r for r in refs if r[1].size == (56, 56)][-6:]
        # each texture shown as a 2 x 2 repeat, so a seam would show
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
