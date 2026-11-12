# The game's own art, as travel.html uses it

Copied byte-identical from this repository on 2026-11-12 (docs/mockups/ART-DIRECTION.md).
Re-copy from the source when the source changes; never edit these copies.

| File | Source | Used for |
| --- | --- | --- |
| `kestrel.glb` | `assets/models/vehicles/kestrel/kestrel.glb` | The ship flying the transfer (atlas embedded in the GLB) |
| `iron.png`, `plank.png`, `plaster.png`, `thatch.png` | `assets/textures/settlement/` | Highport's hull blocks, the departure pad and its houses, the skimmer platform's deck, hab and rails |
| `atlas.png` | `assets/tilesets/atlas.png` | Hotbar block icons (side tiles at 1x) |
| `pickaxe.png`, `shovel.png` | `assets/items/tools/` | Hotbar |
| `torch.png`, `lantern_post.png`, `lantern_wall.png`, `lantern_hanging.png`, `brazier.png`, `candle.png` | `assets/items/lights/` | Hotbar |

The travel icons under `docs/mockups/travel/` are mockup-only pixel art drawn by
`tools/art/travel_art.py`; the globes' maps come from `docs/mockups/planets/`.

`*.gltf.json` beside each `.glb` is the same model repackaged losslessly as glTF
JSON with its binary buffer embedded as base64 (the artifact host does not serve
`.glb`); the page loads that. The `.glb` stays here byte-identical to the source.
