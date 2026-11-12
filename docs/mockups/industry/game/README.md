# The game's own art, copied for the industry mockup

Byte-identical copies of files the game already ships, so `industry.html` shows the real
tiles instead of redrawing them. Do not edit them here; change the source and copy again.

| File | Source | Use in the mockup |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Every hex column's top and sides. The page crops 32 px tiles at load (sheet order in `crates/pbd-app/src/planet_terrain.rs`, tile names in `assets/tilesets/biomes.json`): fields #4 (pasture grass, grass over dirt, crumbly dirt, limestone, copper ore in limestone), mountains #8 (scree, slate, iron ore, dark gravel, angular pebbles), redwood forest #10 tile 15 (ancient redwood charcoal, standing in for a coal seam), basalt wastes #1 (ash crust, basalt, sulfur, cracked basalt), beach #2 (dry shell sand, wet sand, rounded pebbles), lunar regolith not used. 2 m a repeat on tops, one tile a cell edge and a metre on sides, as the terrain shader maps them. |
| `brick.png`, `cobble.png`, `flag.png`, `plank.png`, `boards.png`, `stone.png`, `slate.png`, `fieldstone.png`, `rubble.png`, `basalt.png`, `granite.png`, `sand.png` | `assets/textures/settlement/<name>.png` | Furnace and chimney walls, yard floors, the spoil heap, the dam's facing, filter beds, at `repeat_m` from `assets/textures/settlement/manifest.ron` (2.0, `flag` 2.5). |
| `iron.png`, `timber.png` | `assets/textures/settlement/<name>.png` | Fittings, pipes, posts, `repeat_m` 1.0. |
| `items/pickaxe.png`, `items/shovel.png`, `items/hand.png` | `assets/items/tools/` | 16 px, at 2x in the hotbar. |
| `items/lantern_post.png`, `items/torch.png`, `items/brazier.png` | `assets/items/lights/` | 16 px, at 2x in the hotbar. |

All are sampled nearest, as the game samples them. Everything else on the page is mockup-only
pixel art drawn with the balatro-pixel-art kit (`tools/art/industry_art.py`, files one folder up:
machine, generator and item icons, and the machine paint, belt, concrete, hazard, white cladding,
furnace grate and ether textures), plus the raw ore, coal and ingot icons reused from the crafting
mockup (`docs/mockups/crafting/`, `tools/art/crafting_art.py`). The shared scene code is
`docs/mockups/building/scene-kit.js` (the game's cell, tree, block light and lamp post).
