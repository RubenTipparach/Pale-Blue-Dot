"""Build-piece icons and mockup-only textures for the building mockup
(docs/mockups/building.html), also used by docs/mockups/npcs.html (the kiosk).

Pixel art drawn in code with the balatro-pixel-art kit (pixelkit), the
solitaire collection's style: a tiny native canvas, one bold subject, 3 to 5
tone ramps lit from the top left, dark seams, a 1 px outline, single-pixel
glints, every colour from Sweaft-64. Nothing here is drawn by hand.

    python tools/art/building_art.py                  # writes docs/mockups/building/*.png
    python tools/art/building_art.py --sheet <png>    # and a contact sheet beside the kit's references

Icons (icon_*.png, 32 x 32) are the palette's pieces on a transparent ground,
shown at 2x. Only the textures the game does not have are drawn here: the
habitat's metal panel and glass (tex_*.png), 32 x 32 tiles like the game's
settlement textures, built with wrap-around arithmetic so they repeat without
a seam. Every other wall, roof and ground in the mockups is the game's own
(assets/textures/settlement/, assets/tilesets/atlas.png), copied byte for
byte into docs/mockups/building/game/.
"""

import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art")
sys.path.insert(0, os.path.join(SKILL, "scripts"))
from pixelkit import Sprite, RAMPS, rgb, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "building")
WHITE = "#fffefe"
BRICK = ["#4a051e", "#841c1d", "#d4462b", "#fe6420"]
TILE = ["#2e0d06", "#733126", "#a8633e", "#e69d69"]
LIGHT = ["#c56d25", "#fb902f", "#faaf5c", "#fffefe"]
STONE = ["#434137", "#a09f91", "#bfcad7", "#fffefe"]


def darker(ramp):
    return [ramp[0]] + ramp[:-1]


def hexagon(cx, cy, r, squash=1.0):
    return [(cx + r * math.cos(k * math.pi / 3), cy + r * squash * math.sin(k * math.pi / 3)) for k in range(6)]


def spark(sp, x, y):
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], "#faaf5c", glow=True)
    sp.plot(x, y, WHITE, glow=True)


# ---------- piece icons (32 x 32) -----------------------------------------------------------

def icon_floor():
    sp = Sprite(32, 32)
    top = hexagon(15.5, 13, 14.5, 0.6)
    side = [top[0], top[1], top[2], top[3], (top[3][0], top[3][1] + 6), (top[2][0], top[2][1] + 6), (top[1][0], top[1][1] + 6), (top[0][0], top[0][1] + 6)]
    sp.part(sp.poly(side), darker(RAMPS["wood"]), "rim")
    t = sp.poly(top)
    sp.part(t, RAMPS["wood"], "rim")
    for y in (9, 13, 17):                                                           # plank joints
        sp.fill(t & sp.rect(0, y, 31, y), "#725244")
    sp.plots([(9, 11), (22, 15), (14, 19)], "#41322c")                              # nail heads
    sp.plot(8, 6, WHITE)
    sp.outline()
    return sp.image()


def icon_wall():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(26, 4), (30, 6), (30, 28), (26, 28)]), darker(BRICK), "rim")  # the wall's end
    face = sp.rect(2, 4, 26, 28)
    sp.part(face, BRICK, "rim")
    for k, y in enumerate(range(8, 28, 4)):                                         # mortar
        sp.fill(sp.rect(2, y, 26, y), "#eab798")
        off = 0 if k % 2 else 3
        for x in range(4 + off, 26, 6):
            sp.fill(sp.rect(x, y - 3, x, y - 1), "#eab798")
    sp.plot(4, 5, WHITE)
    sp.outline()
    return sp.image()


def icon_roof():
    sp = Sprite(32, 32)
    sp.part(sp.rect(4, 19, 18, 29), RAMPS["bone"], "rim")                           # the gable wall
    sp.part(sp.rect(18, 19, 29, 28), darker(RAMPS["bone"]), "rim")
    sp.part(sp.poly([(2, 20), (11, 6), (20, 20)]), RAMPS["sand"], "rim")            # the gable end
    slope = sp.poly([(11, 5), (28, 8), (31, 21), (20, 20)])
    sp.part(slope, TILE, "rim")                                                     # the long slope
    for t in (0.33, 0.66):
        sp.fill(sp.line([(11 + 9 * t, 5 + 15 * t), (28 + 3 * t, 8 + 13 * t)], 1) & slope, "#733126")
    sp.part(sp.line([(1, 21), (11, 5), (28, 8)], 2), RAMPS["wood"], "flat", base=1)  # barge boards
    sp.fill(sp.rect(9, 23, 13, 27), "#41322c")                                      # a window in the gable
    sp.plot(13, 8, WHITE)
    sp.outline()
    return sp.image()


def icon_door():
    sp = Sprite(32, 32)
    sp.part(sp.rect(6, 2, 25, 30) & ~sp.rect(6, 2, 7, 3) & ~sp.rect(24, 2, 25, 3), darker(RAMPS["wood"]), "rim")   # frame
    leaf = sp.rect(9, 5, 22, 30)
    sp.part(leaf, RAMPS["wood"], "rim")
    for x in (13, 18):
        sp.fill(sp.rect(x, 6, x, 30), "#725244")
    for y in (9, 25):
        sp.part(sp.rect(9, y, 22, y + 1), RAMPS["soot"], "flat", base=2)            # iron bands
    sp.part(sp.rect(19, 16, 20, 17), RAMPS["gold"][:4], "flat", base=3)             # the latch
    sp.plot(10, 6, "#cfaa81")
    sp.outline()
    return sp.image()


def icon_window():
    sp = Sprite(32, 32)
    sp.part(sp.rect(4, 4, 27, 26), darker(RAMPS["wood"]), "rim")                    # frame
    sp.part(sp.rect(2, 26, 29, 29), STONE, "rim")                                   # sill
    sp.part(sp.rect(7, 7, 24, 24), LIGHT, "inset")                                  # lamplight behind the glass
    sp.part(sp.rect(15, 7, 16, 24) | sp.rect(7, 15, 24, 16), RAMPS["wood"], "flat", base=1)   # mullions
    sp.plots([(9, 9), (10, 9), (9, 10)], WHITE)
    sp.outline()
    return sp.image()


def icon_stairs():
    sp = Sprite(32, 32)
    for k in range(5):
        x0, y0 = 2 + k * 5, 25 - k * 5
        sp.part(sp.rect(x0, y0, 29, y0 + 5), RAMPS["wood"] if k % 2 else darker(RAMPS["wood"]), "rim")
    sp.part(sp.rect(2, 30, 29, 30) | sp.rect(28, 5, 29, 30), darker(RAMPS["wood"]), "flat", base=1)
    sp.plot(23, 6, WHITE)
    sp.outline()
    return sp.image()


def icon_lantern():
    sp = Sprite(32, 32)
    sp.part(sp.rect(15, 0, 16, 5), RAMPS["soot"], "flat", base=2)                   # the hook
    sp.part(sp.poly([(10, 9), (13, 5), (18, 5), (21, 9)]), RAMPS["soot"], "rim")    # cap
    sp.part(sp.rect(9, 9, 22, 24), LIGHT, "inset")                                  # the lit glass
    sp.part(sp.rect(9, 9, 9, 24) | sp.rect(22, 9, 22, 24), RAMPS["soot"], "flat", base=1, seam=False)
    sp.part(sp.rect(8, 24, 23, 27), RAMPS["soot"], "rim")                           # base
    sp.plot(11, 7, "#a09f91")
    sp.outline()
    sp.part(sp.poly([(14, 21), (14, 16), (15.5, 11), (17, 16), (17, 21)]), ["#a61e1f", "#fe6420", "#fffefe"], "inset", glow=True)   # the flame
    spark(sp, 4, 12)
    spark(sp, 27, 17)
    spark(sp, 26, 4)
    return sp.image()


def icon_airlock():
    sp = Sprite(32, 32)
    body = sp.rect(2, 2, 29, 29) & ~sp.rect(2, 2, 3, 3) & ~sp.rect(28, 2, 29, 3) & ~sp.rect(2, 28, 3, 29) & ~sp.rect(28, 28, 29, 29)
    sp.part(body, RAMPS["metal"], "rim", base=3)
    sp.part(sp.circle(15.5, 15.5, 10.5), RAMPS["metal"], "rim", base=1)             # the ring
    sp.part(sp.circle(15.5, 15.5, 7.5), RAMPS["glass"], "sphere")                   # its port
    sp.part(sp.rect(15, 8, 16, 23) | sp.rect(8, 15, 23, 16), RAMPS["gold"][:4], "flat", base=2)   # locking bars
    sp.part(sp.rect(14, 14, 17, 17), RAMPS["gold"][:4], "rim")
    for k in range(8):                                                              # bolts
        a = k * math.pi / 4 + math.pi / 8
        sp.plot(round(15.5 + 11.5 * math.cos(a)), round(15.5 + 11.5 * math.sin(a)), "#bfcad7")
    for y in range(4, 10, 2):                                                       # hazard stripes
        sp.fill(sp.rect(4, y, 6, y), "#fb902f")
        sp.fill(sp.rect(25, y, 27, y), "#fb902f")
    sp.plots([(11, 11), (12, 11), (11, 12)], "#c1f1d8")
    sp.outline()
    return sp.image()


def icon_post():
    sp = Sprite(32, 32)
    sp.part(sp.poly(hexagon(15.5, 27, 9, 0.4)) | sp.rect(7, 27, 24, 29), STONE, "rim", base=1)   # footing
    sp.part(sp.rect(13, 2, 18, 27), RAMPS["wood"], "cyl_v")                         # the post
    for y in (8, 19):
        sp.part(sp.rect(12, y, 19, y + 1), RAMPS["soot"], "flat", base=2)
    sp.plot(14, 3, WHITE)
    sp.outline()
    return sp.image()


# ---------- textures (32 x 32, wrapping) ----------------------------------------------------

def tile(fn):
    """A 32 x 32 tile from fn(x, y) -> '#rrggbb'; indices wrap, so it repeats seamlessly."""
    a = np.zeros((32, 32, 4), np.uint8)
    for y in range(32):
        for x in range(32):
            a[y, x, :3] = rgb(fn(x, y))
            a[y, x, 3] = 255
    return Image.fromarray(a, "RGBA")


def tex_metal():
    """Four riveted 16 px plates: a dark seam on each plate's first row and column, a lit top and
    left lip, a shadowed bottom and right, rivets in the corners, a few scuffs."""
    seam, shadow, body, lit, rivet = "#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"
    scuffs = {(5, 8), (6, 8), (22, 26), (23, 26), (24, 27), (27, 6), (9, 21)}

    def px(x, y):
        u, v = x % 16, y % 16
        if u == 0 or v == 0:
            return seam
        if (u, v) in ((3, 3), (13, 3), (3, 13), (13, 13)):
            return rivet
        if (u, v) in ((3, 4), (13, 4), (3, 14), (13, 14), (4, 3), (14, 3), (4, 13), (14, 13)):
            return shadow
        if u == 15 or v == 15:
            return shadow
        if u == 1 or v == 1:
            return lit
        if (x, y) in scuffs:
            return lit
        if x >= 17 and y <= 15 and 6 <= v <= 7 and 4 <= u <= 11:                   # a stencilled label band
            return "#fb902f" if (u + v) % 4 < 2 else "#c56d25"
        return body
    return tile(px)


def tex_glass():
    """Glazing: two 16 px panes in a dark metal frame, deep teal glass, a diagonal sheen and a
    single bright glint on each pane."""
    frame, frame_lit, deep, body, sheen, bright = "#1f3b52", "#3e586e", "#0c2f44", "#204f63", "#53998b", "#84e3b5"

    def px(x, y):
        u, v = x % 16, y % 16
        if u == 0 or v == 0:
            return frame
        if u == 1 and v > 0 or v == 1 and u > 0:
            return frame_lit
        d = (u + v) % 16
        if d in (9, 10) and 3 < u < 14:
            return sheen
        if d == 12 and 6 < u < 13:
            return sheen
        if (u, v) in ((4, 4), (5, 4), (4, 5)):
            return bright
        if u >= 13 or v >= 13:
            return deep
        return body
    return tile(px)


ICONS = {
    "icon_floor": icon_floor, "icon_wall": icon_wall, "icon_roof": icon_roof, "icon_door": icon_door,
    "icon_window": icon_window, "icon_stairs": icon_stairs, "icon_lantern": icon_lantern, "icon_airlock": icon_airlock,
    "icon_post": icon_post, "tex_metal": tex_metal, "tex_glass": tex_glass,
}
REFS = ["armour_plating", "afterburner_kit", "emp_burst", "drop_tanks", "cluster_bombs"]


def main():
    os.makedirs(OUT, exist_ok=True)
    items = []
    for name, fn in ICONS.items():
        img = fn()
        p = os.path.join(OUT, name + ".png")
        save(img, p, source="pixelkit (tools/art/building_art.py)")
        items.append((name, img))
        print("wrote", p)
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1]
        tiled = [(n + " 2x2", Image.fromarray(np.tile(np.array(im), (2, 2, 1)), "RGBA")) for n, im in items if n.startswith("tex_")]
        refs = [("ref " + r, Image.open(os.path.join(SKILL, "references", r + ".png")).convert("RGBA")) for r in REFS]
        sheet(items + tiled + refs, scale=4, cols=6, cell=(64 * 4, 64 * 4)).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main()
