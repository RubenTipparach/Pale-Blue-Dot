# Design: the player builds

## Context

See `proposal.md` for why. What the design has to work with, on `main` and
after the changes this one follows (2026-09-27):

- **Placing today.** A right-click puts the held block into the aimed cell,
  as a cell edit in the durable log. The ten slots hold blocks, fish and tool
  items, with the tool in its own slot (G to pick).
- **Keys.**
  - On foot, B does nothing; in a ship, B is the brake.
  - G is the tool picker, J the field guide, and M the map after
    `world-map`.
- **Materials.**
  - The terrain's materials are stone, soil, grass, sand, dry and jungle
    grass, snow, rock, dirt, ore and the torch; `lamps-and-lanterns` adds the
    lights.
  - The axe "chops wood", but no tree drops an item.
  - There is no timber, thatch, brick, slate or clay as an item.
- **After `cities-in-the-world`, the engine has:**
  - the pieces (walls on edges with openings, posts, floors, the three
    stairs, roofs fitted over a footprint, decks, walkways, outdoor stairs,
    doors, furniture);
  - thin-solid contact;
  - kits as data;
  - the layout checks (headroom at every walkable point, roof overlap,
    landings, pentagons, furniture clearance, coplanar faces);
  - settlements as packed piece arrays in active chunk entities;
  - doors as world edits.
- **The mockup.** `docs/mockups/towns.html` builds every piece in three.js and
  runs every layout check in the page. A build mockup can reuse it directly.

## Goals / Non-Goals

**Goals:**
- A player can build anything a town has, with the town's pieces, and it
  walks, collides and lights exactly as a town does.
- A player cannot build something the walker cannot use: a stair with no
  headroom, a roof through a roof.
- What is built is kept at once, and one rule decides placement for single
  player and a future server.

**Non-Goals:**
- **The materials economy:** felling trees for timber, cutting thatch,
  firing brick, quarrying slate. The first version is free building. What
  building costs is an owner decision (Open Questions) and a change of its
  own.
- **Placing whole cells differently.** Right-click placing of blocks stays as
  it is. Build mode is a separate mode for pieces.
- **Consequences for taking a town apart.** The owner allows it (survey P2),
  "but people will be upset". Townsfolk reacting to it is a later change. This
  one only allows the removal and records it as the player's edit.
- **Free-form placement** off the cut, at any angle or any height. The cut is
  what makes a building walkable, and it is what the checks are written
  against.
- **Multiplayer permissions** (whose house it is). The placement rule is in
  the core so a server can run it, but ownership is a later change.

## Decisions

**1. The mockup comes first, grown from the towns mockup.**
- `docs/mockups/build.html` takes the towns mockup's pieces, kits, walker and
  checks. It adds build mode, the palette, the ghost, the reason text, the
  roof tool and blueprints, on an empty plot and beside the walled town. It
  is lit at night, per CLAUDE.md.
- The owner builds a house in it and approves the feel before any Rust. The
  key, the palette's layout and whether the ghost follows the aim or a grid
  cursor are settled there.

**2. Aiming picks the slot of the cut nearest the aim, for the selected
piece.**
- The aim ray's hit is resolved to the nearest valid slot for the piece kind:
  - a wall: the nearest edge, at the storey whose floor is at or below the
    hit;
  - a floor: the cell, at the nearest layer line;
  - a stair: the aimed cell and the next cell in the facing direction, from
    this storey's floor to the next;
  - a post: the nearest corner;
  - a door or window: the nearest opening in an existing wall.
- The wheel steps a door or window through the openings, and a stair through
  its three rotations.
- *Alternative:* a free cursor on a grid overlay. Kept as an option for the
  mockup. The aim-snap is the default because it is how blocks are placed
  now.

**3. The checks are one set of functions, called per piece or per layout.**
- `settlement::check(layout)` from `tenebris-towns` is split into per-rule
  functions. Each takes the pieces within its reach of a candidate.
- The layout check and the placement check both call them. The ghost's
  reason is the first rule that fails, named as the town's requirement names
  it.
- A test builds every template twice, once whole and once piece by piece
  through the placement check. Both are accepted, which proves the two
  callers agree.

**4. Piece edits join the journal, authored by the player.**
- A piece edit is either add (kind, kit, cell or edge, layer range, rotation)
  or remove (piece id). A piece id is its slot in the cut plus its kind, so
  the same place and kind is the same id. Like cell edits, it enters the
  durable path at once, as an entry authored by the player
  (`world-persistence`). So a later world process, such as a town growing,
  yields to it.
- Removal is refused when another piece depends on the removed one: a table
  on a floor, a roof on a wall, a door in a wall. The refusal names the
  dependant.
- *Alternative:* store player buildings as their own settlements.
  Rejected: an edit log of pieces is the same mechanism doors use, and it
  replays in order like the terrain's.

**5. Blueprints are the templates' buildings, one at a time.**
- Any building in `assets/settlements/` can be chosen and rotated by
  multiples of 60°, checked as a whole ghost, then filled:
  - all at once, in free building;
  - piece by piece otherwise, with the ghost showing what is left.
- Each piece placed from a blueprint is an ordinary piece edit. A blueprint
  is not saved as a unit, so removing half a house leaves half a house.

**6. The roof tool fits over an enclosed footprint.**
- The player aims at a room. The tool flood-fills its cells, bounded by walls
  and at the same floor, and fits the kit's roof: a gable over the rows'
  bounding box, a cone over one cell, or a flat roof with a parapet.
- It uses the same fitting function the templates use, and is checked for
  overlap.

**7. Free building is a world setting, not a cheat.**
- A world is made with building free or costed. Only free exists until the
  materials change lands. The setting is saved with the world, so a costed
  world cannot be turned free by a config edit.

**8. A floor over lower ground stands on a footing (2026-11-22).** The
first captures in the game showed a blueprint's floor slab hanging a layer
or two over the desert's lower terraces with nothing under it. Every floor
piece with no floor of the plot under it in the same cell, whose cell's
ground is below it, is cut with a footing: a prism of the cell's own outline
in fieldstone (`fieldstone`, the stone kit's wall) from the ground's top up
to the slab's underside. It is solid to the walker. The ground is the
generated ground (`planet::surface_height`, towns' ground included), read
by the plot's cut; a dig under a house after it is built does not move it.
Fieldstone for every kit, recommendation taken (ask only with screenshots):
one look for a building's foot whatever its walls are. Posts were the other
choice; a footing closes the gap a post leaves for the eye to look through.

**9. A player's building casts the sun's shadow.** Each cut plot's
triangles (not its lanterns' flames, not cut-out textures) are put in the
sun's caster list as a town's are (`planet_shadow::TownCasters`, chunked by
`towns::casting_of`), under an id of the plot's own past every site id, and
replaced whenever the plots are cut again. So a house darkens the ground and
the rooms under it as a town's house does.

## Risks / Trade-offs

- [The per-frame placement check is slow in a crowded town] → It only reads
  pieces within reach of the ghost, which is a few cells. It is timed in a
  core test against the walled town's densest street.
- [Players build things the checks did not foresee, like a stair inside a
  stair] → Every check is run on every placement. A layout the walker cannot
  use is refused, even if no rule names its exact shape, when the walkable
  surface's headroom check finds it. New shapes found in play become tests.
- [Free building makes towns trivial to copy] → That is the point of the first
  version. The costs are the owner's decision.
- [B on foot and B in a ship do different things] → The controls list names
  both. The mockup tests whether that confuses. If it does, build mode moves
  to another free key (I, K, N, O, U or Y).

## Migration Plan

- The edit log's format gains piece edits, with the version bumped. An old
  save has none and reads unchanged.
- Rollback is the previous build. Piece edits are ignored, so the buildings
  vanish, and the terrain and cell edits are untouched. The format comment
  says so.

## Decided by the owner (survey, 2026-09-27)

- **P1, "lets save this for after all the other work is done":** building is
  free. What it costs is decided after the rest of the roadmap.
- **P2, "yes, but people will be upset":** a player may take a town's
  buildings apart. Townsfolk reacting to it is later work (the owner, T5:
  "you can steal from people lol, but there will be consequences"). Until then
  the removal is allowed and recorded as a player edit, so a later change can
  read it.
- **P3, "lets save this for after all the other work is done":** build mode is
  on B provisionally. The key is settled at the end.

## As built in phase 3 (2026-11-21)

What `cities-in-the-world` was expected to leave behind, and what reading
the code found before anything was built:

- **There is no `settlement::check`.** The towns' layout rules are held by
  tests (`settlement::tests`: `no_two_roofs_cut_into_each_other`,
  `every_stair_is_cut_and_nothing_floors_its_well`, ...) and by how the
  cutter lays a template out, not by a function a placement could call. So
  task 2.1 is not a split of an existing function: the rule set is written
  once, in `pbd_core::building::rules`, and the town side reaches it by
  turning a template's building into pieces (`building::blueprint`) and
  checking them (`building::check_layout`). The test that every plain
  template building is accepted whole and piece by piece is the proof the
  two callers agree.
- **Towns are cut whole, not as pieces.** A town's building is one
  `BuildingDef`, cut into meshes and solids by `settlement::pieces`. A
  player's building is a set of pieces, each its own record, and is cut by
  `building::cut` into the same `BuildingSolids` and `Meshes` types, so the
  walker and the field light read it as they read a town.

### B1. Pieces sit on the planet's own cells

A piece names real finest cells by their exact key (`cell_key`), never a
chart's offset coordinates:

- a **wall**, **door** or **window** on the edge between two cells, the
  pair of keys sorted (shared edge ownership: one edge seen from either side
  is one slot), standing one storey (3 layers) from its base layer; a door
  is a wall with a doorway (no leaf yet), a window a wall with a window;
- a **floor** in a cell, on a layer line;
- a **stair** from a foot cell to the head cell across one of the foot's
  sides, rising one storey;
- a **post** at a corner, the three cells that meet there, sorted;
- a **lantern** in a cell, on a layer line;
- a **roof** over a set of cells, its eaves on a layer line.

A layer is the column's own index (`column::layer_altitude`), counted from
`column::BASE_M`, which never moves, as a cell edit's layer is. Pentagons
are real: a cell with five sides has five neighbours, and the pentagon rule
refuses it and its neighbours.

### B2. The rules: one function each, the first to fail is the reason

`building::rules`, in M6's order and with M6's reasons:

| Rule | Refused when |
| --- | --- |
| `slot` | the slot already holds a piece of the same class |
| `pentagon` | a cell of it, or a neighbour of one, is a pentagon |
| `held_up` | an upper floor, wall, post, stair or lantern has nothing under it: a wall or post on its cell below, a floor beside it on its storey, or the ground storey |
| `headroom` | a floor in a stair's well (its two cells, the storey above its foot), or a stair whose well holds a floor |
| `stair_cut` | a wall would stand across a stair's flight |
| `landing` | the stair's way off its head is walled at the top |
| `room` | a roof's cells are not closed round by walls, or part of them has no floor |
| `roof_overlap` | a roof would take another roof's cells, eaves included |
| `opening` | a lantern on a stair |

A refusal is a `Refusal { rule }`; the ghost shows the rule's locale
string, `build.refused.<rule>`, in M6's plain words.

**Removal** is refused with the dependant named (`building::dependant`):
a lantern on a floor, a roof on a wall or a floor, a wall upstairs on a
wall below, a stair on its floor.

### B3. Records, not a new log line: the format question

Each piece is a record of kind `piece` (schema 1). Its id is the hash of
its slot and class (decision 4: "the same place and kind is the same id"),
its body the piece and its kit, or `None` once removed. It enters the
journal at once through `WorldSave::store`, as a `rec` line authored by
the player, and the first piece notes the kind in the world's identity
(`note_record_kinds`): that note is the version bump. **The edit log's line
format is not changed.** That was chosen over a new line shape with the
log's own format number bumped:

- the `rec` line already exists, is versioned per kind, and is kept as
  text by a build that does not know the kind, so an older build opening a
  newer world loses nothing (it does not draw the houses and keeps their
  lines);
- an old save has no `piece` records and opens unchanged: nothing has to
  migrate, and the format tests show an old log still reads;
- three other phase-3 agents (townsfolk, trade, roads) edit the save in
  parallel, and a contended format number buys nothing the record line
  does not already give.

Building is free (decision 7), so a piece line carries no slots.

### B4. Cutting

The pieces within `PLOT_M` (24 m) of a plot's anchor cell are cut in one
frame, the tangent plane at that anchor, as a town building is, into
`Meshes` in the kit's own textures (the wall's outside and inside, the
floor, the roof material, the gable) and one `BuildingSolids`: walls as
thin solids with their door and window gaps, floors as surfaces, stairs as
flights, roofs as their plan. The anchor is the lowest cell key among a
plot's pieces, so the same pieces always cut the same way.

### B5. The game side

- **N** on foot opens and closes build mode (moved to B on 2026-11-22, B10). The design named B, which
  was free on foot when it was written; it is the breeding screen's key
  now, so build mode takes N from the risk note's own list of free keys
  (I, K, N, O, U, Y; recommendation taken, ask only with screenshots). The
  hotbar row is hidden and the palette row shown in its place: wall, door,
  window, floor, roof, stairs, post, lantern, blueprint, remove, with M6's
  icons (copied into `assets/textures/ui/build/`, provenance in its
  README). The player's slots are never touched and the selection is given
  back on leaving, so the hotbar is as it was. While build mode is open its
  clicks, the number row and K are taken from every other reader before
  `Update`, so a click does not also dig, cast or eat.
- The number row picks a palette slot; the wheel turns a stair, a blueprint
  or the blueprint's choice. Left click places; right click removes the
  aimed piece (the Remove slot does the same on a left click).
- The ghost is the piece's own cut, drawn translucent green or red; a red
  one carries its reason on the line under the crosshair. A ghost is a
  preview, not a thing placed in the world, so it is not lit by the field.
- **K** in build mode opens the kit picker: every kit side by side, with
  its wall, floor and roof textures; the wheel steps the chosen kit, K
  closes it (the walker keeps the pointer, so the picker needs no cursor).
- The roof tool is the Roof slot: aimed at a room, it flood-fills the cells
  bounded by walls on that storey and fits the kit's roof over them: a
  cone over one cell, the kit's flat roof, or a gable along the room's
  long axis.
- A blueprint is any plain building of the templates (no newel, stilts,
  raised floor, parapet or dome), charted onto the cells at the aim and
  turned by the wheel in sixths (shift and the wheel steps which); one click fills it whole (free building,
  decision 7). Each of its pieces is an ordinary piece record, so half a
  blueprint removed is half a house.

### B6. Not built in this pass

- The build camera (task 4.5) as first planned: double-click to walk and
  the cutaway. The orbit, pan, zoom and a build layer are built as
  architecture mode (B9); double-click to walk and the cutaway came after
  (B11, 2026-11-24).
- Filling a blueprint piece by piece in the game (the core test does it;
  the game fills it whole, as free building allows).
- Building costs (decision 7: free only).
- Frame cost: not measured in the cloud (lavapipe); `can_place` is timed in
  a core test instead (task 5.1).

## The owner's notes on phase 3 (2026-11-22)

The owner, on the survey's known issues for this step, then "continue
build":

- "The stone footing has no texture yet": "add one" (B7);
- "No build camera; doors are open doorways": "need to craft door, add
  crafting camera \"architecture mode\"" (B8, B9);
- "Build mode is on N (B opens breeding)": "fix it" (B10).

### B7. The footing's own texture

The footing (design 8) was cut in `fieldstone`, the fieldstone kit's wall:
loose round field stones in a dark mortar, drawn for a wall seen from a
lane. Under a floor, a storey's height of it reads as a grey smear, which
is what the owner saw. It gets a texture of its own, `footing`:

- 32 x 32 in `assets/textures/settlement/`, as every town texture is, with
  its row in that folder's `manifest.ron` (`repeat_m` 2.0, so four courses
  stand in a metre and a half);
- dressed blocks laid in courses, each course's joints staggered from the
  one under it, the blocks a little uneven in shade and lit from above, the
  mortar recessed and darker;
- every colour taken from the stone the town kits already use
  (`fieldstone`, `stone` and `rubble`), so it sits in their palette and
  in the ground's; no outline and no contrast stronger than theirs;
- drawn in code by `tools/art/footing_texture.py`, deterministic, with
  `--check`; a test holds every colour to the three stones' palette, and the
  manifest and the PNG together;
- sampled nearest, as every town texture is (`towns::texture`).

`building::cut::FOOTING` names it. Only the footing changes; the stone kit's
walls keep `fieldstone`.

### B8. Doors: crafted, hung in a doorway, opened and shut

**The item.** A door is a good, `door` (a stack of 8), made at the
workbench from **4 planks and 1 iron ingot** in 4 s: the leaf is planks, the
fitting (two strap hinges and a pull) is iron. Iron over bronze because the
forge's first smelt of mixed ore gives iron directly, where bronze takes a
second step (copper and tin); a door should come before a bronze pickaxe.
Recommendation taken (ask only with screenshots). Its icon is drawn by
`tools/gen_item_icons.py`, an inventory icon with the 1 px outline.

**The piece.** `Shape::Door { a, b, layer }`, class `Door`, on the same
edge and layer as a wall, in its own slot (`door:a:b:layer`), so a wall and
its door are two records. A wall with a doorway (`Opening::Door`) is still a
doorway: open, with no leaf.

- **Rule `doorway`:** a door goes only where a wall with a doorway stands
  on its edge, at its layer ("Needs a doorway").
- **Removal:** a wall with a door in it is not taken away before the door
  ("Remove the door first").
- **Placing** takes a door from the pack: the piece and the slots go into
  the journal as one line (`WorldSave::store_with_hands`). With none
  carried, nothing is placed and the line under the crosshair says "No door
  in the pack". Taking a door away gives it back.
- **In the palette** the Door slot does both jobs: aimed at an edge with a
  doorway and no door, its ghost is the door; aimed anywhere else, the
  doorway as before.

**Opening and shutting.** F (interact), in reach of the leaf as a town's
door is (`towns::DOOR_REACH_M`, in front of the eye), offers "Door", and
opens or shuts it. The state is the towns' own door record (`door`, schema
1, open or not, no record shut), at the id `building::door_state_id(piece)`:
the piece's id with the top bit set, past every town door's id (a site's
building over its door number never reaches 2^56). The record is written
before the leaf moves; a refused save leaves the door as it was. The leaf
swings a quarter turn at the towns' swing speed. Shut, the leaf is a solid
of the plot (`BuildingSolids::doors`), so the walker cannot pass; open, it
is none. Taking the door away writes it shut, so a door hung there later
starts shut.

**Old houses keep their doorways.** Nothing is migrated: a saved wall with
`Opening::Door` has no `Door` piece, so it is cut as the doorway it was, and
stays open until the player hangs a door in it. Blueprints still fill
doorways, not doors: building is free, doors are crafted.

**The model.** One shaped form, not boxes:

- the **leaf** is one slab cut from a chamfered outline (the corners
  clipped), its faces bevelled into its edges all round, drawn in its own
  texture, `door` (32 x 64): four vertical planks, two ledges and a brace,
  the iron strap hinges on the hinge side and a ring pull on the other,
  painted into the face rather than modelled (CLAUDE.md, "No obvious
  primitives": detail painted into the surface). Its colours are the
  `plank`, `timber` and `iron` textures';
- the **frame** is one U-shaped casing lining the doorway's two jambs and
  its head, cut as a single outline through the wall's thickness, its
  inner edge chamfered, in the kit's wall edge texture so it belongs to the
  wall;
- the leaf hangs on the jamb on the side away from the room's floor (or
  cell `a`'s side where both or neither are floored) and swings into the
  room.

Both are children of the plot's root, so they take the field and the
terrain's sun as the walls do (`LitByField`, `LitLikeTerrain`).

### B9. Architecture mode: the build camera

**Z** in build mode turns architecture mode on and off (B leaves build mode
and architecture mode together). The camera pulls up and back from the
walker's eye, over the build site, and the pointer is freed: the cursor is
the aim.

| Control | In architecture mode |
| --- | --- |
| Z | back to the walker's own view |
| Move keys (W A S D) | pan over the site, along the camera's heading |
| Right drag | orbit: turn round the site and tilt (20 to 85 degrees down) |
| Ctrl + wheel | zoom (6 to 60 m) |
| Wheel, Shift + wheel | as in build mode: turn a piece; the next blueprint |
| Page Up / Page Down | the build layer up or down a storey (three layers) |
| Left click | place the ghost at the cursor |
| Right click (no drag) | remove the piece at the cursor |
| Number row, K | as in build mode |

- **The site** is where the walker's aim was when the mode opened (or 8 m
  ahead of the eye if it was on nothing), and **the build layer** starts as
  the floor layer there. The cursor's ray is marched to the ground or a
  piece as the walker's aim is, but no further than the build layer's
  sphere: a ray that reaches the build layer in open air lands there, so a
  floor or wall can be put on an upper storey by pointing at it.
- **It moves smoothly.** The shown eye and look ease toward the target
  (a time constant of 0.12 s), so entering, panning, orbiting and zooming
  glide. The walker's rule of raw mouse look (CLAUDE.md) is for walking and
  is not this camera's.
- **The walker stands still.** While it is on, the walker reads no keys and
  no mouse, as under a menu (`controls::Pointer`), so its position, heading
  and pitch are exactly as they were.
- **Returning restores the walker's camera exactly.** Leaving the mode
  hands the pointer back (the menu's edge rule) and the walker's camera is
  written from the walker's own pose on the same frame: nothing is eased
  back, and nothing of the build camera is left in it.
- **Positions** are kept as a planet direction and a radius in f64 and put
  in the local frame through the origin (`SimOrigin::at`), so a rebase
  moves nothing (CLAUDE.md, floating origin).

The camera's pose, its easing and the cursor's ray are in
`pbd_app::build_view`; the walker reads only its on flag.

### B11. Double-click to walk, and the cutaway (2026-11-24, task 4.5)

Written before the code. The towns mockup's overview does both
(`walkFrom`, and its "Cutaway height" slider, a clip plane at that height
over the whole scene). In the game they are architecture mode's.

- **Double-click to walk there.** A double left-click in architecture mode
  (two presses within `DOUBLE_CLICK_S`, 0.3 s, and 6 px) on a place a
  walker can stand (the cursor's ray came down onto the ground or a floor,
  under the cutaway when it is on) sets the walker down there, facing the
  way the build camera faces, and leaves architecture mode: the walker's
  camera takes over from the walker's new pose on that frame, as B9's
  leaving does. A ray that met a wall's side, or the build layer in open
  air, walks nowhere. It is the mockup's `walkFrom`. In architecture mode a
  single left click still places the ghost, but only once the double-click
  window has passed with no second press (the click is held 0.3 s), so a
  double-click never also places a piece. Out of architecture mode nothing
  changes.
- **The cutaway.** **End** in architecture mode turns it on and off (End
  sits by Page Up and Page Down, which move the cut, and is read by
  nothing on foot: X, the first choice, is the road planner's menu with
  stakes in hand; `controls` gains the row, its name from the locale
  file). While on, every field-lit thing (the towns,
  the player's plots and sites, props, people) within `CUT_REACH_M` (40 m)
  of the site is cut at **the build layer's floor plus `CUT_OVER_M`
  (1.2 m)**, so a storey's floor, what stands on it and the foot of its
  walls stay in sight, and Page Up and Page Down move the cut a storey with
  the build layer. This is the building planner's isometric cutaway (M6
  version 3: "the back outside walls full height and the near and inner
  walls cut low") made 3D, not a second rule: past the cut, what stands
  *beyond* the site from the camera keeps standing up to the storey's
  ceiling (the floor plus a storey), so the far walls frame the room as
  they do in the planner's view, and everything nearer the camera, and
  everything over the storey (the roof, the storeys above), is cut. The
  rule is one function, `build_view::Cutaway`, which the planner's painter
  asks too (`stands_full`: an outer wall on the far side), and which
  `field_lit.wgsl` mirrors with the same constants, pinned by a test that
  reads the shader.
- **What it does not cut.** The terrain (the planet's surface shader) is
  not cut: a hill is not a storey, and a cut through the planet would show
  its inside. Nor are the ghost and the water. The sun's shadows are cast
  by what is cut as before, so the cut room keeps the light it had.
- **Where it is carried.** The cut is two `vec4`s on the field-lit
  materials' per-frame view (`FieldView::cut`, `cut_far`), zero when off,
  so a frame with the cutaway off draws as it did.
- **Its double-click and its cut are not saved**: the cutaway is off each
  time architecture mode opens.

### B10. Keys: build mode on B, breeding on N

The owner's P3 ("B for now") and "fix it": build mode moves to **B** on
foot. B is the brake in flight and the bail in a canoe, which are never live
with the walker (the bindings' contexts), so nothing clashes.

Breeding needs a new key. Every binding was checked (`controls::BINDINGS`
and every `KeyCode` a system reads): on foot the free letters were E, N, Q
(read by the pack's toss, though not listed), U, X, Y and Z. E is kept free
for the roll in space (owner, 2026-11-16), Q is the toss, and U and Y are
the raft's logs. **N** is taken for breeding: it is the key build mode
gives up, beside B on the keyboard, read by nothing else in any context, so
the set of keys the game reads does not grow. Architecture mode takes Z
(B9) and the build layer Page Up and Page Down, all free on foot.
Recommendations taken (ask only with screenshots).

A player who rebound breeding (`on_foot:KeyB`) or build mode
(`building:KeyN`) keeps that key: the settings file's old slot names are
moved to the new ones (`on_foot:KeyN`, `building:KeyB`) when they are read,
as the interact key's were. Every on-screen key name is printed from the
table, so the controls page, `--help` and the hints follow the table with no
text of their own.
