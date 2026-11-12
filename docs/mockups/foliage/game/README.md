# The game's own textures, copied for the foliage mockup

These files are byte-identical copies of textures the game ships. They are here
only so the published `docs/mockups/foliage.html` artifact can load them by a
relative path. Change the originals, never these, and copy again.

| File | Source |
| --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png`: the 14 biome tilesets baked by `tools/build_tileset_atlas.py`, sheet order = sorted file names (`planet_terrain.rs` `TILESETS`) |
| `needles.png` | `assets/textures/settlement/needles.png` |
| `leaves.png` | `assets/textures/settlement/leaves.png` |
| `sage.png` | `assets/textures/settlement/sage.png` |
| `reed.png` | `assets/textures/settlement/reed.png` |
| `litter.png` | `assets/textures/settlement/litter.png` |
| `iron.png` | `assets/textures/settlement/iron.png` |
| `timber.png` | `assets/textures/settlement/timber.png` |

The mockup samples the atlas the way `planet_surface.wgsl`'s `pixel_tile` does:
the sheet by slot, the tile by its (column, row) in the sheet's 4 x 4 grid, 32
texels a tile, nearest.
