"""Stand-in item icons for M33's POWER tab (docs/mockups/mission-tree.html#power).

The game has no icon yet for early-power's goods and machines (design 3):
a wooden gear, an axle, a millstone, the windmill and the water mill, flour,
a loaf, feed mash, copper wire, a dynamo, a power pole, an electric lamp, a
motor, bellows, or for its missions' marks (a sluice, a bolt for power). These
are 16 x 16 UI icons in the item set's manner (a ramp lit from the top left
and the 1 px dark outline UI icons keep; CLAUDE.md keeps outlines to UI
icons), drawn for the mockup only and labelled there as stand-ins; the game's
own come from tools/gen_item_icons.py when the things are built.

    python3 tools/art/early_power_icons.py
"""

from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[2] / "docs/mockups/mission-tree/items"
OUTLINE = (42, 26, 16, 255)

C = {
    "W": (200, 149, 92), "w": (154, 106, 60), "v": (107, 70, 38),  # wood
    "S": (214, 208, 196), "s": (155, 149, 138), "z": (102, 97, 90),  # stone
    "I": (184, 192, 200), "i": (122, 132, 144), "j": (74, 82, 92),  # iron
    "K": (232, 150, 92), "k": (190, 100, 52), "q": (130, 64, 34),  # copper
    "F": (246, 238, 220), "f": (214, 200, 172),  # flour, linen
    "L": (222, 164, 84), "l": (170, 110, 48), "m": (120, 72, 30),  # crust
    "G": (176, 196, 96), "g": (120, 146, 60),  # mash
    "A": (110, 180, 240), "a": (60, 120, 200), "c": (200, 230, 255),  # water
    "O": (255, 214, 96), "o": (240, 150, 40), "e": (255, 246, 190),  # light
    "T": (232, 206, 120), "t": (190, 160, 80),  # thatch
    "H": (150, 98, 60), "h": (104, 66, 40),  # leather
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


def rect(d, box, col):
    d.rectangle(box, fill=C[col] + (255,))


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


def gear(d, cx=8, cy=8, r=5, a="W", b="w", hole="v"):
    for x, y in ((0, -1), (1, 0), (0, 1), (-1, 0), (1, -1), (1, 1), (-1, 1), (-1, -1)):
        k = r + 1 if x == 0 or y == 0 else r
        X, Y = cx + x * k * (0.72 if x and y else 1), cy + y * k * (0.72 if x and y else 1)
        rect(d, (round(X) - 1, round(Y) - 1, round(X) + 1, round(Y) + 1), b)
    ell(d, (cx - r, cy - r, cx + r, cy + r), a)
    d.arc((cx - r, cy - r, cx + r, cy + r), 20, 200, fill=C[b] + (255,))
    ell(d, (cx - 1, cy - 1, cx + 1, cy + 1), hole)


def wooden_gear(d):
    gear(d)
    px(d, [(6, 5), (5, 6)], "T")


def axle(d):
    line(d, (2, 13), (13, 2), "I", 2)
    line(d, (3, 13), (13, 3), "i")
    rect(d, (3, 10, 5, 12), "W")
    rect(d, (10, 3, 12, 5), "W")
    px(d, [(3, 10), (10, 3)], "T")


def millstone(d):
    ell(d, (1, 4, 14, 13), "s")
    ell(d, (1, 2, 14, 11), "S")
    ell(d, (6, 5, 9, 8), "z")
    line(d, (3, 6), (6, 6), "s")
    line(d, (9, 9), (12, 8), "s")


def windmill(d):
    poly(d, [(6, 15), (10, 15), (9, 7), (7, 7)], "W")
    line(d, (9, 8), (10, 15), "w")
    poly(d, [(6, 7), (10, 7), (8, 5)], "t")
    for a, b in (((8, 6), (2, 0)), ((8, 6), (14, 1)), ((8, 6), (3, 12)), ((8, 6), (14, 11))):
        line(d, a, b, "v")
    poly(d, [(8, 6), (3, 1), (2, 2)], "f")
    poly(d, [(8, 6), (13, 0), (14, 2)], "f")
    poly(d, [(8, 6), (2, 11), (4, 12)], "f")
    poly(d, [(8, 6), (14, 10), (13, 12)], "f")
    px(d, [(8, 6)], "j")
    rect(d, (7, 12, 8, 14), "v")


def water_mill(d):
    rect(d, (7, 5, 14, 12), "W")
    poly(d, [(6, 5), (15, 5), (12, 2), (9, 2)], "t")
    rect(d, (9, 7, 10, 9), "O")
    rect(d, (12, 9, 13, 12), "v")
    ell(d, (0, 6, 8, 14), "w")
    ell(d, (2, 8, 6, 12), "W")
    for a, b in (((4, 6), (4, 14)), ((0, 10), (8, 10))):
        line(d, a, b, "v")
    rect(d, (0, 13, 15, 15), "A")
    px(d, [(2, 13), (6, 14), (11, 13)], "c")


def flour(d):
    poly(d, [(3, 5), (12, 5), (14, 14), (1, 14)], "f")
    poly(d, [(4, 6), (11, 6), (12, 13), (3, 13)], "F")
    rect(d, (5, 2, 10, 5), "f")
    line(d, (4, 4), (11, 4), "t")
    px(d, [(6, 9), (9, 10), (7, 11)], "f")


def loaf(d):
    ell(d, (1, 5, 15, 14), "m")
    ell(d, (1, 4, 15, 12), "L")
    for x in (4, 7, 10):
        line(d, (x, 6), (x + 2, 9), "l")
    px(d, [(4, 5), (5, 5)], "e")


def feed_mash(d):
    poly(d, [(1, 8), (15, 8), (13, 14), (3, 14)], "w")
    line(d, (2, 9), (14, 9), "W")
    ell(d, (2, 4, 14, 10), "G")
    px(d, [(5, 6), (9, 5), (11, 7), (7, 7)], "g")
    px(d, [(6, 5), (10, 6)], "T")


def copper_wire(d):
    ell(d, (2, 2, 14, 14), "q")
    ell(d, (3, 3, 13, 13), "K")
    ell(d, (6, 6, 10, 10), "v")
    for r in (4, 6):
        d.arc((8 - r, 8 - r, 8 + r, 8 + r), 200, 320, fill=C["e"] + (255,))
    line(d, (13, 9), (15, 13), "k")


def dynamo(d):
    rect(d, (2, 4, 11, 12), "i")
    rect(d, (2, 4, 11, 6), "I")
    for x in (4, 6, 8, 10):
        line(d, (x, 7), (x, 11), "K")
    line(d, (11, 8), (15, 8), "I", 2)
    rect(d, (1, 12, 12, 14), "j")
    px(d, [(3, 5), (4, 5)], "c")


def power_pole(d):
    line(d, (7, 15), (7, 2), "w", 2)
    line(d, (2, 4), (13, 4), "W", 2)
    px(d, [(3, 3), (12, 3), (8, 3)], "I")
    line(d, (0, 6), (3, 4), "k")
    line(d, (12, 4), (15, 6), "k")
    px(d, [(8, 6), (8, 10)], "v")


def electric_lamp(d):
    line(d, (7, 1), (7, 3), "k")
    poly(d, [(4, 4), (11, 4), (13, 7), (2, 7)], "i")
    line(d, (3, 5), (11, 5), "I")
    ell(d, (4, 6, 11, 13), "O")
    ell(d, (5, 7, 9, 11), "e")
    px(d, [(7, 13)], "o")


def motor(d):
    rect(d, (2, 4, 12, 12), "i")
    for y in (5, 7, 9, 11):
        line(d, (3, y), (11, y), "I")
    rect(d, (12, 7, 15, 9), "I")
    rect(d, (1, 12, 13, 14), "j")
    rect(d, (4, 2, 7, 3), "K")


def bellows(d):
    poly(d, [(1, 4), (10, 7), (10, 10), (1, 13)], "W")
    poly(d, [(2, 6), (9, 8), (9, 9), (2, 11)], "H")
    line(d, (3, 7), (8, 8), "h")
    line(d, (3, 10), (8, 9), "h")
    rect(d, (10, 7, 15, 9), "j")
    px(d, [(15, 8)], "o")


def sluice(d):
    rect(d, (1, 11, 15, 15), "A")
    px(d, [(3, 12), (12, 13)], "c")
    rect(d, (2, 2, 3, 13), "w")
    rect(d, (12, 2, 13, 13), "w")
    rect(d, (4, 5, 11, 12), "W")
    for y in (7, 9):
        line(d, (4, y), (11, y), "w")
    line(d, (7, 1), (7, 5), "I")
    rect(d, (5, 0, 9, 1), "I")


def bolt(d):
    poly(d, [(9, 0), (3, 9), (7, 9), (5, 15), (13, 5), (9, 5), (11, 0)], "O")
    poly(d, [(9, 1), (5, 8), (7, 8)], "e")


ICONS = {
    "wooden-gear": wooden_gear,
    "axle": axle,
    "millstone": millstone,
    "windmill": windmill,
    "water-mill": water_mill,
    "flour": flour,
    "loaf": loaf,
    "feed-mash": feed_mash,
    "copper-wire": copper_wire,
    "dynamo": dynamo,
    "power-pole": power_pole,
    "electric-lamp": electric_lamp,
    "motor": motor,
    "bellows": bellows,
    "sluice": sluice,
    "bolt": bolt,
}

if __name__ == "__main__":
    for name, fn in ICONS.items():
        draw(name, fn)
    print(f"{len(ICONS)} icons in {OUT}")
