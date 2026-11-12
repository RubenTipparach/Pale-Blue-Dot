# The game's own textures, as the animals mockup uses them

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Hex tops and sides of every biome, sampled the way `assets/shaders/planet_surface.wgsl` (`pixel_tile`) does: sheet = slot in `planet_terrain.rs` `TILESETS`, top tile (0,0), first metre of side (1,0), earth below (2,0); snow from the tundra sheet ((2,2) top, (1,0) side) as the shader's snow slot does. |
| `bark.png`, `kapok.png`, `alder.png` | `assets/textures/settlement/` | Tree trunks |
| `leaves.png`, `needles.png` | `assets/textures/settlement/` | Canopies |
| `cactus.png` | `assets/textures/settlement/cactus.png` | Desert cactus |
| `timber.png` | `assets/textures/settlement/timber.png` | Lantern posts |
| `fieldstone.png` | `assets/textures/settlement/fieldstone.png` | Campfire ring |
| `linen.png` | `assets/textures/settlement/linen.png` | The hunter's tent |

Repeats are `repeat_m` from `assets/textures/settlement/manifest.ron`. Sampling is nearest, as the game's is.
