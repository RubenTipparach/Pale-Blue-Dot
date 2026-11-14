# The game's own art, as the cooking mockup uses it

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Hex tops and sides outdoors, sampled the way `assets/shaders/planet_surface.wgsl` (`pixel_tile`) does: the fields sheet (slot 4 in `planet_terrain.rs` `TILESETS`), top tile (0,0), first metre of side (1,0), earth below (2,0). |
| `brick.png`, `brickdark.png` | `assets/textures/settlement/` | Bake oven, chimneys, the kitchen's hearth surround (repeat 2.0 m) |
| `clay.png`, `claytile.png` | `assets/textures/settlement/` | Pizza oven dome, yard oven (repeat 2.0 m) |
| `stone.png`, `fieldstone.png`, `cobble.png`, `flag.png` | `assets/textures/settlement/` | Hearth, quern, fire ring; footings; yard paving; floors (repeat 2.0 m) |
| `plank.png`, `boards.png`, `timber.png` | `assets/textures/settlement/` | Tables, shelves, racks, smoker, icebox (plank, boards 2.0 m; timber 1.0 m) |
| `plaster.png`, `whitewash.png` | `assets/textures/settlement/` | Bakery and kitchen walls (repeat 2.0 m) |
| `tile.png`, `slate.png`, `thatch.png` | `assets/textures/settlement/` | Inn kitchen floor, smoker roof, the farmhouse roof (repeat 2.0 m) |
| `iron.png` | `assets/textures/settlement/` | Stove, pot, wok, fryer, hooks (repeat 1.0 m) |
| `barrel.png` | `assets/textures/settlement/` | Brine and ferment barrels (mapped once per face, `repeat_m` 0) |
| `linen.png` | `assets/textures/settlement/` | Flour sacks, aprons on hooks (repeat 1.0 m) |
| `fish-<species>.png` | `assets/items/fish/<species>.png` | The eight fish the game catches today (silverfin is the "any caught fish" ingredient) |
| `light-candle.png`, `light-lantern_post.png`, `light-lantern_hanging.png` | `assets/items/lights/candle.png`, `lantern_post.png`, `lantern_hanging.png` | The kitchens' lights, beside "Lit at night by" |
| `tool-rod.png`, `tool-hand.png` | `assets/items/tools/rod.png`, `hand.png` | The rod (where fish come from, in the sea table) and the hand (prep with no station) |

Repeats are `repeat_m` from `assets/textures/settlement/manifest.ron`. Sampling is nearest, as the game's is.
