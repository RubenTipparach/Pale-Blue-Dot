# The game's own art, as the ship-crafting mockup uses it

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | The field round the yard: hex columns topped with the fields sheet (slot 4 in `planet_terrain.rs` `TILESETS`), ground tile (0,0) on top, its side tile (1,0) on the first metre, earth (2,0) below, as `planet_surface.wgsl` picks them |
| `granite.png` | `assets/textures/settlement/granite.png` | The apron (repeat_m 2) |
| `iron.png` | `assets/textures/settlement/iron.png` | The pads and the floodlight masts (repeat_m 1) |
| `stone.png` | `assets/textures/settlement/stone.png` | The pads' rims and the hangar walls (repeat_m 2) |
| `plank.png` | `assets/textures/settlement/plank.png` | The boardwalk across the apron and the boat cradles (repeat_m 2) |
| `slate.png` | `assets/textures/settlement/slate.png` | The hangar roof (repeat_m 2) |
| `kestrel.glb`, `tern.glb`, `loon.glb` | `assets/models/vehicles/<craft>/<craft>.glb` (atlas embedded) | The game's craft, parked by the build bay as a scale and style reference |
| `hand.png` | `assets/items/tools/hand.png` | The hotbar's first slot, as the game draws it |

Sampling is nearest with no mipmaps, as the game's is. The ships' module icons
and hull textures (`../*.png`) have no counterpart in the game and are drawn by
`tools/art/ship-crafting_art.py` with the balatro-pixel-art kit.

The published artifact cannot serve `.glb`, so it carries each model as `<craft>.glb.txt`, the same bytes in base64; the page loads the `.glb` where it is served (a local checkout) and falls back to the text copy.
