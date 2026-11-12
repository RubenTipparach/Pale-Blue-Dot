# The game's own art, as the ship-tiers mockup uses it

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again
(`python tools/art/ship_tiers_art.py` copies them all).

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Every hex column: the launch field (fields sheet, slot 4 in `planet_terrain.rs` `TILESETS`), the landing sites (lunar_regolith 7, basalt_wastes 1, frozen_wastes 5) and the cells of each globe from orbit; ground tile (0,0) on top, its side (1,0) on the first metre, earth (2,0) below, as `planet_surface.wgsl` picks them |
| `iron.png` | `assets/textures/settlement/iron.png` | The launch pad and the gantry (repeat_m 1) |
| `granite.png` | `assets/textures/settlement/granite.png` | The apron round the pad (repeat_m 2) |
| `stone.png` | `assets/textures/settlement/stone.png` | The pad's rim and the flame trench (repeat_m 2) |
| `slate.png` | `assets/textures/settlement/slate.png` | The assembly hall's roof (repeat_m 2) |
| `plank.png` | `assets/textures/settlement/plank.png` | The kestrel's and the tern's cradles (repeat_m 2) |
| `kestrel.glb`, `tern.glb`, `loon.glb` | `assets/models/vehicles/<craft>/<craft>.glb` (atlas embedded) | The game's craft, parked by the pad: the ships before the tiers, and a scale reference |
| `hand.png`, `pickaxe.png` | `assets/items/tools/` | Hotbar |
| `lantern_post.png`, `torch.png` | `assets/items/lights/` | Hotbar |

`*.gltf.json` beside each `.glb` is the same model repackaged losslessly as glTF
JSON with its binary buffer embedded as base64 (the artifact host does not serve
`.glb`); the page loads that. The `.glb` stays here byte-identical to the source.

Sampling is nearest with no mipmaps, as the game's is. The part icons and part
textures one folder up (`../*.png`) have no counterpart in the game and are drawn
by `tools/art/ship_tiers_art.py` with the balatro-pixel-art kit.
