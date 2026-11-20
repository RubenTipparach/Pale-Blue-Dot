# Foliage playground art (M13)

`foliage.html` loads everything in this folder by a relative path. `game/` holds byte-identical
copies of the game's own textures (see `game/README.md`). The rest is mockup-only pixel art, made
by scripts, never by hand:

| Files | Made by | Notes |
| --- | --- | --- |
| `icon_*.png`, `cap_*.png`, `bark_palm.png`, `imp_*_bare.png` | `tools/art/foliage_art.py` | The biome picker's icons are UI and keep their 1 px outline. |
| `flower_*.png`, `imp_<species>.png`, `bark_birch.png` | `world_art.py` (this folder), after `foliage_art.py` | The world's versions: no outline ring, no seams, no ink (owner, 2026-11-15). |
| `fruit/hang_*.png`, `fruit/plant_*.png` | `fruit_art.py` (this folder) | Wild fruit, world style. Leaves and vines are drawn in three KEY greens that the page swaps for the biome's ground ramp. |
| `mountain/alpine_turf.png`, `mountain/sheet.png` | `alpine_turf.py` (this folder) | The Mountains tab (`mountain-ground` design 9): the alpine turf candidate, world style, no outline. Its plants are the playground's kit, not art of their own (M13 v6). |
| `crops/*.png` | `../farming/world_art.py` | Byte-identical to `../farming/world/`: M8's crop stages, outline-free, so both pages draw a crop the same way. |

Order to regenerate: `python tools/art/foliage_art.py`, then `python docs/mockups/foliage/world_art.py`,
`python docs/mockups/foliage/fruit_art.py`, `python docs/mockups/foliage/alpine_turf.py` and `python docs/mockups/farming/world_art.py`.
