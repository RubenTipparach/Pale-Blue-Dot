"""Building planner icons (`building-planner` task 3.1): the 16 px slot icons
of the mallet (a good) and the drafting table (a station), in the furniture
icons' style and palette: flat tones lit from the top left, a 1 px dark
outline (icons are UI, CLAUDE.md), no anti-aliasing. Also the planner's
Erase tool, M6's own rubber copied unchanged into build mode's icon folder.

    python3 tools/art/planner_icons.py            # writes the icons
    python3 tools/art/planner_icons.py --sheet X  # also a contact sheet
"""

import shutil
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]

OUTLINE = (22, 22, 18)
PAL = {
    # wood, the planks' ramp
    "0": (65, 50, 44),
    "1": (114, 82, 68),
    "2": (171, 132, 101),
    "3": (207, 170, 129),
    # iron
    "i": (52, 56, 64),
    "j": (98, 104, 112),
    "k": (150, 156, 160),
    # drafting paper and its blue lines
    "l": (170, 160, 140),
    "m": (214, 206, 186),
    "n": (238, 232, 214),
    "b": (80, 130, 160),
}

ICONS = {
    # a wooden mallet: a round-cornered head on a handle, held head up
    "mallet": [
        "................",
        "................",
        "....12333.......",
        "...1223333......",
        "...12223332.....",
        "...112223321....",
        "....11222211....",
        ".....1111112....",
        "........012.....",
        ".......012......",
        "......012.......",
        ".....012........",
        "....012.........",
        "...012..........",
        "...00...........",
        "................",
    ],
    # a drafting table: a sloped board with a sheet on it, on a frame
    "drafting-table": [
        "................",
        "................",
        "..........3333..",
        "......33333nn2..",
        "..3333nnnnbnm2..",
        ".23nnnnnbnnnm1..",
        ".2nnbbbbbbnml1..",
        ".2nnnnnbnnml11..",
        ".2mmmmmmmmll1...",
        ".1111111111111..",
        "..12........12..",
        "..12.1....1.12..",
        "..1211111111.2..",
        "..12........12..",
        "..00........00..",
        "................",
    ],
}


def paint(rows):
    im = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    for y, row in enumerate(rows):
        assert len(row) == 16, (y, row)
        for x, ch in enumerate(row):
            if ch in PAL:
                im.putpixel((x, y), PAL[ch] + (255,))
    # the 1 px outline round everything drawn
    px = im.load()
    out = im.copy()
    for y in range(16):
        for x in range(16):
            if px[x, y][3]:
                continue
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                u, v = x + dx, y + dy
                if 0 <= u < 16 and 0 <= v < 16 and px[u, v][3] and px[u, v][:3] != OUTLINE:
                    out.putpixel((x, y), OUTLINE + (255,))
                    break
    return out


def main(argv):
    made = {name: paint(rows) for name, rows in ICONS.items()}
    made["mallet"].save(ROOT / "assets" / "items" / "goods" / "mallet.png")
    made["drafting-table"].save(ROOT / "assets" / "items" / "stations" / "drafting-table.png")
    # The planner's Erase tool: M6 version 3's rubber, unchanged.
    shutil.copyfile(
        ROOT / "docs" / "mockups" / "building" / "icon_eraser.png",
        ROOT / "assets" / "textures" / "ui" / "build" / "erase.png",
    )
    if "--sheet" in argv:
        path = argv[argv.index("--sheet") + 1]
        sheet = Image.new("RGBA", (len(made) * 136, 136), (40, 38, 34, 255))
        for k, im in enumerate(made.values()):
            big = im.resize((128, 128), Image.NEAREST)
            sheet.paste(big, (k * 136 + 4, 4), big)
        sheet.save(path)


if __name__ == "__main__":
    main(sys.argv[1:])
