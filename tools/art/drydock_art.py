"""Block icons and the mockup-only textures for the drydock mockup (docs/mockups/drydock.html).

Pixel art drawn in code with the balatro-pixel-art kit (`.claude/skills/balatro-pixel-art`), the
solitaire collection's style, every colour from Sweaft-64 (the kit's save() refuses anything else).

- block icons: 32 x 32, transparent, one bold subject, 1 px outline, light from the top left, shown
  only at whole multiples of 32 px with `image-rendering: pixelated`;
- textures: 32 x 32 tiles that repeat seamlessly, opaque, no outline, like the game's settlement
  textures, sampled nearest with no mipmaps.

What the game already has is not drawn here: the deck is the game's plank, the keel and masts its
timber, the dock its granite and stone, the gate its iron (copied byte for byte into
docs/mockups/drydock/game/). Only the tarred hull strakes, the seamed sail canvas, the gantry's
painted lattice and the cabin's window wall are new.

    python tools/art/drydock_art.py
    python tools/art/drydock_art.py --sheet OUT.png
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

OUT = os.path.join(ROOT, "docs", "mockups", "drydock")
N = 32
R = dict(RAMPS)
R.update({
    "tar": ["#010e1a", "#161612", "#2c2c25", "#434137"],
    "hull": ["#2e0d06", "#41322c", "#725244", "#ab8465"],
    "canvas": ["#725244", "#ab8465", "#cfaa81", "#eab798"],
    "stone": ["#2c2c25", "#434137", "#a09f91", "#bfcad7"],
    "paint": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "sea": ["#0c2f44", "#15578a", "#1096c3", "#74effe"],
})
GLINT = "#fffefe"
YS, XS = np.mgrid[0:N, 0:N]


def hexagon(sp, cx, cy, r, squash=1.0):
    pts = [(cx + r * np.cos(np.radians(60 * i + 30)), cy + r * squash * np.sin(np.radians(60 * i + 30))) for i in range(6)]
    return sp.poly([(int(round(x)), int(round(y))) for x, y in pts])


def hex_block(sp, top, side, dark=None, h=10):
    """A hex column seen a little from above: a top hexagon and two side faces below it."""
    top_m = hexagon(sp, 16, 11, 13, 0.62)
    body = sp.poly([(3, 11), (3, 11 + h), (16, 19 + h), (29, 11 + h), (29, 11), (16, 19)])
    left = body & (XS <= 16)
    right = body & (XS > 16)
    sp.part(left, side, "flat", base=2)
    sp.part(right, dark or side, "flat", base=1)
    sp.part(top_m, top, "rim", base=2)
    return top_m, left, right


# ---------------------------------------------------------------- block icons

def keel(sp):
    _, l, r = hex_block(sp, R["wood"], R["wood"], h=12)
    for y in (16, 21, 26):
        sp.fill(sp.line([(3, y - 4), (16, y + 4)], 1) & l, R["wood"][0])
    sp.fill(sp.rect(14, 2, 18, 9), R["wood"][1])
    sp.plots([(9, 7), (10, 7)], GLINT)
    sp.outline()


def hull(sp):
    _, l, r = hex_block(sp, R["wood"], R["tar"], h=13)
    for k in range(4):
        sp.fill(sp.line([(3, 13 + k * 4), (16, 21 + k * 4)], 1) & l, R["tar"][3])
        sp.fill(sp.line([(16, 21 + k * 4), (29, 13 + k * 4)], 1) & r, R["tar"][2])
    sp.plots([(8, 8), (9, 8)], GLINT)
    sp.outline()


def ballast(sp):
    hex_block(sp, R["stone"], R["stone"], h=10)
    for cx, cy in [(10, 10), (17, 8), (22, 12), (14, 13)]:
        sp.part(sp.ellipse(cx - 3, cy - 2, cx + 3, cy + 2), R["stone"], "sphere")
    sp.plots([(9, 9)], GLINT)
    sp.outline()


def bulkhead(sp):
    sp.part(sp.rect(5, 3, 26, 28), R["wood"], "rim", base=2)
    for x in (10, 15, 20):
        sp.fill(sp.rect(x, 4, x, 27), R["wood"][0])
    sp.part(sp.rect(5, 12, 26, 14), R["steel"], "rim")
    sp.part(sp.rect(5, 22, 26, 24), R["steel"], "rim")
    sp.plots([(7, 5), (8, 5)], GLINT)
    sp.outline()


def deck(sp):
    top, _, _ = hex_block(sp, R["sand"], R["wood"], h=5)
    for k in range(-3, 4):
        sp.fill(sp.line([(16 + k * 4 - 6, 4), (16 + k * 4 + 6, 18)], 1) & top, R["sand"][0])
    sp.plots([(11, 7)], GLINT)
    sp.outline()


def hold(sp):
    sp.part(sp.rect(4, 6, 27, 28), R["wood"], "rim", base=1)
    sp.fill(sp.rect(6, 8, 25, 26), R["tar"][1])
    sp.part(sp.ellipse(7, 15, 15, 26), R["wood"], "cyl_v", base=2)
    sp.fill(sp.rect(7, 18, 15, 18) | sp.rect(7, 23, 15, 23), R["steel"][0])
    sp.part(sp.rect(17, 17, 25, 26), R["sand"], "rim", base=2)
    sp.fill(sp.line([(17, 17), (25, 26)], 1), R["sand"][0])
    sp.part(sp.rect(18, 9, 24, 15), R["paint"], "rim", base=2)
    sp.plots([(8, 16)], GLINT)
    sp.outline()


def cabin(sp):
    sp.part(sp.poly([(3, 13), (16, 3), (29, 13)]), R["wood"], "rim", base=1)
    sp.part(sp.rect(5, 13, 27, 28), R["paper"], "rim", base=1)
    sp.part(sp.rect(9, 16, 15, 22), R["gold"], "inset", base=3, glow=False)
    sp.fill(sp.rect(12, 16, 12, 22) | sp.rect(9, 19, 15, 19), R["wood"][0])
    sp.part(sp.rect(18, 17, 23, 28), R["wood"], "rim", base=1)
    sp.plots([(10, 17)], GLINT)
    sp.outline()


def hatch(sp):
    sp.part(sp.rect(3, 6, 28, 27), R["sand"], "rim", base=2)
    sp.fill(sp.rect(8, 10, 23, 23), R["ink"][1])
    for y in (12, 16, 20):
        sp.part(sp.rect(10, y, 21, y + 1), R["wood"], "flat", base=3)
    sp.fill(sp.rect(10, 10, 11, 23) | sp.rect(20, 10, 21, 23), R["wood"][2])
    sp.plots([(5, 8), (6, 8)], GLINT)
    sp.outline()


def mast(sp):
    sp.part(sp.rect(13, 2, 18, 29), R["wood"], "cyl_v", base=2)
    sp.part(sp.rect(4, 7, 27, 9), R["wood"], "cyl_h", base=1)
    sp.part(sp.rect(11, 2, 20, 4), R["wood"], "rim", base=1)
    sp.part(sp.rect(10, 26, 21, 29), R["steel"], "rim")
    sp.plots([(14, 12), (14, 13)], GLINT)
    sp.outline()


def sail(sp):
    cloth = sp.poly([(5, 5), (27, 5), (29, 27), (16, 29), (3, 27)])
    sp.part(cloth, R["canvas"], "rim", base=2)
    for x in (10, 16, 22):
        sp.fill(sp.line([(x, 6), (x, 27)], 1) & cloth, R["canvas"][1])
    sp.part(sp.rect(2, 3, 29, 5), R["wood"], "cyl_h", base=1)
    sp.fill(sp.rect(5, 17, 27, 17) & cloth, R["canvas"][1])
    sp.plots([(8, 8), (9, 8)], GLINT)
    sp.outline()


def rudder(sp):
    blade = sp.poly([(10, 4), (17, 4), (26, 18), (24, 29), (10, 29)])
    sp.part(blade, R["tar"], "rim", base=2)
    for y in (10, 16, 22):
        sp.fill(sp.rect(10, y, 25, y) & blade, R["tar"][3])
    sp.part(sp.rect(6, 6, 11, 8) | sp.rect(6, 15, 11, 17) | sp.rect(6, 24, 11, 26), R["steel"], "rim")
    sp.part(sp.rect(13, 1, 15, 4), R["wood"], "cyl_v", base=2)
    sp.plots([(12, 6)], GLINT)
    sp.outline()


def helm(sp):
    ring = sp.circle(16, 15, 11) & ~sp.circle(16, 15, 8)
    spokes = sp.empty()
    for a in range(0, 180, 45):
        dx, dy = np.cos(np.radians(a)) * 14, np.sin(np.radians(a)) * 14
        spokes |= sp.line([(int(16 - dx), int(15 - dy)), (int(16 + dx), int(15 + dy))], 2)
    sp.part(spokes & ~sp.circle(16, 15, 11), R["wood"], "rim", base=2)
    sp.part(ring, R["wood"], "rim", base=2)
    sp.part(spokes & sp.circle(16, 15, 8), R["wood"], "flat", base=1)
    sp.part(sp.circle(16, 15, 3), R["gold"], "sphere", base=2)
    sp.part(sp.rect(14, 26, 18, 30), R["wood"], "rim", base=1)
    sp.plots([(9, 7), (15, 14)], GLINT)
    sp.outline()


def capstan(sp):
    sp.part(sp.rect(9, 8, 22, 28), R["wood"], "cyl_v", base=2)
    sp.part(sp.ellipse(7, 4, 24, 11), R["wood"], "rim", base=2)
    sp.part(sp.rect(2, 6, 29, 8), R["wood"], "cyl_h", base=1)
    sp.part(sp.rect(9, 14, 22, 16) | sp.rect(9, 21, 22, 23), R["steel"], "rim")
    sp.part(sp.rect(23, 18, 28, 29), R["steel"], "rim")
    sp.fill(sp.rect(24, 20, 27, 20) | sp.rect(24, 24, 27, 24), R["steel"][0])
    sp.plots([(11, 9)], GLINT)
    sp.outline()


def rail(sp):
    for x in (4, 15, 26):
        sp.part(sp.rect(x, 9, x + 2, 29), R["wood"], "cyl_v", base=2)
    sp.part(sp.rect(2, 7, 29, 10), R["wood"], "cyl_h", base=2)
    sp.part(sp.rect(2, 18, 29, 19), R["wood"], "cyl_h", base=1)
    sp.plots([(6, 8), (7, 8)], GLINT)
    sp.outline()


def lantern(sp):
    sp.part(sp.rect(14, 1, 17, 4), R["steel"], "rim")
    sp.part(sp.poly([(9, 8), (22, 8), (19, 5), (12, 5)]), R["steel"], "rim")
    sp.part(sp.rect(9, 9, 22, 24), R["steel"], "rim", base=1)
    sp.part(sp.rect(11, 10, 20, 23), R["flame"], "inset", glow=False)
    sp.part(sp.rect(8, 24, 23, 27), R["steel"], "rim")
    sp.fill(sp.rect(15, 10, 16, 23), R["steel"][0])
    sp.outline()
    sp.plots([(13, 14), (13, 15)], GLINT, glow=True)


def gangway(sp):
    plank = sp.poly([(2, 23), (24, 6), (30, 13), (8, 30)])
    sp.part(plank, R["wood"], "rim", base=2)
    for k in range(5):
        x0, y0 = 7 + k * 4, 24 - k * 3
        sp.fill(sp.line([(x0, y0), (x0 + 3, y0 + 3)], 1) & plank, R["wood"][0])
    sp.part(sp.line([(5, 20), (24, 4)], 2), R["wood"], "flat", base=1)
    sp.part(sp.rect(4, 19, 5, 27) | sp.rect(23, 3, 24, 10), R["wood"], "flat", base=1)
    sp.plots([(10, 20)], GLINT)
    sp.outline()


ICONS = {
    "keel": keel, "hull": hull, "ballast": ballast, "bulkhead": bulkhead, "deck": deck, "hold": hold,
    "cabin": cabin, "hatch": hatch, "mast": mast, "sail": sail, "rudder": rudder, "helm": helm,
    "capstan": capstan, "rail": rail, "lantern": lantern, "gangway": gangway,
}


# ---------------------------------------------------------------- textures (32 x 32, seamless)

SPECK = np.random.default_rng(11).random((N, N)) < 0.06


def tile():
    return Sprite(N, N)


def tex_strakes(sp):
    """Tarred hull strakes: 8 px planks (a 2 m repeat is four strakes of 0.5 m), caulked seams,
    butts staggered strake by strake, treenails."""
    ramp = R["tar"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[2])
    sp.fill(SPECK, ramp[1])
    v = YS % 8
    sp.fill(v == 0, ramp[0])
    sp.fill(v == 1, ramp[3])
    sp.fill(v == 7, ramp[1])
    strake = YS // 8
    butt = (XS == (strake * 11) % 32)
    sp.fill(butt & (v != 0), ramp[0])
    sp.fill(((XS + strake * 5) % 16 == 3) & (v == 4), R["hull"][2])


def tex_sail(sp):
    """Sail canvas: cloths 8 px wide sewn with darker seams, a reef band, a faint dither of weave."""
    ramp = R["canvas"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[2])
    sp.dither(sp.rect(0, 0, 31, 31), ramp[2], ramp[1], 0.12)
    sp.fill(XS % 8 == 0, ramp[1])
    sp.fill(XS % 8 == 1, ramp[3])
    sp.fill(YS % 32 == 22, ramp[1])
    sp.fill((YS % 32 == 21) & (XS % 4 == 2), R["wood"][1])


def tex_gantry(sp):
    """The gantry's painted lattice: yellow steel, a diagonal web, rivets, rust at the joints."""
    sp.fill(sp.rect(0, 0, 31, 31), R["night"][1])
    p = R["paint"]
    u, v = XS % 32, YS % 32
    diag = (np.abs(u - v) <= 1) | (np.abs(u + v - 31) <= 1)
    sp.fill(diag, p[2])
    sp.fill(diag & ((u - v) == -1), p[3])
    frame = (u <= 2) | (u >= 29) | (v <= 2) | (v >= 29)
    sp.fill(frame, p[2])
    sp.fill((u == 0) | (v == 0), p[3])
    sp.fill((u == 31) | (v == 31), p[0])
    for rx, ry in [(1, 1), (30, 1), (1, 30), (30, 30), (15, 15), (16, 16)]:
        sp.fill((u == rx) & (v == ry), R["rust"][1])


def tex_window(sp):
    """A deckhouse's whitewashed planking with one small window, its panes dark by day (the page
    lights them at night with this texture as the emissive map)."""
    ramp = R["paper"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[1])
    sp.fill(YS % 8 == 7, ramp[0])
    sp.fill(SPECK & (YS % 8 != 7), ramp[0])
    w = sp.rect(10, 9, 21, 20)
    sp.fill(w, R["wood"][1])
    sp.fill(sp.rect(12, 11, 19, 18), R["gold"][3])
    sp.fill(sp.rect(15, 11, 16, 18) | sp.rect(12, 14, 19, 15), R["wood"][1])
    sp.fill(sp.rect(12, 11, 14, 13), R["gold"][4])


TEXTURES = {"tex-strakes": tex_strakes, "tex-sail": tex_sail, "tex-gantry": tex_gantry, "tex-window": tex_window}


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
        refs = [r for r in refs if r[1].size == (56, 56)][-4:]
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
