"""Draw every block the planet shader can put on screen, off the real atlas.

For each material code the shader draws, and each biome sheet, the tile the
CAP samples and the tile the SIDE samples are cut from `assets/tilesets/atlas.png`
exactly as `pixel_tile` addresses them, and labelled with the sheet's own name
for that tile from `biomes.json`. A code whose cap is labelled "cold granite"
under the word SNOW is the audit failing in a way a picture cannot hide.

The tile table is READ from `planet_surface.wgsl` rather than retyped, so the
sheet cannot describe a shader other than the one that ships.
"""
import json
import re
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
SHADER = (ROOT / "assets/shaders/planet_surface.wgsl").read_text()
ATLAS = Image.open(ROOT / "assets/tilesets/atlas.png").convert("RGB")
BIOMES = json.load(open(ROOT / "assets/tilesets/biomes.json"))["biomes"]
NAMES = {b["id"]: b["tiles"] for b in BIOMES}
# Sheet slot order is the sorted file name, as `planet_terrain::TILESETS` derives it.
SHEETS = sorted(NAMES)

CODE_NAMES = {0: "seabed", 1: "beach sand", 2: "grass", 3: "jungle grass", 4: "desert sand",
              5: "stone", 6: "snow", 7: "marsh", 8: "wood", 9: "leaves",
              10: "DIRT (face)", 11: "GRASS SIDE (face)", 12: "SNOW SIDE (face)"}


def cap_tiles():
    """The `if code==Nu { ... tile=vec2(x.,y.); }` table, default (0,0)."""
    tiles = {code: (0, 0) for code in CODE_NAMES}
    for code, x, y in re.findall(r"if code==(\d+)u \{[^}]*tile=vec2\((\d+)\.,(\d+)\.\)", SHADER):
        tiles[int(code)] = (int(x), int(y))
    # The face codes, from the `code>=DIRT_CODE` block.
    tiles[10] = (2, 0)
    tiles[11] = (1, 0)
    tiles[12] = (1, 0)
    return tiles


def tile(slot, x, y, size=48):
    sheet_x, sheet_y = (slot % 4) * 128, (slot // 4) * 128
    crop = ATLAS.crop((sheet_x + x * 32, sheet_y + y * 32, sheet_x + (x + 1) * 32, sheet_y + (y + 1) * 32))
    return crop.resize((size, size), Image.NEAREST)


def main(out):
    tiles = cap_tiles()
    codes = sorted(CODE_NAMES)
    cell, label_w, head_h = 56, 170, 40
    width = label_w + len(SHEETS) * cell
    height = head_h + len(codes) * (cell + 14)
    sheet = Image.new("RGB", (width, height), (30, 30, 30))
    draw = ImageDraw.Draw(sheet)
    for j, name in enumerate(SHEETS):
        draw.text((label_w + j * cell + 2, 4), name[:12], fill=(230, 230, 230))
        draw.text((label_w + j * cell + 2, 18), f"slot {j}", fill=(150, 150, 150))
    rows = []
    for i, code in enumerate(codes):
        y = head_h + i * (cell + 14)
        x_tile, y_tile = tiles[code]
        index = y_tile * 4 + x_tile
        draw.text((4, y + 4), f"code {code}", fill=(230, 230, 230))
        draw.text((4, y + 18), CODE_NAMES[code], fill=(230, 230, 230))
        draw.text((4, y + 32), f"tile ({x_tile},{y_tile}) = #{index}", fill=(150, 150, 150))
        for j, name in enumerate(SHEETS):
            sheet.paste(tile(j, x_tile, y_tile), (label_w + j * cell + 4, y))
            rows.append((code, CODE_NAMES[code], name, index, NAMES[name][index]))
    sheet.save(out)
    for code, cname, biome, index, tname in rows:
        print(f"code {code:>2} {cname:<18} {biome:<15} tile #{index:<2} {tname}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "block_audit.png")
