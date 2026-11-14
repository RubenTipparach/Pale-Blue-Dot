# The game's own art, as the seas, rivers and marshes mockup uses it

Byte-identical copies, so the published mockup (`docs/mockups/ocean.html`) can
load them; nothing here is edited. The originals are the source of truth: if one
changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | Every ground, seabed, ice and mud cell. Sheets by slot (`planet_terrain.rs` `TILESETS`): ocean 9 (sand, rubble, coral, seagrass, cobble tiles on the reef, kelp shelf and coast), beach 2 (sand and pebbles), frozen_wastes 5 (snow, ice, ice cobble for floes, the berg and the shelf), swamp 12 (moss, mud, mudflats), basalt_wastes 1 (the vent floor), fields 4 (river and dam banks), mountains 8 (headland, boulders, trench walls). A column wears its sheet's (0,0) ground on top unless the scene names another tile, (1,0) on its first metre, (2,0) earth for three more and (3,0) stone below, as `planet_surface.wgsl` picks them |
| `stone.png` | `assets/textures/settlement/stone.png` | The lighthouse, the mill (repeat_m 2) |
| `slate.png` | `assets/textures/settlement/slate.png` | Roofs of the hut, mill, cabin and lighthouse cap (repeat_m 2) |
| `timber.png` | `assets/textures/settlement/timber.png` | Masts, stilts, the cabin, the wreck's deckhouse and chest (repeat_m 1) |
| `plank.png` | `assets/textures/settlement/plank.png` | The sloop's cells, the polar ship and camp, the player's jetty, the wreck's hull (repeat_m 2) |
| `iron.png` | `assets/textures/settlement/iron.png` | Lantern posts, the lighthouse lantern, the chest's band (repeat_m 1) |
| `linen.png` | `assets/textures/settlement/linen.png` | The sloop's sail, the camp's roof (repeat_m 1) |
| `driftwood.png` | `assets/textures/settlement/driftwood.png` | The fisher's hut, the wreck's broken cells, ribs and mast (repeat_m 2) |
| `basalt.png` | `assets/textures/settlement/basalt.png` | Black smoker chimneys (repeat_m 2) |
| `ice.png`, `snow.png` | `assets/textures/settlement/` | Loaded with the common materials (repeat_m 2) |
| `peat.png` | `assets/textures/settlement/peat.png` | Cut peat cells in the marsh (repeat_m 2) |
| `granite.png`, `rockface.png` | `assets/textures/settlement/` | Loaded with the common materials (repeat_m 2) |
| `leaves.png`, `bark.png` | `assets/textures/settlement/` | Trees as the game builds them, hex-prism trunk and leaf prisms; stumps; mangrove canopies (repeat_m 2 and 1) |
| `fish-minnow.png`, `fish-silverfin.png`, `fish-perch.png`, `fish-ray.png`, `fish-eel.png`, `fish-reef.png`, `fish-deepback.png`, `fish-serpent.png` | `assets/items/fish/<name>.png` | The eight species that swim in the game today, as pixel cards in the scenes and on their bestiary cards (provenance in `assets/items/fish/PROVENANCE.md`) |

Sampling is nearest with no mipmaps, as the game's is.
