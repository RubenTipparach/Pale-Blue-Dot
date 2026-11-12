# The game's own art, used by stations.html

Copied byte-identical from the game's assets; nothing here is redrawn.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Kessler Rock's hex columns: sheet 0 ("asteroid"), tile 0 metal dust on the tops, tile 3 nickel iron rock and tile 4 iron ore on the sides, tile 13 sintered metal plates on the landing pad |
| `iron.png` | `assets/textures/settlement/iron.png` (repeat 1.0 m) | spokes, rails, the mass driver, gun barrels |
| `kestrel.glb` | `assets/models/vehicles/kestrel/kestrel.glb` (atlas embedded) | the ship on the docking run and the NPC through the scan gate |

Everything else under `stations/` is mockup-only pixel art drawn by `tools/art/stations_art.py`
with the balatro-pixel-art kit.

`kestrel.glb.js` holds the same GLB bytes as base64 (`window.KESTREL_GLB`), because the artifact host
does not serve `.glb`; the page parses it with GLTFLoader. Regenerate it from `kestrel.glb` with the
command at its top whenever the GLB is recopied.
