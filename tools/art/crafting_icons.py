"""Game icons for `tool-crafting`, made from the mockups' own pixel art.

Run from the repository root: `python3 tools/art/crafting_icons.py`.

Sources:
- docs/mockups/crafting/ (M3, tools/art/crafting_art.py, the balatro-pixel-art
  kit in Sweaft-64): fibre, stone, log, planks, rope, the workbench and the
  stone, bronze and iron picks (32 px), reduced to the game's 16 px by
  `farming_icons.down` (area, cut at half alpha, each pixel snapped to a
  colour of its own source).
- assets/items/tools/{shovel,axe}.png (the game's own 16 px tools): each tier's
  shovel and axe is the same picture with its head's three greys swapped for
  the tier's three. The game's tools were drawn as iron, so iron is them
  unchanged.
"""

from pathlib import Path

from PIL import Image

from farming_icons import CRAFT, OUT, down

# The heads' three greys in the game's tool icons, light to dark.
GREYS = [(238, 243, 244), (201, 211, 214), (125, 138, 143)]
# Each tier's three, light to dark, taken from M3's picks (pick-stone,
# pick-bronze) so a tier reads the same in the pack and in the picker.
TIER_GREYS = {
    "stone": [(184, 176, 160), (140, 132, 118), (92, 86, 78)],
    "bronze": [(246, 196, 120), (214, 140, 66), (140, 84, 38)],
    "iron": GREYS,
}


def swap(img, to):
    out = img.convert("RGBA").copy()
    px = out.load()
    for y in range(out.height):
        for x in range(out.width):
            r, g, b, a = px[x, y]
            if a and (r, g, b) in GREYS:
                px[x, y] = to[GREYS.index((r, g, b))] + (255,)
    return out


def main():
    for name in ("fibre", "stone", "log", "planks", "rope"):
        down(Image.open(CRAFT / f"{name}.png"), 16).save(OUT / "goods" / f"{name}.png")
    down(Image.open(CRAFT / "workbench.png"), 16).save(OUT / "stations" / "workbench.png")
    for tier in ("stone", "bronze", "iron"):
        down(Image.open(CRAFT / f"pick-{tier}.png"), 16).save(OUT / "tools" / f"{tier}-pickaxe.png")
        for tool in ("shovel", "axe"):
            swap(Image.open(OUT / "tools" / f"{tool}.png"), TIER_GREYS[tier]).save(
                OUT / "tools" / f"{tier}-{tool}.png"
            )


if __name__ == "__main__":
    main()
