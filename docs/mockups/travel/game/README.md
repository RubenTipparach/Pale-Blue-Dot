# The game's own art, as travel.html uses it

Reworked 2026-11-14 (the trip is now made on M21's rockets): the Kestrel is parked on the
launch field; granite, stone and slate added for the pads, apron and assembly hall.

Copied byte-identical from this repository on 2026-11-12 (docs/mockups/ART-DIRECTION.md).
Re-copy from the source when the source changes; never edit these copies.

| File | Source | Used for |
| --- | --- | --- |
| `kestrel.glb` | `assets/models/vehicles/kestrel/kestrel.glb` | The Kestrel parked on the launch field's apron (atlas embedded in the GLB) |
| `iron.png`, `granite.png`, `stone.png`, `slate.png` | `assets/textures/settlement/` | The pads' rims, masts and gantry (iron), the apron (granite), the pads (stone), the assembly hall's roof (slate), Lantern's depot hab |
| `plank.png`, `plaster.png`, `thatch.png` | `assets/textures/settlement/` | Not used since the 2026-11-14 rework (kept byte-identical) |
| `atlas.png` | `assets/tilesets/atlas.png` | Every hex column (fields and mountains sheets on Tenebris, lunar regolith on Lantern) and the hotbar's block icons |
| `pickaxe.png`, `shovel.png` | `assets/items/tools/` | Hotbar |
| `torch.png`, `lantern_post.png`, `lantern_wall.png`, `lantern_hanging.png`, `brazier.png`, `candle.png` | `assets/items/lights/` | Hotbar |

M21's rocket part textures and icons are copied byte-identical into `../rocket/` from
`docs/mockups/ship-tiers/` (drawn by `tools/art/ship_tiers_art.py`). The travel icons under `docs/mockups/travel/` are mockup-only pixel art drawn by
`tools/art/travel_art.py`; the globes' maps come from `docs/mockups/planets/`.

`*.gltf.json` beside each `.glb` is the same model repackaged losslessly as glTF
JSON with its binary buffer embedded as base64 (the artifact host does not serve
`.glb`); the page loads that. The `.glb` stays here byte-identical to the source.
