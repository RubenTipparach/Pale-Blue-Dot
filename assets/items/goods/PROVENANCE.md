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
| `assets/items/crops/<crop>_<stage>.png` | M8's four stage sprites per crop, in their outline-free world version (`docs/mockups/farming/world/`, drawn by `docs/mockups/farming/world_art.py`; since 2026-11-15, `flora-v2` design 11) | copied byte for byte, 32 px |
| `assets/items/crops/flame.png` | the forge's flame in the world | the fire of M3's `forge.png`, cut out alone |
| `assets/items/goods/<crop>.png` | produce | M8's produce, reduced to 16 px |
| `assets/items/goods/<crop>-seed.png` | seeds | M8's seed sack at 16 px, the produce as a 9 px badge |
| `assets/items/goods/{coal,tin-ore,copper-ingot,iron-ingot,bronze-ingot}.png` | smelting | M3's icons, reduced to 16 px |
| `assets/items/stations/forge.png` | the forge block's slot icon | M3's `forge.png`, reduced to 16 px |

A reduction is an area average cut at half alpha, each pixel snapped to a
colour of its own source, so no colour enters that the source lacks. Every
picture is drawn nearest-sampled. Each good's row in `pbd_core::goods` names
its file, and `farm::tests::every_good_has_its_icon` holds them together.

## animal-husbandry

`fence`, `gate`, `egg`, `wool` and `milk` are new to Pale Blue Dot and drawn by
`tools/gen_item_icons.py` (`animal-husbandry` design 12, S6) from its pixel
grids and one palette (`FARM`); `python3 tools/gen_item_icons.py --check`
fails if a PNG differs from what it draws: two posts and two rails; a gate
with its brace; a cream egg; a shorn fleece; an iron pail of milk.

## Cooking (`cooking` design S1)

The dishes (`grilled-fish`, `chili-fish`, `flatbread`, `roast-turnip`,
`kale-skewer`, `fried-egg`, `roast-meat`) are made by
`tools/art/cooking_icons.py` from the M27 mockup's own pixel art
(`docs/mockups/cooking/`, `tools/art/cooking_art.py`, balatro-pixel-art kit,
Sweaft-64), reduced from 24 px to 16 px the same way as above. The script
names which mockup picture each dish takes.

## tool-crafting

Made by `tools/art/crafting_icons.py` (`tool-crafting` task 2.1), by the same
reduction:

| Where | What | How |
| --- | --- | --- |
| `assets/items/goods/{fibre,stone,log,planks,rope}.png` | tier 0 and the hands work | M3 icons, reduced to 16 px |
| `assets/items/stations/workbench.png` | the workbench block slot icon | M3 `workbench.png`, reduced to 16 px |
| `assets/items/tools/{stone,bronze,iron}-pickaxe.png` | a pick of each tier | M3 `pick-<tier>.png`, reduced to 16 px |
| `assets/items/tools/{stone,bronze,iron}-{shovel,axe}.png` | a shovel and an axe of each tier | the game own `shovel.png` and `axe.png` with the head three greys swapped for the tier three (iron is the picture as drawn) |

`fertile-egg` and `incubator` (`animal-husbandry` design 13, E3 and E4) are
drawn the same way by `tools/gen_item_icons.py`, in the same palette with a
speckle brown and two glass blues (`HATCH`).

## flora-v2 design 12.5: the trees' fruit

`apple`, `pear`, `plum`, `cherry`, `mango`, `papaya`, `marula`, `baobab` and
`pine-cone` are M13's own hanging-fruit sprites
(`docs/mockups/foliage/fruit/hang_<kind>.png`, 16 px, drawn by
`docs/mockups/foliage/fruit_art.py` in the Sweaft-64 palette), given the 1 px
outline of an inventory icon (the sprite's darkest tone at 45%) by
`tools/gen_fruit_icons.py`; `--check` holds them.

