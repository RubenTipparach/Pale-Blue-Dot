# The game's own textures, as the farming mockup uses them

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Hex tops and sides, sampled the way `assets/shaders/planet_surface.wgsl` (`pixel_tile`) does: sheet = slot in `planet_terrain.rs` `TILESETS`, top tile (0,0), first metre of side (1,0), earth below (2,0). Tilled soil is the fields sheet's tile (2,3) "tilled soil" (`assets/tilesets/biomes.json`). |
| `timber.png` | `assets/textures/settlement/timber.png` | Lantern posts, pen fence (repeat 1.0 m) |
| `plank.png` | `assets/textures/settlement/plank.png` | Water trough (repeat 2.0 m) |
| `plaster.png` | `assets/textures/settlement/plaster.png` | Farmhouse walls (repeat 2.0 m) |
| `thatch.png` | `assets/textures/settlement/thatch.png` | Farmhouse roof (repeat 2.0 m) |
| `boards.png` | `assets/textures/settlement/boards.png` | Farmhouse door (repeat 2.0 m) |
| `iron.png` | `assets/textures/settlement/iron.png` | Greenhouse frame, airlock (repeat 1.0 m) |
| `fieldstone.png` | `assets/textures/settlement/fieldstone.png` | Greenhouse footing (repeat 2.0 m) |

Repeats are `repeat_m` from `assets/textures/settlement/manifest.ron`. Sampling is nearest, as the game's is.
