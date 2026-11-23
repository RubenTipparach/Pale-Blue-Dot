#!/usr/bin/env python3
"""Draw the item icons this repository makes itself: the three fish new to
Pale Blue Dot, the six lights and the bare hand, as 16x16 RGBA PNGs under
assets/items/. The four tool icons are drawn from the tools' own hexes by
tools/gen_held_tools.py, which uses this file's PNG writer.

The lights (`lamps-and-lanterns` task 5.3) are drawn as the game draws them:
iron frames round lit glass in the towns mockup's lamp colour, and flames
white-gold at the root and orange at the tip. One palette serves all six, so
a lantern's glass is the same colour in every slot.

The pixel grids are the fishing mockup's (docs/mockups/fishing.html), which is
what was shown and approved; a grid letter names a palette colour and "." is
transparent. The five fish reused from Tenebris are NOT drawn here: they are
copied byte for byte with their provenance (assets/items/fish/PROVENANCE.md).

    python3 tools/gen_item_icons.py            # write the icons
    python3 tools/gen_item_icons.py --check    # fail if any differs

Deterministic, stdlib only (zlib), so --check holds on any machine.
"""

import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "assets" / "items"


ICONS = {
    "fish/silverfin": ([
        "................", "................", "................", "................",
        "..........s.....", "....ssssssss..t.", "..sssssssssssst.", ".seSsssssssssstt",
        "..sssssssssssst.", "....ssssssss..t.", "..........s.....", "................",
        "................", "................", "................", "................",
    ], {"s": "#cfe3ea", "S": "#9fb9c2", "e": "#1b1b1b", "t": "#9fb9c2"}),
    "fish/perch": ([
        "................", "................", "......d.d.......", ".....ddddd......",
        "...pbpbpbpbp..t.", "..pbpbpbpbpbp.t.", ".epbpbpbpbpbpttt", "..pbpbpbpbpbp.t.",
        "...pbpbpbpbp..t.", ".....ppppp......", "................", "................",
        "................", "................", "................", "................",
    ], {"p": "#d8a13a", "b": "#6b4a18", "d": "#c8612b", "e": "#1b1b1b", "t": "#c8612b"}),
    "fish/deepback": ([
        "................", "................", "........dd......", "......dddd......",
        "..ddddddddddd...", ".dddddddddddddtt", "eddddddddddddttt", ".lllllllllllddtt",
        "..lllllllllll...", "................", "................", "................",
        "................", "................", "................", "................",
    ], {"d": "#4f6f8f", "l": "#9fb6c9", "e": "#e8e8e8", "t": "#4f6f8f"}),
}


# The lights' one palette: iron dark to light, lit glass, flame root to tip,
# wood and wax.
LIGHT = {
    "k": "#1e1e22", "i": "#3c3c44", "I": "#5d5d68",
    "g": "#ffd9a0", "G": "#fff2cc",
    "y": "#fff0b0", "F": "#ffc050", "f": "#ff8a3a",
    "w": "#5a3a1e", "W": "#8a5a30",
    "x": "#efe6d0", "X": "#cbbf9f",
}

ICONS.update({
    "lights/torch": ([
        "................", "........y.......", ".......yF.......", "......fFyf......",
        "......FyyF......", ".....fFyyFf.....", "......fFFf......", "......kkkk......",
        "......wWWw......", ".......Ww.......", ".......Ww.......", ".......Ww.......",
        ".......Ww.......", ".......Ww.......", ".......ww.......", "................",
    ], LIGHT),
    "lights/lantern_post": ([
        "......kkkk......", ".....kIiiIk.....", ".....kgGGgk.....", ".....kgGGgk.....",
        ".....kggggk.....", ".....kkkkkk.....", ".......ki.......", ".......ki.......",
        ".......ki.......", ".......ki.......", ".......ki.......", ".......ki.......",
        ".......ki.......", "......kiik......", ".....kiiiik.....", "................",
    ], LIGHT),
    "lights/lantern_wall": ([
        "................", "kk..............", "kIkkkkkkk.......", "kIiiiiiik.......",
        "kI.....k........", "kI....kkkk......", "kI...kIiiIk.....", "kk...kgGGgk.....",
        ".....kgGGgk.....", ".....kgGGgk.....", ".....kggggk.....", ".....kkkkkk.....",
        "......kiik......", "................", "................", "................",
    ], LIGHT),
    "lights/lantern_hanging": ([
        ".......k........", ".......I........", ".......k........", ".......I........",
        ".......k........", "......kkkk......", ".....kIiiIk.....", ".....kgGGgk.....",
        ".....kgGGgk.....", ".....kgGGgk.....", ".....kggggk.....", ".....kkkkkk.....",
        "......kiik......", "................", "................", "................",
    ], LIGHT),
    "lights/brazier": ([
        "................", "................", ".......y........", "......yFy.......",
        ".....fFyFf......", "....fFFyFFf.....", "...fFFFFFFFf....", "..kiiiiiiiiik...",
        "..kIiiiiiiiIk...", "...kiiiiiiik....", "....kkkkkkk.....", ".......ki.......",
        ".......ki.......", "......kiik......", "....kki..ikk....", "................",
    ], LIGHT),
    "lights/candle": ([
        "................", "................", "................", ".......y........",
        "......yFy.......", "......fFf.......", ".......k........", "......xxX.......",
        "......xxX.......", "......xxX.......", "......xxX.......", "......xxX.......",
        "......xxX.......", "....kIiiiIk.....", ".....kkkkk......", "................",
    ], LIGHT),
})


# Bare hands (`inventory-grid` decision 8): an open palm, fingers up, in the
# held hand's two skin tones (assets/models/held_tools.ron) with a dark rim.
ICONS["tools/hand"] = ([
    "................", "......k.k.......", ".....kskskk.....", ".....ksksksk....",
    ".....ksksksk....", ".....ksksksk....", "..k..ksksksk....", ".ksk.kssssssk...",
    ".ksskssssssk....", "..kssssssssk....", "...ksssssssk....", "....kssssssk....",
    ".....kSSSSk.....", ".....kSSSSk.....", "......kkkk......", "................",
], {"k": "#5a3a24", "s": "#e3b089", "S": "#c39776"})


# The husbandry items (`animal-husbandry` design 12, S6): the pen pieces in
# the fence's own wood (the game draws posts and rails in these two browns),
# and what kept animals give. One palette serves all five.
FARM = {
    "k": "#3a2614", "w": "#7a5230", "W": "#a8743f",
    "e": "#f2e6cc", "E": "#d9c7a0",
    "m": "#ffffff", "M": "#dfe6ea",
    "i": "#3c3c44", "I": "#8a8a96",
}

ICONS.update({
    "goods/fence": ([
        "................", "................", "..kk.......kk...", ".kWWk.....kWWk..",
        ".kWwkkkkkkkWwk..", ".kWwWWWWWWWWwk..", ".kWwwwwwwwwwwk..", ".kWwkkkkkkkWwk..",
        ".kWwk.....kWwk..", ".kWwkkkkkkkWwk..", ".kWwWWWWWWWWwk..", ".kWwwwwwwwwwwk..",
        ".kWwkkkkkkkWwk..", ".kWwk.....kWwk..", ".kkkk.....kkkk..", "................",
    ], FARM),
    "goods/gate": ([
        "................", "................", ".kk..........kk.", "kWWk........kWWk",
        "kWwkkkkkkkkkkWwk", "kWwWWWWWWWWWWWwk", "kWwkkkkkkkkWWkwk", "kWwk.....kWWkkwk",
        "kWwk...kWWk..Wwk", "kWwkkkWWkkkkkWwk", "kWwWWWWWWWWWWWwk", "kWwkkkkkkkkkkWwk",
        "kWwk........kWwk", "kWwk........kWwk", "kkkk........kkkk", "................",
    ], FARM),
    "goods/egg": ([
        "................", "................", "................", "......kkkk......",
        ".....keeeek.....", "....keeeeeek....", "....keemeeek....", "...keemeeeeek...",
        "...keeeeeeeek...", "...keeeeeeeek...", "...keeeeeeEEk...", "....keeeeEEk....",
        "....kEeeEEEk....", ".....kEEEEk.....", "......kkkk......", "................",
    ], FARM),
    "goods/wool": ([
        "................", "................", "................", "......kk.kk.....",
        "....kkmmkmmkk...", "...kmmmmmmmmmk..", "..kmmMmmmmMmmmk.", "..kmmmmmMmmmmmk.",
        ".kmmMmmmmmmmMmk.", ".kmmmmmMmmmmmmk.", "..kmmmmmmmMmmk..", "..kMmmMmmmmmMk..",
        "...kkMMkkMMkkk..", ".....kk..kk.....", "................", "................",
    ], FARM),
    "goods/milk": ([
        "................", "................", "....kkkkkkkk....", "...k........k...",
        "..k..........k..", ".kiiiiiiiiiiiik.", ".kImmmmmmmmmmIk.", ".kIMmmmmmmmmMIk.",
        "..kIiiiiiiiiIk..", "..kIiIIIIIIiIk..", "..kIiiiiiiiiIk..", "..kIiIIIIIIiIk..",
        "...kIiiiiiiIk...", "...kIIIIIIIIk...", "....kkkkkkkk....", "................",
    ], FARM),
})


# Leather (`small-boats` design 2e): a tanned hide, stretched out, in the fence's
# browns with a lighter grain, from a kept plodder.
ICONS["goods/leather"] = ([
    "................", "..kk........kk..", ".khLk......kLhk.", "..kLLkkkkkkLLk..",
    "...kLhhLLhhLk...", "..kLLhLLLLLhLk..", "..kLhLLLLlLLLk..", "..kLLLLlLLLLhk..",
    "..kLhLLLLLLLLk..", "..kLLLLLLhLLLk..", "..kLLhLLLLLlLk..", "...kLLLLLLLLk...",
    "..kLLkkkkkkLLk..", ".khLk......kLhk.", "..kk........kk..", "................",
], {"k": "#3a2614", "L": "#9a6a3c", "l": "#7a5230", "h": "#b9844f"})


# Design 13's two (`animal-husbandry` E3, E4): the fertile egg is the egg
# with the warm brown speckle of a laid-under-a-cockerel egg, and the
# incubator a small crate of the fence's wood with a glass lid.
HATCH = dict(FARM, **{"s": "#a8743f", "g": "#a9d8e8", "G": "#e8f6fb"})

ICONS.update({
    "goods/fertile-egg": ([
        "................", "................", "................", "......kkkk......",
        ".....keeeek.....", "....keseeeek....", "....keemeesk....", "...keemeeeeek...",
        "...kseeeseeek...", "...keeeeeeesk...", "...keeseeeEEk...", "....keeeeEsk....",
        "....kEseEEEk....", ".....kEEEEk.....", "......kkkk......", "................",
    ], HATCH),
    "goods/incubator": ([
        "................", "................", "................", "................",
        "..kkkkkkkkkkkk..", "..kGgggggggggk..", "..kgGgeeegegGk..", "..kggeeeeegegk..",
        "..kkkkkkkkkkkk..", "..kWwWwWwWwWwk..", "..kWwkkkkkkWwk..", "..kWwWwWwWwWwk..",
        "..kWwWwWwWwWwk..", "..kkkkkkkkkkkk..", "...kk......kk...", "................",
    ], HATCH),
})


# The wooden bucket (`water-flow` E11): staves in the fence's two browns,
# two iron hoops and a rope handle; full, the sea's own blues at its brim.
BUCKET = dict(FARM, **{"r": "#c9a46a", "b": "#1c5ea0", "B": "#5a8ac0", "d": "#1d2b53"})

ICONS.update({
    "goods/bucket": ([
        "................", ".....krrrrk.....", "....r......r....", "...r........r...",
        "..kkkkkkkkkkkk..", "..kddddddddddk..", "..kIiiiiiiiiIk..", "...kWwWwWwWwk...",
        "...kWwWwWwWwk...", "...kIiiiiiiIk...", "...kWwWwWwWwk...", "....kWwWwWwk....",
        "....kIiiiiIk....", "....kWwWwWwk....", ".....kkkkkk.....", "................",
    ], BUCKET),
    "goods/water-bucket": ([
        "................", ".....krrrrk.....", "....r......r....", "...r........r...",
        "..kkkkkkkkkkkk..", "..kbBbbBbbbBbk..", "..kIiiiiiiiiIk..", "...kWwWwWwWwk...",
        "...kWwWwWwWwk...", "...kIiiiiiiIk...", "...kWwWwWwWwk...", "....kWwWwWwk....",
        "....kIiiiiIk....", "....kWwWwWwk....", ".....kkkkkk.....", "................",
    ], BUCKET),
})


# Bait (`starter-missions` G1): a lump of wet shore earth with two worms
# curling out of it, in the bucket's dark outline and the earth's browns.
BAIT = {
    "k": "#2a1a10", "e": "#6b4a2e", "E": "#8a6440", "w": "#4a3220",
    "p": "#c8707a", "P": "#e8a0a4", "q": "#9a4a56",
}

ICONS.update({
    "goods/bait": ([
        "................", "................", "......kk........", ".....kPpk.......",
        ".....kpqk...kk..", "......kpk..kPpk.", "......kpk..kpqk.", "...kkkkpkkkkpk..",
        "..kEEeekpeEepkk.", ".kEeeewekeeekEk.", ".keeEeeweeEeeek.", ".kweeeeeeeweeek.",
        ".keeweEeeeeewek.", "..kweeeeweeeek..", "...kkkkkkkkkk...", "................",
    ], BAIT),
})


# The made boats (`small-boats` design 3): the inner tube M34's felted-wool
# ring, sealed with pitch, in its felt browns with the dark seam; the raft five logs in the
# log's browns with rope across; the rowboat a planked hull seen from above
# with its thwart and two oars, in the fence's wood.
BOATS = dict(FARM, **{"f": "#665133", "F": "#8a7048", "d": "#3a2d1c",
                      "r": "#4a3b26", "R": "#a88a52"})

ICONS.update({
    "goods/inner-tube": ([
        "................", "................", ".....kkkkkk.....", "...kkfFFFFfkk...",
        "..kfFfdrdffFfk..", "..kFfkkkkkkfFk..", ".kfFk......kFfk.", ".kFdk......kdFk.",
        ".kFrk......krFk.", ".kfFk......kFfk.", "..kFfkkkkkkfFk..", "..kfFfdrdffFfk..",
        "...kkfdddddkk...", ".....kkkkkk.....", "................", "................",
    ], BOATS),
    "goods/raft": ([
        "................", "................", ".kkkkkkkkkkkkkk.", ".kWWWWWWWWWWWWk.",
        ".kwwwwwwwwwwwwk.", ".kkRkkkkkkkkRkk.", ".kWRWWWWWWWWRWk.", ".kwRwwwwwwwwRwk.",
        ".kkRkkkkkkkkRkk.", ".kWRWWWWWWWWRWk.", ".kwRwwwwwwwwRwk.", ".kkRkkkkkkkkRkk.",
        ".kWWWWWWWWWWWWk.", ".kwwwwwwwwwwwwk.", ".kkkkkkkkkkkkkk.", "................",
    ], BOATS),
    "goods/rowboat": ([
        "................", ".......kk.......", "......kWWk......", ".....kWwwWk.....",
        "....kWwddwWk....", "....kWddddWk....", "k...kWddddWk...k", ".kkkkkkkkkkkkkk.",
        "....kWddddWk....", "....kWddddWk....", "k...kWddddWk...k", ".kk.kWddddWk.kk.",
        "....kWwddwWk....", "....kkWwwWkk....", "......kkkk......", "................",
    ], BOATS),
})


# The glowing mushroom (`glowing-mushrooms` design 3): the glowing plant the
# meadows grow, its low cap in the cyan its head burns with at night
# (`flora::GLOW_HEAD_RGB`), on the plant's pale green stalk, with the 1 px
# outline of an inventory icon.
GLOW = {
    "k": "#16302c", "c": "#2e8a7a", "C": "#5fd9c0", "L": "#a8ffe8", "W": "#e6fff8",
    "s": "#9bb675", "S": "#c1f1d8", "d": "#559642",
}

ICONS.update({
    "goods/glowing-mushroom": ([
        "................", "................", "......kkkk......", "....kkLWWLkk....",
        "...kCLLWWLLCk...", "..kCCLLLLLLCCk..", ".kcCCCLLLLCCCck.", ".kccCCCCCCCCcck.",
        "..kkccccccccck..", "....kkkkkkkk....", "......kSsk......", "......kSdk......",
        "......kSdk......", ".....kSssdk.....", ".....kkkkkk.....", "................",
    ], GLOW),
})


# Town jobs (`townsfolk` design 9): the parcel a delivery job hands you, brown
# paper tied with twine; and the coin beside the hotbar (design 5), a gold
# piece with a stamped ring. Inventory and UI icons, with the 1 px outline.
ECONOMY = {
    "k": "#3a2614", "p": "#b98a56", "P": "#d6a96e", "d": "#8a6238",
    "t": "#efe3c2", "c": "#7a4f12", "g": "#d9a334", "G": "#f4cf5c", "W": "#fff3b8",
    "o": "#a87521",
}

ICONS.update({
    "goods/parcel": ([
        "................", "................", "......k..k......", ".....ktkkt......",
        "...kkkkttkkkkk..", "..kPPPPttPPPPk..", "..kPppPttPppPk..", "..kPpppttpppPk..",
        "..kttttttttttk..", "..kpppptttpppk..", "..kppppttppppk..", "..kdppptttpddk..",
        "..kddddttddddk..", "...kkkkkkkkkk...", "................", "................",
    ], ECONOMY),
    "ui/coin": ([
        "................", "................", ".....kkkkkk.....", "....kGGGGGGk....",
        "...kGWWggggok...", "..kGWgooooggok..", "..kGgoGGGGogok..", "..kGgoGggGogok..",
        "..kGgoGggGogok..", "..kGgoGGGGogok..", "..kggoooooogok..", "...kggggggook...",
        "....koooooock...", ".....kkkkkk.....", "................", "................",
    ], ECONOMY),
})


# `townsfolk` design 31: gravel, a heap of crushed stone for a road.
ROADS = {
    "k": "#2e2a26", "w": "#e6e0d4", "l": "#c4bcae", "m": "#998f82", "d": "#6b645a",
}

ICONS.update({
    "goods/gravel": ([
        "................", "................", "................", "................",
        "......kkkk......", ".....kwlmlk.....", "....kllmdmlk....", "...kwmdlwmdlk...",
        "..klmlwmdlmwmk..", "..kdmlmdwlmdlk..", ".klwmdlmlmdwmlk.", ".kmdlwmdwlmldmk.",
        ".kdmmdmddmdmmdk.", "..kkkkkkkkkkkk..", "................", "................",
    ], ROADS),
})


# land-transport design 10: the cart wheel and the two carts. The grids are
# drawn by a few shapes (a ring of rim and iron tyre, spokes, planks, shafts)
# and then given the 1 px ink outline of an inventory icon wherever an empty
# pixel touches a filled one; the palette is the boats' browns, the iron
# ingot's grey for the tyre.
CARTS = dict(BOATS, **{"t": "#4b4a48", "s": "#c9a46a", "h": "#5a4127"})


def _outlined(grid):
    """Ink round the shapes: an empty pixel beside a filled one is 'k'."""
    out = [list(row) for row in grid]
    for y in range(16):
        for x in range(16):
            if grid[y][x] != ".":
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < 16 and 0 <= ny < 16 and grid[ny][nx] not in ".k":
                    out[y][x] = "k"
                    break
    return ["".join(r) for r in out]


def _wheel(grid, cx, cy, r, spokes):
    import math
    for y in range(16):
        for x in range(16):
            dx, dy = x + 0.5 - cx, y + 0.5 - cy
            d = math.hypot(dx, dy)
            a = math.atan2(dy, dx) % (math.pi * 2 / spokes)
            spoke = min(a, math.pi * 2 / spokes - a) * d < 0.55
            if d <= r and d > r - 0.9:
                grid[y][x] = "t"
            elif d <= r - 0.9 and d > r - 2.0:
                grid[y][x] = "F"
            elif d <= 1.3:
                grid[y][x] = "h"
            elif d <= r - 2.0 and spoke:
                grid[y][x] = "s"


def _fill(grid, x0, y0, x1, y1, c):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if 0 <= x < 16 and 0 <= y < 16:
                grid[y][x] = c


def _cart_icon(kind):
    g = [["."] * 16 for _ in range(16)]
    if kind == "wheel":
        _wheel(g, 8.0, 8.0, 7.0, 8)
    elif kind == "hand-cart":
        # The bed and its low side, its handle rising to the right, the
        # wheel under it.
        _fill(g, 1, 6, 10, 6, "F")
        _fill(g, 1, 4, 10, 5, "f")
        _fill(g, 1, 3, 1, 5, "F")
        for i, x in enumerate(range(11, 16)):
            _fill(g, x, 5 - i // 2, x, 5 - i // 2, "f")
        _wheel(g, 6.0, 11.0, 4.0, 6)
    else:
        # A longer bed with its seat, the shafts out to the right, a big
        # wheel under it.
        _fill(g, 1, 5, 9, 6, "f")
        _fill(g, 1, 7, 9, 7, "F")
        _fill(g, 1, 3, 1, 6, "F")
        _fill(g, 8, 3, 9, 4, "F")
        _fill(g, 10, 7, 15, 7, "f")
        _wheel(g, 5.5, 11.0, 4.5, 8)
    return _outlined(["".join(r) for r in g])


ICONS.update({
    f"goods/{name}": (_cart_icon(name), CARTS)
    for name in ("wheel", "hand-cart", "horse-cart")
})


def rgba(hex_colour):
    h = hex_colour.lstrip("#")
    return bytes(int(h[i:i + 2], 16) for i in (0, 2, 4)) + b"\xff"


def png(rows, palette):
    assert len(rows) == 16 and all(len(r) == 16 for r in rows)
    raw = b"".join(
        b"\x00" + b"".join(rgba(palette[c]) if c in palette else b"\x00\x00\x00\x00" for c in row)
        for row in rows
    )

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", 16, 16, 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def main():
    check = "--check" in sys.argv[1:]
    stale = []
    for name, (rows, palette) in ICONS.items():
        path = ROOT / f"{name}.png"
        data = png(rows, palette)
        if check:
            if not path.exists() or path.read_bytes() != data:
                stale.append(str(path))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    if stale:
        raise SystemExit("stale icons (run tools/gen_item_icons.py): " + ", ".join(stale))
    print(("checked " if check else "wrote ") + f"{len(ICONS)} icons")


if __name__ == "__main__":
    main()
