# The game's own art, as the hangar mockup (M24) uses it

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | The field round the airfield: hex columns topped with the fields sheet (slot 4 in `planet_terrain.rs` `TILESETS`), ground tile (0,0) on top, its side tile (1,0) on the first metre, earth (2,0) below, as `planet_surface.wgsl` picks them |
| `granite.png` | `assets/textures/settlement/granite.png` | The apron, the taxiway and the hangar floor (repeat_m 2) |
| `flag.png` | `assets/textures/settlement/flag.png` | The helipad (repeat_m 2.5) |
| `stone.png` | `assets/textures/settlement/stone.png` | The hangar's walls and pillars, the tower (repeat_m 2) |
| `slate.png` | `assets/textures/settlement/slate.png` | The hangar's roof (repeat_m 2) |
| `iron.png` | `assets/textures/settlement/iron.png` | Struts, masts, hubs, floodlight poles, the windsock's pole, the hangar's roof trusses (repeat_m 1) |
| `plank.png` | `assets/textures/settlement/plank.png` | The workbench and the wheel chocks (repeat_m 2) |
| `kestrel.glb` | `assets/models/vehicles/kestrel/kestrel.glb` (atlas embedded) | The game's tiltrotor, parked on the apron as the scale and style reference the chibi aircraft are measured against |
| `hand.png` | `assets/items/tools/hand.png` | The hotbar's first slot, as the game draws it |

Sampling is nearest with no mipmaps, as the game's is. The part icons, the
livery paints and decals (shark mouth, roundels, digits), the canopy glass,
rotor blades, tyres, the hangar door, the runway's asphalt and markings and the
windsock's stripes (`../*.png`) have no counterpart in the game and are drawn
by `tools/art/hangar_art.py` with the balatro-pixel-art kit.

The published artifact cannot serve `.glb`, so it carries the model as
`kestrel.glb.txt`, the same bytes in base64; the page loads the `.glb` where it
is served (a local checkout) and falls back to the text copy.
