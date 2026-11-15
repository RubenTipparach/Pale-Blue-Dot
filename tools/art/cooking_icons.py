"""Game icons for `cooking` (design S1 and S2), from the M27 mockup's own art.

Run from the repository root: `python3 tools/art/cooking_icons.py`.

Sources: docs/mockups/cooking/ (M27, tools/art/cooking_art.py, drawn with the
balatro-pixel-art kit in Sweaft-64): the dishes at 24 px and the campfire
station at 32 px. Each is reduced to the game's 16 px slot icon the way
`farming_icons.py` reduces M8's and M3's art (area average, cut at half
alpha, every pixel snapped to a colour of its own source).

Which mockup picture each dish takes:
- grilled fish: `grilled_fish`; chili fish: `fillet_cooked`;
- flatbread: `flatbread`; roast turnip: `baked_potato` (a roast root);
- kale skewer: `skewers`; fried egg: `omelette`; roast meat:
  `meat_game_cooked`;
- the campfire (a light's icon, `items/lights/`): `st_campfire`.
"""

import sys
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))
from farming_icons import down  # noqa: E402

COOK = Path("docs/mockups/cooking")
OUT = Path("assets/items")

DISHES = {
    "grilled-fish": "grilled_fish",
    "chili-fish": "fillet_cooked",
    "flatbread": "flatbread",
    "roast-turnip": "baked_potato",
    "kale-skewer": "skewers",
    "fried-egg": "omelette",
    "roast-meat": "meat_game_cooked",
}


def main():
    for code, source in DISHES.items():
        down(Image.open(COOK / f"{source}.png"), 16).save(OUT / "goods" / f"{code}.png")
    down(Image.open(COOK / "st_campfire.png"), 16).save(OUT / "lights" / "campfire.png")


if __name__ == "__main__":
    main()
