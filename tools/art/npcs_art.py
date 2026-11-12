"""Portraits and UI icons for the NPC mockup (docs/mockups/npcs.html).

Pixel art drawn in code with the balatro-pixel-art kit (pixelkit), the
solitaire collection's style: a tiny native canvas, one bold subject, 3 to 5
tone ramps lit from the top left, dark seams, a 1 px outline, single-pixel
glints, every colour from Sweaft-64. Nothing here is drawn by hand.

    python tools/art/npcs_art.py                  # writes docs/mockups/npcs/*.png
    python tools/art/npcs_art.py --sheet <png>    # and a contact sheet beside the kit's references

A portrait (48 x 48) is a bust of a townsperson as the townsfolk change
builds them: hexagonal prisms that read as cubes (owner, survey T2), so the
head is a flat-fronted block, not a round face. Each role carries the one
thing that says what it does (a hammer, a spear, a datapad). The page shows
portraits at 1x or 2x and the UI icons (16 x 16) at 2x, nearest neighbour.
"""

import os
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art")
sys.path.insert(0, os.path.join(SKILL, "scripts"))
from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "npcs")

WHITE = "#fffefe"
SKIN = {
    "pale": ["#a8633e", "#e69d69", "#eab798", "#fffefe"],
    "light": ["#7d4636", "#a8633e", "#e69d69", "#eab798"],
    "tan": ["#522b2f", "#7d4636", "#a8633e", "#e69d69"],
    "dark": ["#2f1525", "#522b2f", "#7d4636", "#a8633e"],
}
HAIR = {
    "brown": ["#2e0d06", "#41322c", "#725244", "#ab8465"],
    "black": ["#010000", "#161612", "#2c2c25", "#434137"],
    "red": ["#4a051e", "#841c1d", "#d4462b", "#fe6420"],
    "ginger": ["#733126", "#8d4526", "#c56d25", "#fb902f"],
}
CLOTH = {
    "green": RAMPS["green"],
    "leather": ["#2e0d06", "#522b2f", "#7d4636", "#a8633e"],
    "plum": ["#2b1f35", "#4f1d83", "#9e3ba2", "#c34988"],
    "blue": RAMPS["blue"],
    "navy": ["#010e1a", "#191954", "#15578a", "#1096c3"],
    "olive": ["#2c2c25", "#434137", "#559642", "#9bb675"],
    "cyan": ["#0c2f44", "#204f63", "#1096c3", "#74effe"],
    "dusk": ["#170916", "#2b1f35", "#3e3551", "#69608b"],
    "yellow": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "straw": ["#725244", "#ab8465", "#cfaa81", "#eab798"],
    "crimson": ["#4a051e", "#841c1d", "#a61e1f", "#d12d41"],
}


def darker(ramp):
    return [ramp[0]] + ramp[:-1]


def spark(sp, x, y):
    """A white pixel in a plus of the palest warm tone; light, so never outlined."""
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], "#faaf5c", glow=True)
    sp.plot(x, y, WHITE, glow=True)


# ---------- a bust ---------------------------------------------------------------------

def torso(sp, shirt, wide=0):
    s = wide
    sp.part(sp.poly([(3 - s, 47), (3 - s, 41), (9 - s, 35), (38 + s, 35), (44 + s, 41), (44 + s, 47)]), shirt, "rim")
    return shirt


def head(sp, skin, hair=None, style="cap", mouth="smile", eyes="#011c34", top=12):
    """Neck, block head, hair, eyes and mouth. `top` is the head's top row."""
    sp.part(sp.rect(20, 29, 27, 35), darker(skin), "flat")
    sp.part(sp.rect(15, top, 32, 30), skin, "rim")
    if hair:
        if style in ("cap", "long", "beard"):
            sp.part(sp.rect(14, top - 2, 33, top + 3), hair, "rim")
        if style == "long":
            sp.part(sp.rect(12, top + 2, 15, 32), hair, "rim")
            sp.part(sp.rect(32, top + 2, 35, 32), hair, "rim")
        if style == "beard":
            sp.part(sp.poly([(15, 25), (32, 25), (31, 30), (27, 33), (20, 33), (16, 30)]), hair, "rim")
    # eyes: square pupils, the pixel figures' own
    for x in (18, 27):
        sp.fill(sp.rect(x, 19, x + 2, 21), WHITE)
        sp.fill(sp.rect(x + 1, 20, x + 2, 21), eyes)
    lip = skin[0]
    if style != "beard":
        if mouth == "smile":
            sp.plots([(21, 25), (22, 26), (23, 26), (24, 26), (25, 26), (26, 25)], lip)
        elif mouth == "flat":
            sp.plots([(x, 26) for x in range(21, 27)], lip)


# ---------- the roles ------------------------------------------------------------------

def farmer():
    sp = Sprite(48, 48)
    sp.part(sp.line([(41, 47), (41, 11)], 2), RAMPS["wood"], "cyl_v")              # pitchfork shaft
    sp.part(sp.rect(36, 9, 46, 11), RAMPS["steel"], "rim")
    for x in (36, 40, 44):
        sp.part(sp.rect(x, 2, x + 1, 9), RAMPS["steel"], "flat", base=3)
    torso(sp, CLOTH["green"])
    sp.part(sp.rect(17, 37, 30, 47), CLOTH["blue"], "rim")                          # dungaree bib
    sp.plots([(18, 38), (29, 38)], "#faaf5c")
    head(sp, SKIN["light"], HAIR["brown"])
    sp.part(sp.ellipse(7, 11, 40, 16), CLOTH["straw"], "rim")                       # straw hat brim
    sp.part(sp.rect(15, 3, 32, 12) & ~sp.rect(15, 3, 15, 3) & ~sp.rect(32, 3, 32, 3), CLOTH["straw"], "cyl_v")
    sp.part(sp.rect(15, 9, 32, 10), CLOTH["crimson"], "flat", base=2)
    sp.plot(17, 5, WHITE)
    sp.outline()
    return sp.image()


def smith():
    sp = Sprite(48, 48)
    torso(sp, CLOTH["leather"], wide=2)
    sp.part(sp.rect(15, 37, 32, 47), RAMPS["rust"], "rim")                          # leather apron
    sp.part(sp.line([(15, 37), (19, 35)], 2) | sp.line([(32, 37), (28, 35)], 2), HAIR["brown"], "flat")
    head(sp, SKIN["dark"], HAIR["black"], style="beard")
    sp.part(sp.line([(40, 42), (37, 13)], 2), RAMPS["wood"], "cyl_v")              # hammer handle
    sp.part(sp.rect(36, 38, 42, 44), SKIN["dark"], "rim")                           # fist
    sp.part(sp.rect(31, 7, 44, 14), RAMPS["metal"], "cyl_h")                        # hammer head
    sp.plot(33, 8, WHITE)
    sp.outline()
    spark(sp, 45, 4)
    spark(sp, 28, 3)
    spark(sp, 46, 19)
    return sp.image()


def trader():
    sp = Sprite(48, 48)
    torso(sp, CLOTH["plum"])
    sp.part(sp.poly([(8, 37), (13, 35), (34, 47), (27, 47)]), RAMPS["gold"][:4], "rim")   # sash
    head(sp, SKIN["tan"], HAIR["black"])
    sp.part(sp.ellipse(12, 3, 35, 17), RAMPS["gold"][:4], "cyl_h")                  # turban
    sp.part(sp.line([(14, 12), (23, 8), (33, 13)], 1), ["#8d4526"], "flat", seam=False)
    sp.part(sp.rect(22, 9, 25, 12), RAMPS["cyan"], "sphere")                        # its jewel
    sp.plot(22, 9, WHITE)
    sp.part(sp.ellipse(34, 36, 46, 47), RAMPS["wood"], "sphere")                    # coin pouch
    sp.part(sp.rect(36, 36, 44, 37), HAIR["brown"], "flat")
    for (x, y) in ((1, 38), (4, 42), (0, 44)):
        sp.part(sp.ellipse(x, y, x + 5, y + 4), RAMPS["gold"][:4], "rim")
        sp.plot(x + 1, y + 1, WHITE)
    sp.outline()
    spark(sp, 6, 33)
    return sp.image()


def guard():
    sp = Sprite(48, 48)
    sp.part(sp.rect(40, 9, 41, 47), RAMPS["wood"], "cyl_v")                         # spear shaft
    sp.part(sp.poly([(40.5, 0), (44, 7), (40.5, 11), (37, 7)]), RAMPS["metal"], "rim", base=3)
    torso(sp, CLOTH["blue"])
    sp.part(sp.rect(17, 36, 30, 47), CLOTH["crimson"], "rim")                       # tabard
    sp.part(sp.star(23.5, 41.5, 5, 2.2), RAMPS["gold"][:4], "flat", base=3)
    head(sp, SKIN["light"], mouth="flat")
    helm = sp.ellipse(13, 5, 34, 29) & sp.rect(13, 5, 34, 17)
    sp.part(helm, RAMPS["metal"], "sphere", base=3)                                 # steel cap
    sp.part(sp.rect(13, 16, 34, 17), RAMPS["metal"], "flat", base=2)
    sp.part(sp.rect(23, 17, 24, 24), RAMPS["metal"], "flat", base=3)                # nasal
    sp.plot(18, 8, WHITE)
    sp.plot(40, 3, WHITE)
    sp.outline()
    return sp.image()


def fisher():
    sp = Sprite(48, 48)
    torso(sp, CLOTH["navy"])
    head(sp, SKIN["pale"], HAIR["ginger"], style="beard")
    sp.part(sp.poly([(9, 15), (13, 6), (34, 6), (38, 15), (38, 17), (9, 17)]), CLOTH["yellow"], "rim")   # sou'wester
    sp.part(sp.rect(10, 15, 37, 17), CLOTH["yellow"], "flat", base=1)
    sp.plot(15, 8, WHITE)
    for y in (39, 43):                                                              # oilskin toggles
        sp.part(sp.rect(23, y, 24, y + 1), CLOTH["yellow"], "flat", base=3)
    sp.part(sp.poly([(33, 37), (29, 33), (29, 42)]), darker(RAMPS["cyan"]), "rim")  # the fish's tail
    sp.part(sp.ellipse(32, 34, 47, 42), RAMPS["cyan"], "cyl_h")                     # the fish
    sp.fill(sp.rect(43, 36, 44, 37), "#010e1a")
    sp.plot(43, 36, WHITE)
    sp.outline()
    sp.plots([(40, 30), (44, 28), (36, 29)], "#74effe", glow=True)                  # drips
    return sp.image()


def miner():
    sp = Sprite(48, 48)
    sp.part(sp.line([(4, 46), (40, 12)], 2), RAMPS["wood"], "rim")                 # pickaxe handle
    sp.part(sp.poly([(30, 4), (39, 7), (46, 15), (46, 20), (43, 19), (38, 12), (30, 8)]), RAMPS["steel"], "rim")
    torso(sp, CLOTH["olive"])
    sp.part(sp.line([(12, 36), (19, 47)], 2), HAIR["brown"], "flat")               # strap
    sp.part(sp.rect(31, 40, 36, 45), CLOTH["yellow"], "rim")                        # pouch
    head(sp, SKIN["dark"], mouth="flat")
    sp.plots([(17, 24), (18, 24), (30, 25), (29, 26)], "#2f1525")                  # soot
    dome = sp.ellipse(13, 3, 34, 27) & sp.rect(13, 3, 34, 15)
    sp.part(dome, CLOTH["yellow"], "sphere")                                        # hard hat
    sp.part(sp.rect(11, 14, 36, 16), CLOTH["yellow"], "flat", base=1)
    sp.part(sp.rect(21, 6, 26, 11), RAMPS["soot"], "rim")                           # lamp housing
    sp.outline()
    sp.part(sp.rect(22, 7, 25, 10), ["#faaf5c", "#fffefe"], "inset", glow=True)    # the lamp, lit
    return sp.image()


def clerk():
    sp = Sprite(48, 48)
    torso(sp, CLOTH["cyan"])
    sp.part(sp.line([(15, 36), (23, 40), (32, 36)], 2), RAMPS["paper"], "flat", base=2)   # collar stripe
    head(sp, SKIN["pale"], HAIR["red"], style="long")
    sp.part(sp.line([(13, 20), (13, 10), (23, 7), (34, 10), (34, 20)], 2), RAMPS["ink"], "flat", base=2)  # headset
    sp.part(sp.rect(11, 18, 15, 25), RAMPS["soot"], "rim")
    sp.part(sp.line([(14, 25), (19, 29)], 2), RAMPS["ink"], "flat", base=2)
    sp.part(sp.rect(32, 33, 46, 46), RAMPS["soot"], "rim")                          # datapad
    sp.plot(33, 34, "#a09f91")
    sp.outline()
    sp.plots([(19, 29), (20, 29), (19, 30), (20, 30)], "#84e3b5", glow=True)        # mic light
    sp.part(sp.rect(34, 35, 44, 44), ["#53998b", "#84e3b5"], "flat", base=0, glow=True)
    for k, w in enumerate((8, 6, 4)):
        sp.fill(sp.rect(35, 36 + k * 3, 35 + w, 36 + k * 3), "#c1f1d8", glow=True)
    return sp.image()


def smuggler():
    sp = Sprite(48, 48)
    torso(sp, RAMPS["soot"])
    sp.part(sp.poly([(9, 36), (10, 15), (15, 6), (32, 6), (37, 15), (38, 36)]), CLOTH["dusk"], "rim")   # hood
    sp.part(sp.rect(16, 14, 31, 30), SKIN["tan"], "rim")                            # the face in it
    for x in (18, 27):
        sp.fill(sp.rect(x, 19, x + 2, 21), "#2c2c25")
    sp.part(sp.rect(15, 24, 32, 31), CLOTH["crimson"], "rim")                       # scarf
    sp.part(sp.rect(31, 36, 46, 47), RAMPS["wood"], "rim")                          # a crate
    sp.fill(sp.line([(32, 37), (45, 46)], 2) | sp.line([(45, 37), (32, 46)], 2), "#41322c")
    sp.outline()
    for x in (18, 27):                                                              # eyes that catch the light
        sp.fill(sp.rect(x + 1, 20, x + 2, 21), "#9bb675", glow=True)
        sp.plot(x + 1, 20, "#c1f1d8", glow=True)
    spark(sp, 44, 32)
    return sp.image()


# ---------- UI icons (16 x 16) -----------------------------------------------------------

def talk():
    sp = Sprite(16, 16)
    sp.part(sp.ellipse(1, 1, 14, 11) | sp.poly([(3, 9), (2, 14), (8, 10)]), RAMPS["cyan"], "rim", base=2)
    for x in (4, 7, 10):
        sp.fill(sp.rect(x, 5, x + 1, 6), WHITE)
    sp.outline()
    return sp.image()


def quest():
    sp = Sprite(16, 16)
    sp.part(sp.rect(3, 2, 12, 13), RAMPS["bone"], "rim")
    sp.part(sp.rect(1, 1, 14, 3), RAMPS["wood"], "cyl_h")
    sp.part(sp.rect(1, 12, 14, 14), RAMPS["wood"], "cyl_h")
    sp.fill(sp.rect(7, 5, 8, 8), "#a61e1f")
    sp.fill(sp.rect(7, 10, 8, 10), "#a61e1f")
    sp.outline()
    return sp.image()


def trade():
    sp = Sprite(16, 16)
    sp.part(sp.circle(10, 5, 4.6), RAMPS["metal"], "sphere", base=3)               # a silver coin behind
    sp.part(sp.circle(6, 9.5, 5.4), RAMPS["gold"][:4], "rim")                       # a gold one in front
    sp.part(sp.circle(6, 9.5, 2.6), RAMPS["gold"][:4], "flat", base=1)
    sp.plot(3, 6, WHITE)
    sp.plot(9, 2, WHITE)
    sp.outline()
    return sp.image()


def heart():
    sp = Sprite(16, 16)
    sp.part(sp.ellipse(1, 2, 8, 9) | sp.ellipse(7, 2, 14, 9) | sp.poly([(1, 6), (14, 6), (7.5, 14)]), RAMPS["red"], "sphere")
    sp.plot(4, 4, WHITE)
    sp.outline()
    return sp.image()


def schedule():
    sp = Sprite(16, 16)
    dial = sp.circle(7.5, 7.5, 7)
    sp.part(dial, RAMPS["night"], "rim", base=1)                                    # night below
    sp.part(dial & sp.rect(0, 0, 15, 7), RAMPS["blue"], "rim", base=2, seam=False)  # day above
    sp.fill(sp.rect(0, 8, 15, 8) & dial, "#011c34")                                 # the horizon
    sp.fill(sp.rect(4, 3, 6, 5), "#faaf5c")                                         # sun
    sp.plot(4, 3, WHITE)
    sp.fill(sp.rect(9, 10, 11, 12) & ~sp.rect(10, 10, 11, 11), "#bfcad7")          # moon
    sp.fill(sp.line([(8, 8), (11, 4)], 1), WHITE)                                   # the hand
    sp.outline()
    return sp.image()


# ---------- figure textures (16 x 16, wrapping) -------------------------------------------
# The townsfolk figures' pixel textures: hexagonal prisms (openspec/changes/townsfolk) wear
# these, tinted per person by the material colour, so the tinted ones are drawn in the
# palette's lightest greys and the rest in their own ramps. Each one wraps.

def tile16(fn):
    import numpy as np
    a = np.zeros((16, 16, 4), np.uint8)
    for y in range(16):
        for x in range(16):
            h = fn(x, y)
            a[y, x, :3] = [int(h[i:i + 2], 16) for i in (1, 3, 5)]
            a[y, x, 3] = 255
    return Image.fromarray(a, "RGBA")


def fig_cloth():
    """A woven wool: a twill of the lightest greys, a darker seam row at the bottom of each 8."""
    return tile16(lambda x, y: "#92a3b2" if y % 8 == 7 else ("#bfcad7" if (x + y) % 4 == 0 else ("#fffefe" if (x * 3 + y) % 7 else "#bfcad7")))


def fig_skin():
    return tile16(lambda x, y: "#bfcad7" if (x * 5 + y * 3) % 23 == 0 else "#fffefe")


def fig_face():
    """The front of the head: two square eyes with a glint, brows and a mouth, on skin."""
    def px(x, y):
        if y in (5, 6, 7) and x in (3, 4, 11, 12):
            return "#fffefe" if (y == 5 and x in (3, 11)) else "#010e1a"
        if y == 3 and 2 <= x <= 5 or y == 3 and 10 <= x <= 13:
            return "#5d778d"
        if y == 11 and 6 <= x <= 9:
            return "#5d778d"
        if y == 10 and x in (5, 10):
            return "#92a3b2"
        if x in (0, 15):
            return "#bfcad7"
        return "#fffefe"
    return tile16(px)


def fig_hair():
    return tile16(lambda x, y: "#92a3b2" if (x * 2 + y) % 5 == 0 else ("#bfcad7" if (x + 3 * y) % 4 == 0 else "#fffefe"))


def fig_leather():
    return tile16(lambda x, y: "#92a3b2" if y in (0, 15) else ("#bfcad7" if (x + y * 7) % 9 == 0 else "#fffefe") if not (x % 8 == 3 and y % 4 == 2) else "#5d778d")


def fig_metal():
    ramp = RAMPS["metal"]
    return tile16(lambda x, y: ramp[4] if (x + y) % 11 == 0 else (ramp[1] if y == 15 or x == 15 else (ramp[3] if y == 0 or x == 0 else ramp[2])))


def fig_straw():
    ramp = CLOTH["straw"]
    return tile16(lambda x, y: ramp[0] if (x + y * 2) % 6 == 0 else (ramp[3] if x % 3 == 0 else ramp[2]))


def fig_wood():
    ramp = RAMPS["wood"]
    return tile16(lambda x, y: ramp[0] if x % 8 == 0 else (ramp[1] if (y * 3 + x) % 7 == 0 else ramp[2]))


FIGTEX = {"fig_cloth": fig_cloth, "fig_skin": fig_skin, "fig_face": fig_face, "fig_hair": fig_hair,
          "fig_leather": fig_leather, "fig_metal": fig_metal, "fig_straw": fig_straw, "fig_wood": fig_wood}


ICONS = {
    "farmer": farmer, "smith": smith, "trader": trader, "guard": guard,
    "fisher": fisher, "miner": miner, "clerk": clerk, "smuggler": smuggler,
    "ui_talk": talk, "ui_quest": quest, "ui_trade": trade, "ui_heart": heart, "ui_schedule": schedule,
    **FIGTEX,
}
REFS = ["foreman_okafor", "captain_zefir", "general_sabra", "armour_plating", "afterburner_kit", "emp_burst"]


def main():
    os.makedirs(OUT, exist_ok=True)
    items = []
    for name, fn in ICONS.items():
        img = fn()
        p = os.path.join(OUT, name + ".png")
        save(img, p, source="pixelkit (tools/art/npcs_art.py)")
        items.append((name, img))
        print("wrote", p)
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1]
        refs = [("ref " + r, Image.open(os.path.join(SKILL, "references", r + ".png")).convert("RGBA")) for r in REFS]
        sheet(items + refs, scale=3, cols=7, cell=(96 * 3, 64 * 3)).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main()
