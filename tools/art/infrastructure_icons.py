"""Stand-in item icons for M33's INFRASTRUCTURE tab (docs/mockups/mission-tree.html#infrastructure).

The game has no icon yet for infrastructure-missions' new goods, stations and
works (design 3): a mason's bench, dressed stone, mortar, the civil works'
blueprints (a well, a retaining wall, a culvert, a stone bridge, a ferry, a
mill race), a plot, a blueprint sheet, a market stall, a signpost for a road
between towns, and a town that grows a tier. These are 16 x 16 UI icons in the
item set's manner, drawn with early_power_icons.py's kit (a ramp lit from the
top left and the 1 px dark outline UI icons keep; CLAUDE.md keeps outlines to
UI icons), for the mockup only and labelled there as stand-ins; the game's own
come from tools/gen_item_icons.py when the things are built.

    python3 tools/art/infrastructure_icons.py
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from early_power_icons import C, draw, ell, line, poly, px, rect  # noqa: E402

C.update({
    "B": (92, 140, 214), "b": (58, 96, 170), "n": (36, 62, 120),  # blueprint
    "R": (204, 92, 70), "r": (150, 58, 44),  # awning red
    "D": (150, 120, 84), "d": (110, 86, 58),  # dirt
})


def mason_bench(d):
    rect(d, (1, 8, 14, 10), "W")
    line(d, (1, 8), (14, 8), "T")
    rect(d, (2, 11, 3, 15), "w")
    rect(d, (12, 11, 13, 15), "w")
    rect(d, (4, 4, 10, 7), "S")
    line(d, (4, 7), (10, 7), "s")
    line(d, (11, 2), (14, 5), "I", 2)
    rect(d, (9, 1, 11, 3), "w")


def dressed_stone(d):
    poly(d, [(2, 6), (8, 3), (14, 6), (8, 9)], "S")
    poly(d, [(2, 6), (8, 9), (8, 15), (2, 12)], "s")
    poly(d, [(8, 9), (14, 6), (14, 12), (8, 15)], "z")
    line(d, (4, 6), (8, 4), "F")


def mortar(d):
    poly(d, [(1, 8), (15, 8), (13, 14), (3, 14)], "w")
    line(d, (2, 9), (14, 9), "W")
    ell(d, (2, 4, 14, 10), "f")
    px(d, [(5, 6), (9, 5), (11, 7)], "s")
    line(d, (10, 1), (13, 6), "v", 2)


def well(d):
    rect(d, (2, 9, 13, 14), "s")
    for x in (2, 6, 10):
        line(d, (x, 11), (x + 3, 11), "z")
    line(d, (2, 9), (13, 9), "S")
    rect(d, (3, 3, 4, 9), "w")
    rect(d, (11, 3, 12, 9), "w")
    poly(d, [(1, 4), (8, 0), (14, 4)], "t")
    line(d, (8, 4), (8, 8), "v")
    rect(d, (7, 7, 9, 9), "i")
    px(d, [(5, 13), (10, 12)], "A")


def retaining_wall(d):
    poly(d, [(0, 4), (8, 4), (8, 15), (0, 15)], "D")
    px(d, [(2, 7), (5, 10), (3, 13)], "d")
    for y, off in ((5, 0), (8, 2), (11, 0)):
        rect(d, (8, y, 15, y + 2), "S")
        line(d, (8, y + 2), (15, y + 2), "s")
        line(d, (11 + off, y), (11 + off, y + 2), "z")
    rect(d, (8, 14, 15, 15), "s")
    line(d, (0, 4), (8, 4), "g")


def culvert(d):
    rect(d, (0, 3, 15, 6), "D")
    line(d, (0, 3), (15, 3), "f")
    rect(d, (0, 6, 15, 13), "s")
    ell(d, (4, 7, 11, 16), "z")
    ell(d, (5, 8, 10, 16), "a")
    rect(d, (0, 13, 15, 15), "A")
    px(d, [(2, 14), (12, 14), (7, 12)], "c")
    line(d, (0, 6), (15, 6), "S")


def stone_bridge(d):
    rect(d, (0, 12, 15, 15), "A")
    px(d, [(2, 13), (11, 14)], "c")
    rect(d, (0, 4, 15, 7), "S")
    line(d, (0, 4), (15, 4), "F")
    rect(d, (0, 7, 15, 12), "s")
    ell(d, (3, 8, 12, 18), "a")
    ell(d, (3, 9, 12, 19), "A")
    for x in (2, 7, 12):
        line(d, (x, 5), (x, 6), "z")


def ferry(d):
    line(d, (0, 3), (15, 3), "f")
    rect(d, (0, 2, 1, 15), "w")
    rect(d, (14, 2, 15, 15), "w")
    rect(d, (2, 11, 15, 15), "A")
    rect(d, (4, 8, 11, 10), "W")
    line(d, (4, 10), (11, 10), "v")
    line(d, (7, 3), (7, 8), "t")
    px(d, [(3, 13), (12, 14)], "c")


def mill_race(d):
    poly(d, [(0, 2), (6, 2), (12, 12), (15, 12), (15, 15), (9, 15), (2, 6), (0, 6)], "s")
    poly(d, [(0, 3), (5, 3), (11, 13), (15, 13), (15, 14), (10, 14), (3, 5), (0, 5)], "A")
    px(d, [(2, 4), (7, 8), (12, 13)], "c")
    ell(d, (9, 1, 15, 7), "w")
    ell(d, (11, 3, 13, 5), "W")


def plot(d):
    poly(d, [(0, 9), (8, 5), (15, 9), (8, 14)], "G")
    poly(d, [(3, 9), (8, 7), (12, 9), (8, 12)], "D")
    for a, b in (((3, 9), (8, 7)), ((8, 7), (12, 9)), ((12, 9), (8, 12)), ((8, 12), (3, 9))):
        line(d, a, b, "f")
    for x, y in ((3, 9), (8, 7), (12, 9), (8, 12)):
        rect(d, (x, y - 4, x, y), "W")
        px(d, [(x, y - 4)], "R")


def blueprint(d):
    rect(d, (1, 2, 14, 13), "b")
    rect(d, (1, 2, 14, 3), "B")
    for a, b in (((3, 6), (12, 6)), ((3, 6), (3, 11)), ((12, 6), (12, 11)), ((3, 11), (12, 11)), ((7, 6), (7, 11))):
        line(d, a, b, "c")
    poly(d, [(3, 6), (7, 4), (12, 6)], "B")
    rect(d, (0, 12, 15, 14), "f")
    line(d, (0, 14), (15, 14), "t")


def market(d):
    rect(d, (2, 8, 13, 14), "W")
    line(d, (2, 8), (13, 8), "T")
    rect(d, (3, 9, 12, 10), "L")
    px(d, [(4, 9), (7, 9), (10, 9)], "G")
    rect(d, (2, 10, 3, 15), "w")
    rect(d, (12, 10, 13, 15), "w")
    for i, x in enumerate(range(1, 15, 3)):
        poly(d, [(x, 3), (x + 3, 3), (x + 3, 7), (x, 7)], "R" if i % 2 == 0 else "F")
    line(d, (1, 7), (15, 7), "r")
    rect(d, (7, 1, 8, 3), "v")


def signpost(d):
    rect(d, (7, 4, 8, 15), "w")
    poly(d, [(1, 3), (11, 3), (14, 5), (11, 7), (1, 7)], "W")
    line(d, (2, 5), (10, 5), "v")
    poly(d, [(14, 8), (4, 8), (1, 10), (4, 12), (14, 12)], "W")
    line(d, (5, 10), (13, 10), "v")
    rect(d, (5, 14, 10, 15), "s")


def town_tier(d):
    poly(d, [(0, 9), (4, 6), (8, 9)], "r")
    rect(d, (1, 9, 7, 15), "S")
    rect(d, (3, 11, 4, 15), "w")
    poly(d, [(7, 7), (11, 3), (15, 7)], "r")
    rect(d, (8, 7, 14, 15), "W")
    rect(d, (10, 9, 11, 10), "O")
    rect(d, (10, 12, 12, 15), "v")
    poly(d, [(2, 3), (4, 0), (6, 3)], "O")
    rect(d, (3, 3, 5, 5), "O")


ICONS = {
    "mason-bench": mason_bench,
    "dressed-stone": dressed_stone,
    "mortar": mortar,
    "well": well,
    "retaining-wall": retaining_wall,
    "culvert": culvert,
    "stone-bridge": stone_bridge,
    "ferry": ferry,
    "mill-race": mill_race,
    "plot": plot,
    "blueprint": blueprint,
    "market": market,
    "signpost": signpost,
    "town-tier": town_tier,
}

if __name__ == "__main__":
    out = Path(__file__).resolve().parents[2] / "docs/mockups/mission-tree/items"
    for name, fn in ICONS.items():
        draw(name, fn)
    print(f"{len(ICONS)} icons in {out}")
