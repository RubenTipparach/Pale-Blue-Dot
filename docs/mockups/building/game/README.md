# The game's own textures, copied for the mockups

Byte-identical copies of textures the game already ships, so `building.html`
and `npcs.html` show the real tiles instead of redrawing them. Do not edit
them here; change the source and copy again.

| File | Source | Scale in the mockups |
| --- | --- | --- |
| `grass.png`, `dirt.png`, `reed.png`, `thatch.png`, `plank.png`, `boards.png`, `shingle.png`, `fieldstone.png`, `slate.png`, `brick.png`, `plaster.png`, `cobble.png`, `leaves.png` | `assets/textures/settlement/<name>.png` | `repeat_m` 2.0 from `assets/textures/settlement/manifest.ron` |
| `flag.png` | `assets/textures/settlement/flag.png` | `repeat_m` 2.5 |
| `bark.png`, `timber.png` | `assets/textures/settlement/<name>.png` | `repeat_m` 1.0 |
| `halftimber.png`, `stone.png`, `sand.png`, `cobble.png`, `iron.png`, `cloth.png`, `cloth2.png` | `assets/textures/settlement/<name>.png` | `repeat_m` from the manifest (`halftimber` mapped once per storey piece, as towns.html maps it) |
| `items/lantern_post.png`, `items/lantern_wall.png` | `assets/items/lights/<name>.png` | 16 px, shown at 2x in the hotbar |
| `items/pickaxe.png` | `assets/items/tools/pickaxe.png` | 16 px, shown at 2x in the hotbar |
| `atlas.png` | `assets/tilesets/atlas.png` | the page crops the lunar regolith sheet's "moon dust" tile (sheet 7, tile 0: pixels 384,128 to 416,160) at load; 2.0 m a repeat, as the settlement ground |

The atlas also gives the ground and trees of both pages (`scene-kit.js`): the fields sheet (#4) tiles (0,0) sward, (1,0) sward over earth on a column's top layer, (2,0) earth, (3,2) packed path, (2,1) oak bark, (0,2) oak leaves; the lunar regolith sheet (#7) on the moon.

All are sampled nearest, as the game samples them. Mockup-only art (pixel art
drawn in code with the balatro-pixel-art kit, `tools/art/building_art.py`) is the habitat metal panel and glass,
which the game does not have yet, and the piece icons.
