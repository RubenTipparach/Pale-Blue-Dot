"""Stand-in item icons for M33's Survival tab (docs/mockups/mission-tree.html#survival).

The game has no icon yet for the survival goods and weapons of
survival-mode design 15 to 18 (a spear, a bow, arrows, a hunting axe, a
sword, a straw, raw meat, hide, bone, a feather, a straw hat, a wool cloak,
a bedroll, a tanning rack, meat stew) or for its missions' marks (a drop of
water, a cactus, a hare, a fox, the moon, a breath, a splash, an out torch,
hot and cold, stamina). These are 16 x 16 UI icons in the item set's manner (a ramp
lit from the top left and the 1 px dark outline UI icons keep; CLAUDE.md
keeps outlines to UI icons), drawn for the mockup only and labelled there
as stand-ins; the game's own come from tools/gen_item_icons.py when the
things are built.

    python3 tools/mockup_survival_icons.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[1] / "docs/mockups/mission-tree/items"
OUTLINE = (42, 26, 16, 255)

C = {
    "W": (200, 149, 92), "w": (154, 106, 60), "v": (107, 70, 38),  # wood
    "S": (214, 208, 196), "s": (155, 149, 138), "z": (102, 97, 90),  # stone
    "I": (184, 192, 200), "i": (122, 132, 144), "j": (74, 82, 92),  # iron
    "Z": (230, 170, 90), "y": (196, 128, 56), "x": (140, 84, 34),  # bronze
    "R": (214, 74, 64), "r": (150, 40, 40), "q": (240, 140, 120),  # meat
    "H": (176, 120, 72), "h": (127, 80, 48), "k": (82, 52, 30),  # hide
    "B": (240, 232, 214), "b": (200, 188, 160),  # bone
    "A": (110, 180, 240), "a": (60, 120, 200), "c": (200, 230, 255),  # water
    "G": (120, 170, 80), "g": (70, 120, 52), "f": (170, 210, 110),  # cactus
    "T": (232, 206, 120), "t": (190, 160, 80),  # straw
    "O": (255, 196, 70), "o": (240, 120, 40), "e": (255, 240, 160),  # flame, sun
    "M": (220, 224, 240), "m": (150, 160, 190),  # moon
    "U": (220, 214, 200), "u": (170, 164, 150),  # wool
    "F": (196, 92, 40), "d": (140, 60, 30),  # fox
    "N": (70, 60, 56),  # char
}


def px(d, pts, col):
    for x, y in pts:
        d.point((x, y), fill=C[col] + (255,))


def line(d, a, b, col, w=1):
    d.line([a, b], fill=C[col] + (255,), width=w)


def poly(d, pts, col):
    d.polygon(pts, fill=C[col] + (255,))


def ell(d, box, col):
    d.ellipse(box, fill=C[col] + (255,))


def outlined(im):
    a = im.split()[3]
    out = Image.new("RGBA", im.size, (0, 0, 0, 0))
    w, h = im.size
    pa = a.load()
    po = out.load()
    for y in range(h):
        for x in range(w):
            if pa[x, y]:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                X, Y = x + dx, y + dy
                if 0 <= X < w and 0 <= Y < h and pa[X, Y]:
                    po[x, y] = OUTLINE
                    break
    out.alpha_composite(im)
    return out


def draw(name, fn):
    im = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    fn(ImageDraw.Draw(im))
    outlined(im).save(OUT / f"{name}.png")


def spear(d, head="S", head2="s"):
    line(d, (3, 13), (10, 6), "w")
    line(d, (2, 13), (9, 6), "W")
    poly(d, [(10, 6), (13, 2), (14, 1), (12, 5), (11, 7)], head)
    px(d, [(11, 5), (12, 4)], head2)
    px(d, [(9, 7), (10, 7), (9, 8)], "T")


def bow(d):
    d.arc((1, 1, 17, 17), 190, 260, fill=C["W"] + (255,), width=2)
    d.arc((1, 1, 17, 17), 260, 280, fill=C["w"] + (255,), width=2)
    line(d, (2, 9), (9, 2), "b")
    line(d, (4, 12), (12, 4), "W")
    poly(d, [(12, 4), (14, 2), (13, 5)], "s")
    px(d, [(4, 13), (3, 12), (5, 13)], "R")


def arrows(d):
    for o in (0, 3):
        line(d, (2 + o, 13), (11 + o, 4), "W")
        poly(d, [(11 + o, 4), (13 + o, 2), (12 + o, 5)], "s")
        px(d, [(2 + o, 12), (3 + o, 14), (1 + o, 13)], "U")


def hunting_axe(d, head="Z", dark="x"):
    line(d, (4, 14), (9, 4), "w", 2)
    poly(d, [(8, 3), (13, 2), (14, 7), (11, 6), (9, 6)], head)
    px(d, [(13, 3), (13, 4), (13, 5)], "e")
    px(d, [(10, 6), (11, 5)], dark)
    px(d, [(5, 12), (6, 11)], "k")


def sword(d, blade="Z", dark="x"):
    poly(d, [(5, 10), (12, 3), (14, 1), (13, 4), (6, 11)], blade)
    line(d, (6, 10), (13, 3), "e")
    line(d, (3, 9), (7, 13), dark, 2)
    line(d, (2, 14), (5, 11), "k", 2)
    px(d, [(1, 15)], "y")


def straw(d):
    line(d, (3, 13), (12, 2), "T", 2)
    line(d, (4, 13), (13, 3), "t")
    px(d, [(6, 10), (9, 6)], "t")


def cactus(d):
    poly(d, [(6, 2), (9, 2), (10, 14), (5, 14)], "G")
    poly(d, [(2, 6), (4, 6), (5, 10), (2, 10)], "G")
    poly(d, [(11, 4), (13, 4), (13, 9), (10, 9)], "G")
    line(d, (8, 3), (8, 13), "f")
    line(d, (6, 3), (6, 13), "g")
    px(d, [(7, 1), (12, 3)], "R")


def raw_meat(d):
    ell(d, (2, 4, 14, 13), "R")
    ell(d, (4, 6, 12, 11), "r")
    px(d, [(5, 6), (6, 5), (7, 5)], "q")
    ell(d, (9, 8, 12, 11), "B")


def roast(d):
    ell(d, (2, 4, 13, 12), "h")
    ell(d, (4, 5, 11, 10), "H")
    px(d, [(5, 5), (6, 5)], "T")
    line(d, (11, 10), (14, 13), "B", 2)


def stew(d):
    poly(d, [(2, 7), (14, 7), (12, 13), (4, 13)], "v")
    poly(d, [(3, 7), (13, 7), (12, 9), (4, 9)], "h")
    px(d, [(5, 7), (8, 8), (10, 7)], "R")
    px(d, [(6, 4), (7, 3), (9, 5), (10, 4)], "u")


def hide(d):
    poly(d, [(3, 3), (6, 4), (10, 4), (13, 3), (12, 7), (13, 13), (9, 12), (7, 12), (3, 13), (4, 7)], "H")
    poly(d, [(6, 6), (10, 6), (10, 10), (6, 10)], "h")
    px(d, [(5, 5), (7, 5)], "T")


def bone(d):
    line(d, (4, 12), (11, 5), "B", 2)
    ell(d, (1, 11, 5, 15), "B")
    ell(d, (10, 1, 14, 5), "B")
    px(d, [(5, 12), (11, 6)], "b")


def feather(d):
    poly(d, [(3, 14), (7, 7), (12, 2), (11, 6), (8, 11)], "U")
    line(d, (3, 14), (12, 2), "u")
    px(d, [(10, 4), (9, 6)], "M")


def drop(d):
    poly(d, [(8, 1), (12, 8), (4, 8)], "A")
    ell(d, (3, 6, 13, 15), "A")
    ell(d, (5, 8, 12, 14), "a")
    px(d, [(6, 8), (6, 9), (7, 7)], "c")


def fruit(d):
    ell(d, (3, 4, 13, 14), "R")
    ell(d, (5, 6, 12, 13), "r")
    px(d, [(5, 6), (6, 5)], "q")
    line(d, (8, 4), (9, 1), "v")
    px(d, [(10, 2), (11, 2)], "G")


def hare(d):
    ell(d, (3, 8, 13, 14), "H")
    ell(d, (9, 5, 14, 10), "H")
    line(d, (10, 5), (10, 1), "H", 2)
    line(d, (12, 5), (13, 1), "h", 2)
    px(d, [(12, 7)], "k")
    ell(d, (2, 9, 5, 12), "U")


def fox(d):
    poly(d, [(3, 4), (6, 7), (10, 7), (13, 4), (13, 10), (8, 14), (3, 10)], "F")
    poly(d, [(5, 10), (8, 13), (11, 10), (8, 11)], "U")
    px(d, [(6, 9), (10, 9)], "k")
    px(d, [(4, 5), (12, 5)], "d")


def moon(d):
    ell(d, (3, 2, 14, 13), "M")
    ell(d, (6, 1, 16, 11), "m")
    ell(d, (7, 0, 17, 10), (0, 0, 0, 0)) if False else None
    d.ellipse((7, 0, 17, 10), fill=(0, 0, 0, 0))
    px(d, [(1, 12), (13, 14)], "e")


def sun(d):
    ell(d, (4, 4, 12, 12), "O")
    ell(d, (5, 5, 10, 10), "e")
    for a, b in (((8, 0), (8, 2)), ((8, 14), (8, 15)), ((0, 8), (2, 8)), ((14, 8), (15, 8)),
                 ((2, 2), (3, 3)), ((13, 13), (14, 14)), ((2, 14), (3, 13)), ((13, 3), (14, 2))):
        line(d, a, b, "o")


def snowflake(d):
    for a, b in (((8, 1), (8, 15)), ((2, 4), (14, 12)), ((2, 12), (14, 4))):
        line(d, a, b, "c")
    px(d, [(8, 8)], "A")
    px(d, [(7, 2), (9, 2), (7, 14), (9, 14)], "A")


def breath(d):
    ell(d, (2, 7, 9, 14), "c")
    ell(d, (8, 3, 13, 8), "c")
    ell(d, (11, 10, 14, 13), "c")
    px(d, [(4, 9), (9, 4)], "A")


def splash(d):
    poly(d, [(1, 13), (15, 13), (13, 15), (3, 15)], "a")
    poly(d, [(4, 13), (6, 6), (8, 12), (10, 4), (12, 13)], "A")
    px(d, [(6, 7), (10, 5), (3, 9), (13, 8)], "c")


def torch_out(d):
    line(d, (6, 15), (9, 6), "w", 2)
    ell(d, (7, 2, 12, 7), "N")
    px(d, [(9, 1), (10, 0), (11, 1)], "s")


def hat(d):
    ell(d, (1, 9, 15, 14), "T")
    ell(d, (4, 4, 12, 11), "T")
    line(d, (4, 10), (12, 10), "R")
    px(d, [(5, 5), (6, 5)], "e")
    line(d, (2, 12), (14, 12), "t")


def cloak(d):
    poly(d, [(6, 2), (10, 2), (14, 14), (2, 14)], "U")
    poly(d, [(8, 4), (11, 14), (8, 14)], "u")
    px(d, [(7, 3), (9, 3)], "y")


def bedroll(d):
    ell(d, (1, 6, 9, 14), "H")
    poly(d, [(5, 6), (14, 6), (14, 14), (5, 14)], "H")
    ell(d, (3, 8, 7, 12), "h")
    line(d, (9, 6), (9, 14), "k")
    line(d, (12, 6), (12, 14), "k")


def rack(d):
    line(d, (2, 15), (5, 2), "w", 2)
    line(d, (14, 15), (11, 2), "w", 2)
    line(d, (4, 3), (12, 3), "W", 2)
    poly(d, [(5, 5), (11, 5), (12, 12), (4, 12)], "H")
    px(d, [(6, 7), (9, 9)], "h")


def stamina(d):
    poly(d, [(9, 1), (3, 9), (7, 9), (5, 15), (13, 6), (9, 6), (11, 1)], "O")
    poly(d, [(9, 2), (5, 8), (8, 8)], "e")


ICONS = {
    "stamina": stamina,
    "stone-spear": spear,
    "bronze-spear": lambda d: spear(d, "Z", "x"),
    "bow": bow,
    "arrows": arrows,
    "hunting-axe": hunting_axe,
    "sword": sword,
    "iron-sword": lambda d: sword(d, "I", "j"),
    "straw": straw,
    "cactus": cactus,
    "raw-meat": raw_meat,
    "roast-meat": roast,
    "meat-stew": stew,
    "hide": hide,
    "bone": bone,
    "feather": feather,
    "water-drop": drop,
    "fruit": fruit,
    "hare": hare,
    "fox": fox,
    "moon": moon,
    "sun": sun,
    "snowflake": snowflake,
    "breath": breath,
    "splash": splash,
    "torch-out": torch_out,
    "straw-hat": hat,
    "wool-cloak": cloak,
    "bedroll": bedroll,
    "tanning-rack": rack,
}

if __name__ == "__main__":
    for name, fn in ICONS.items():
        draw(name, fn)
    print(f"{len(ICONS)} icons in {OUT}")
