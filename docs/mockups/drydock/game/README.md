# The game's own art, as the drydock mockup uses it

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again.

| File | Source | Used for |
| --- | --- | --- |
| `atlas.png` | `assets/tilesets/atlas.png` | The ground: hex columns topped from the beach sheet (slot 2 in `planet_terrain.rs` `TILESETS`) along the shore and on the sea floor, and the fields sheet (slot 4) inland, ground tile (0,0) on top, its side tile (1,0) on the first metre, earth (2,0) below, as `planet_surface.wgsl` picks them; the island in "Walk the deck" the same way |
| `stone.png` | `assets/textures/settlement/stone.png` | The dock's floor and gate sill, the pump house walls, the lighthouse (repeat_m 2) |
| `granite.png` | `assets/textures/settlement/granite.png` | The dock's altars and copings, and the quays (repeat_m 2) |
| `timber.png` | `assets/textures/settlement/timber.png` | Keels, bulkheads, masts, yards, rails, the helm, the capstan, the keel blocks, the staging poles, the jetty's piles (repeat_m 1) |
| `plank.png` | `assets/textures/settlement/plank.png` | Decks, the inside of the hull, the staging boards, the jetty (repeat_m 2) |
| `fieldstone.png` | `assets/textures/settlement/fieldstone.png` | Ballast (repeat_m 2) |
| `iron.png` | `assets/textures/settlement/iron.png` | The caisson gate, the crane rails and trolley, the pump pipes, lamp posts and lantern frames (repeat_m 1) |
| `slate.png` | `assets/textures/settlement/slate.png` | The pump house roof (repeat_m 2) |
| `tern.glb`, `loon.glb` | `assets/models/vehicles/<craft>/<craft>.glb` (atlas embedded) | The game's keelboat and canoe, moored at the jetty as a scale and style reference |
| `hand.png` | `assets/items/tools/hand.png` | The hotbar's first slot, as the game draws it |
| `lantern_post.png` | `assets/items/lights/lantern_post.png` | The hotbar's second slot |

Sampling is nearest with no mipmaps, as the game's is. The block icons and the
four mockup-only textures (`../*.png`: tarred hull strakes, sail canvas, the
gantry's painted lattice, a deckhouse wall with a window) have no counterpart in
the game and are drawn by `tools/art/drydock_art.py` with the balatro-pixel-art
kit.

The published artifact cannot serve `.glb`, so it carries each model as
`<craft>.glb.txt`, the same bytes in base64; the page loads the `.glb` where it
is served (a local checkout) and falls back to the text copy.
