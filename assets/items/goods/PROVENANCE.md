# Provenance of the goods, crop and station art

All made by `tools/art/farming_icons.py` (`farming-and-smelting` task 4.7)
from the mockups' own pixel art, which was drawn in code with the
balatro-pixel-art kit in the Sweaft-64 palette:

- `docs/mockups/farming/` (mockup M8, `tools/art/farming_art.py`): crop stage
  sprites (32 px), produce and the seed sack (24 px);
- `docs/mockups/crafting/` (mockup M3, `tools/art/crafting_art.py`): coal,
  tin ore, the ingots and the forge (32 px).

| Where | What | How |
| --- | --- | --- |
| `assets/items/crops/<crop>_<stage>.png` | M8's four stage sprites per crop | copied byte for byte, 32 px |
| `assets/items/crops/flame.png` | the forge's flame in the world | the fire of M3's `forge.png`, cut out alone |
| `assets/items/goods/<crop>.png` | produce | M8's produce, reduced to 16 px |
| `assets/items/goods/<crop>-seed.png` | seeds | M8's seed sack at 16 px, the produce as a 9 px badge |
| `assets/items/goods/{coal,tin-ore,copper-ingot,iron-ingot,bronze-ingot}.png` | smelting | M3's icons, reduced to 16 px |
| `assets/items/stations/forge.png` | the forge block's slot icon | M3's `forge.png`, reduced to 16 px |

A reduction is an area average cut at half alpha, each pixel snapped to a
colour of its own source, so no colour enters that the source lacks. Every
picture is drawn nearest-sampled. Each good's row in `pbd_core::goods` names
its file, and `farm::tests::every_good_has_its_icon` holds them together.

## tool-crafting

Made by `tools/art/crafting_icons.py` (`tool-crafting` task 2.1), by the same
reduction:

| Where | What | How |
| --- | --- | --- |
| `assets/items/goods/{fibre,stone,log,planks,rope}.png` | tier 0 and the hands work | M3 icons, reduced to 16 px |
| `assets/items/stations/workbench.png` | the workbench block slot icon | M3 `workbench.png`, reduced to 16 px |
| `assets/items/tools/{stone,bronze,iron}-pickaxe.png` | a pick of each tier | M3 `pick-<tier>.png`, reduced to 16 px |
| `assets/items/tools/{stone,bronze,iron}-{shovel,axe}.png` | a shovel and an axe of each tier | the game own `shovel.png` and `axe.png` with the head three greys swapped for the tier three (iron is the picture as drawn) |
