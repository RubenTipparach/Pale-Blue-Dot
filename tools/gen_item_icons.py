#!/usr/bin/env python3
"""Draw the item icons this repository makes itself: the three fish new to
Pale Blue Dot and the four tools, as 16x16 RGBA PNGs under assets/items/.

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

WOOD, WOOD2 = "#8a5a2b", "#5e3a18"
IRON, IRON2 = "#c9d3d6", "#7d8a8f"

ICONS = {
    "tools/rod": ([
        "..............ww", "............ww.l", "...........w...l", "..........w....l",
        ".........w.....l", "........w......l", ".......w.......l", "......w........l",
        ".....w.........l", "....w.........bb", "...h..........bb", "..hh............",
        ".hh.............", "hh..............", "h...............", "................",
    ], {"w": "#b9a27a", "h": WOOD2, "l": "#dfe8ea", "b": "#e35d4a"}),
    "tools/shovel": ([
        "............hh..", "...........hwwh.", "..........hwwh..", ".........w.hh...",
        "........w.......", ".......w........", "......w.........", "....ii..........",
        "...iiii.........", "..iiiiii........", ".iiiiii.........", ".iiiii..........",
        ".iiii...........", "..ii............", "................", "................",
    ], {"h": WOOD2, "w": WOOD, "i": IRON}),
    "tools/pickaxe": ([
        "...iiiii........", "..i.....iii.....", ".i.......wIi....", ".........w..i...",
        "........w....i..", ".......w......i.", "......w.......i.", ".....w..........",
        "....w...........", "...w............", "..w.............", ".w..............",
        "w...............", "................", "................", "................",
    ], {"i": IRON, "I": IRON2, "w": WOOD}),
    "tools/axe": ([
        ".......iii......", "......iiiii.....", ".....iiiiIw.....", "......iiIw......",
        ".......Iw.......", "......w.........", ".....w..........", "....w...........",
        "...w............", "..w.............", ".w..............", "w...............",
        "................", "................", "................", "................",
    ], {"i": IRON, "I": IRON2, "w": WOOD}),
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
