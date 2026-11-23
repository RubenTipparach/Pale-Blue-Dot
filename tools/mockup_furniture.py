"""M15's Furniture scene (`furniture` task 1.2): put the game's own furniture
models into `docs/mockups/towns.html`, between its FURNITURE_MODELS markers,
and the chest panel's item icons into its CHEST_ICONS markers.

The models come from `pbd_core::furniture::model`, written out by the core's
ignored test:

    PBD_FURNITURE_EXPORT=docs/mockups/furniture-models.json \
        cargo test --profile fast -p pbd-core -- --ignored export_models
    python3 tools/mockup_furniture.py
"""

import base64
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGE = ROOT / "docs" / "mockups" / "towns.html"
MODELS = ROOT / "docs" / "mockups" / "furniture-models.json"
ICONS = {
    "wheat-seed": "assets/items/goods/wheat-seed.png",
    "wheat": "assets/items/goods/wheat.png",
    "turnip": "assets/items/goods/turnip.png",
    "fibre": "assets/items/goods/fibre.png",
    "planks": "assets/items/goods/planks.png",
    "rope": "assets/items/goods/rope.png",
    "coal": "assets/items/goods/coal.png",
    "stone": "assets/items/goods/stone.png",
    "coin": "docs/mockups/mission-tree/items/coin.png",
}


def between(text, begin, end, body):
    a = text.index(begin) + len(begin)
    b = text.index(end, a)
    return text[:a] + "\n" + body + "\n" + text[b:]


def main():
    page = PAGE.read_text()
    models = json.loads(MODELS.read_text())
    body = "const FURNITURE_MODELS = " + json.dumps(models, separators=(",", ":")) + ";"
    page = between(page, "// FURNITURE_MODELS:BEGIN", "// FURNITURE_MODELS:END", body)
    icons = {
        k: "data:image/png;base64," + base64.b64encode((ROOT / v).read_bytes()).decode()
        for k, v in ICONS.items()
    }
    page = between(
        page,
        "// CHEST_ICONS:BEGIN",
        "// CHEST_ICONS:END",
        "const CHEST_ICONS = " + json.dumps(icons) + ";",
    )
    PAGE.write_text(page)
    print(f"{PAGE}: {len(models)} models, {len(icons)} icons")


if __name__ == "__main__":
    main()
