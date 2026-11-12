# The game's own art, as planets.html uses it

Copied byte-identical from this repository on 2026-11-12, so the mockup shows
the game's textures rather than redrawn ones (docs/mockups/ART-DIRECTION.md).
Re-copy from the source when the source changes; never edit these copies.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Vignette hex columns (sheet order `planet_terrain.rs` TILESETS; ground (0,0), ground-into-earth (1,0), earth (2,0), stone (3,0), wood (2,1), leaves (0,2)); hotbar block icons |
| `plaster.png`, `thatch.png`, `timber.png`, `plank.png`, `leaves.png`, `bark.png`, `iron.png`, `stone.png`, `shingle.png`, `cloth.png` | `assets/textures/settlement/` (repeat from its `manifest.ron`) | Houses, habs, posts, decks, rigs |
| `kestrel.glb` | `assets/models/vehicles/kestrel/kestrel.glb` | The landed ship (atlas embedded in the GLB) |
| `tern.glb` | `assets/models/vehicles/tern/tern.glb` | The boat at Tidewater's jetty |
| `pickaxe.png`, `shovel.png` | `assets/items/tools/` | Hotbar |
| `torch.png`, `lantern_post.png`, `lantern_wall.png`, `lantern_hanging.png`, `brazier.png`, `candle.png` | `assets/items/lights/` | Hotbar |

Everything else under `docs/mockups/planets/` (maps, lights maps, portraits,
resource icons) is mockup-only pixel art drawn by `tools/art/planets_art.py`
with the balatro-pixel-art kit.

`*.gltf.json` beside each `.glb` is the same model repackaged losslessly as glTF
JSON with its binary buffer embedded as base64 (the artifact host does not serve
`.glb`); the page loads that. The `.glb` stays here byte-identical to the source.
