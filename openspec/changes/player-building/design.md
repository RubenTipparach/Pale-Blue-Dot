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
  is a wall with a doorway and its leaf, a window a wall with a window;
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

- **B** on foot opens and closes build mode (B is still the brake in
  flight and the bail in the Loon). The hotbar row is hidden and the
  palette row shown in its place: wall, door, window, floor, roof, stairs,
  post, lantern, blueprint, remove, with M6's icons (copied into
  `assets/textures/ui/build/`, provenance in its README). The player's
  slots and selection are never touched, so leaving shows them as they
  were.
- The number row picks a palette slot; the wheel turns a stair, a blueprint
  or the blueprint's choice. Left click places; right click removes the
  aimed piece (the Remove slot does the same on a left click).
- The ghost is the piece's own cut, drawn translucent green or red; a red
  one carries its reason on the line under the crosshair. A ghost is a
  preview, not a thing placed in the world, so it is not lit by the field.
- **K** in build mode opens the kit picker: every kit side by side, with
  its wall, floor and roof textures; a click picks the kit.
- The roof tool is the Roof slot: aimed at a room, it flood-fills the cells
  bounded by walls on that storey and fits the kit's roof over them: a
  cone over one cell, the kit's flat roof, or a gable along the room's
  long axis.
- A blueprint is any plain building of the templates (no newel, stilts,
  raised floor, parapet or dome), charted onto the cells at the aim and
  turned by the wheel in sixths; one click fills it whole (free building,
  decision 7). Each of its pieces is an ordinary piece record, so half a
  blueprint removed is half a house.

### B6. Not built in this pass

- The build camera (task 4.5): orbit, pan, zoom, double-click to walk and
  the cutaway. It is a camera of its own beside the walker's and is left
  for a later pass.
- Filling a blueprint piece by piece in the game (the core test does it;
  the game fills it whole, as free building allows).
- Building costs (decision 7: free only).
- Frame cost: not measured in the cloud (lavapipe); `can_place` is timed in
  a core test instead (task 5.1).
