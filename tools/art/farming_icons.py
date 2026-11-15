"""Game icons for `farming-and-smelting`, made from the mockups' own pixel art.

Run from the repository root: `python3 tools/art/farming_icons.py`.

Sources (drawn with the balatro-pixel-art kit for the mockups, Sweaft-64):
- docs/mockups/farming/ (M8, tools/art/farming_art.py): crop stages (32 px),
  produce and the seed sack (24 px);
- docs/mockups/crafting/ (M3, tools/art/crafting_art.py): coal, tin ore,
  ingots and the forge (32 px).

Crop stage sprites are copied byte for byte (32 px, the world draws them on
crossed quads). Slot icons are the game's 16 px: each is reduced by area,
cut at half alpha and every pixel snapped to a colour of its own source, so
no new colour enters the palette. A seed is the sack with its crop's produce
as a badge. The forge's flame is the fire of M3's forge icon cut out alone.
"""

import shutil
from pathlib import Path

from PIL import Image

FARM = Path("docs/mockups/farming")
CRAFT = Path("docs/mockups/crafting")
OUT = Path("assets/items")


def palette(img):
    return list({p[:3] for p in img.getdata() if p[3] > 0})


def down(img, size):
    img = img.convert("RGBA")
    pal = palette(img)
    small = img.resize((size, size), Image.BOX)
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    for y in range(size):
        for x in range(size):
            r, g, b, a = small.getpixel((x, y))
            if a < 110:
                continue
            best = min(pal, key=lambda c: (c[0] - r) ** 2 + (c[1] - g) ** 2 + (c[2] - b) ** 2)
            out.putpixel((x, y), best + (255,))
    return out


def main():
    for d in ("goods", "crops", "stations"):
        (OUT / d).mkdir(parents=True, exist_ok=True)
    sack = down(Image.open(FARM / "tool_seeds.png"), 16)
    for crop in ("wheat", "turnip", "frost_kale", "chili"):
        code = crop.replace("_", "-")
        for stage in range(4):
            shutil.copyfile(FARM / f"{crop}_{stage}.png", OUT / "crops" / f"{crop}_{stage}.png")
        produce = Image.open(FARM / f"produce_{crop}.png")
        down(produce, 16).save(OUT / "goods" / f"{code}.png")
        seed = sack.copy()
        seed.alpha_composite(down(produce, 9), (7, 7))
        seed.save(OUT / "goods" / f"{code}-seed.png")
    for name in ("coal", "tin-ore", "copper-ingot", "iron-ingot", "bronze-ingot"):
        down(Image.open(CRAFT / f"{name}.png"), 16).save(OUT / "goods" / f"{name}.png")
    down(Image.open(CRAFT / "forge.png"), 16).save(OUT / "stations" / "forge.png")
    forge = Image.open(CRAFT / "forge.png").convert("RGBA")
    flame = Image.new("RGBA", (32, 32), (0, 0, 0, 0))
    for y in range(32):
        for x in range(32):
            r, g, b, a = forge.getpixel((x, y))
            if a and r > 200 and g < 220 and b < 120:
                flame.putpixel((x, y), (r, g, b, 255))
    flame = flame.crop(flame.getbbox())
    w, h = flame.size
    out = Image.new("RGBA", (max(w, h), max(w, h)), (0, 0, 0, 0))
    out.paste(flame, ((out.width - w) // 2, out.height - h))
    out.save(OUT / "crops" / "flame.png")


if __name__ == "__main__":
    main()
