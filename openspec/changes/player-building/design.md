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
