# The game's own art, as the construction mockup uses it

Byte-identical copies, so the published mockup (`construction.html`, M22) can load them; nothing
here is edited. The originals are the source of truth: if one changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | The ground round the yard: hex columns topped with the fields sheet (slot 4 in `planet_terrain.rs` `TILESETS`), ground tile (0,0) on top, its side tile (1,0) on the first metre, earth (2,0) below, as `planet_surface.wgsl` picks them; the old and printed sites' plots are topped with the crumbly dirt tile (2,0) |
| `flag.png` | `assets/textures/settlement/flag.png` | The yard's apron and the bay's floor (repeat_m 2.5) |
| `rubble.png` | `assets/textures/settlement/rubble.png` | The new site's gravel pad (repeat_m 2) |
| `stone.png` | `assets/textures/settlement/stone.png` | The old house's ground storey (the towns' half-timbered kit), the ashlar blocks, the tower crane's footing of four cells (repeat_m 2) |
| `halftimber.png` | `assets/textures/settlement/halftimber.png` | The old house's upper-storey panels, mapped once per face as the towns mockup maps it (repeat_m 0) |
| `timber.png` | `assets/textures/settlement/timber.png` | The old house's frame and trusses, the scaffold's poles, ledgers, putlogs and ladders, the treadwheel crane, the beams it lifts, lantern posts (repeat_m 1) |
| `plank.png` | `assets/textures/settlement/plank.png` | The old house's upper floor, the scaffold's decks, the treadwheel's treads, cradles and dunnage (repeat_m 2) |
| `rope.png` | `assets/textures/settlement/rope.png` | The scaffold's lashings, the treadwheel crane's rope and slings (repeat_m 1) |
| `thatch.png` | `assets/textures/settlement/thatch.png` | The thatch bundles waiting for the old roof (repeat_m 2) |
| `brick.png` | `assets/textures/settlement/brick.png` | The assembly bay's walls (repeat_m 2) |
| `slate.png` | `assets/textures/settlement/slate.png` | The bay's and the office's roofs (repeat_m 2) |
| `boards.png` | `assets/textures/settlement/boards.png` | The site office's walls (repeat_m 2) |
| `iron.png` | `assets/textures/settlement/iron.png` | Rails, floodlight masts, the brazier, the parts bed's plate, the hopper's legs (repeat_m 1) |
| `hand.png` | `assets/items/tools/hand.png` | The hotbar's first slot, as the game draws it |
| `brazier.png` | `assets/items/lights/brazier.png` | A hotbar slot |
| `lantern_post.png` | `assets/items/lights/lantern_post.png` | A hotbar slot |

Sampling is nearest with no mipmaps, as the game's is. The machine and item icons and the
printstone, precast panel, concrete, glass, polymer, crane paint, lattice, crate, container,
counterweight, hazard and plate tiles (`../*.png`) have no counterpart in the game and are drawn by
`tools/art/construction_art.py` with the balatro-pixel-art kit. The item icons in the assembly bay's
recipes (steel, glass, log, rope, circuit and the rest) are the crafting mockup's
(`../../crafting/*.png`, `tools/art/crafting_art.py`), mockup-only too. The worker in the treadwheel
is the towns mockup's person (`towns.html`, `person()`), flat-coloured as it is there.
