# Proposal: the player builds (step 3 of the cities plan)

## Why

**The owner (2026-09-27): "step 3 implement ways for me to make cities. as a
player I want to be able to put together these buildings myself."**

`tenebris-towns` asked this as its fourth open question: "Should the player be
able to build with these pieces?" This request answers it: yes.

By the time this change starts, `cities-in-the-world` will have built the
pieces into the engine:
- walls on cell edges;
- floors in cells;
- three stairs that land on layer lines;
- roofs, doors, windows, furniture, decks and walkways;
- kits;
- the layout checks that keep a town walkable.

Today a player can only place whole blocks: a right-click puts the held
material into the aimed cell. A block is 2.833 m across. A house made of
blocks has walls as thick as a room, which is the problem `tenebris-towns`
started from. A player who wants to build a town needs the town's own pieces.

## What Changes

- **A build mockup first.** `docs/mockups/build.html`, grown from the towns
  mockup's code, lets the owner try building before any Rust:
  - the palette;
  - the snapping;
  - the checks with their reasons;
  - blueprints and the roof tool.

  The owner approves the feel there.
- **A build mode.** On foot, B opens it and closes it (B is the ship's brake
  and does nothing on foot today). The hotbar becomes a palette of pieces for
  the chosen kit:
  - walls, with door and window openings;
  - floors;
  - the three stairs;
  - posts;
  - roofs;
  - decks and walkways;
  - doors;
  - furniture;
  - `lamps-and-lanterns`' lights.
- **Pieces snap to the cut.** A wall goes on a cell edge and a floor in a
  cell, at a layer line. A storey is three layers. A stair takes its own
  cells and lands on a layer line. The cut is the town's, so a player's house
  and a town's house are the same kind of thing.
- **A placement is checked by the town's own rules.** Before it is placed, a
  piece's ghost is shown green or red. Red comes with the reason:
  - headroom at every walkable point;
  - roof overlap;
  - a stair's landing;
  - a pentagon;
  - furniture in a wall;
  - two faces in one plane.

  One implementation of each rule, in `pbd-core`, serves the town layouts and
  the player alike.
- **Roofs are fitted, not placed tile by tile.** A roof tool picks an enclosed
  footprint and fits the kit's roof over it, as the town layouts do.
- **Blueprints.** Any building from the settlement templates can be placed as
  a ghost and filled in, one piece at a time or all at once.
- **Every placement and removal is a world edit**, through the durable path,
  like mining. A player's building survives a reload, and a future server can
  check it with the same rule.
- **Free building first.** The pieces are made of timber, thatch, brick, slate
  and so on, and the game has none of those as items: the axe fells nothing
  that drops wood. The first version builds for free. What building should
  cost, and where the materials come from, is a question for the owner
  (design, Open Questions).

## Capabilities

### New Capabilities
- `player/building`: the build mode, the palette, snapping to the cut, checked
  placement and its reasons, the roof tool, blueprints, removing what you
  built, and building as world edits.

### Modified Capabilities
- None in the main specs. `world/settlements`, introduced by `tenebris-towns`,
  gains nothing: the player's pieces are the same pieces, held to the same
  requirements.

## Impact

- **`pbd-core`:**
  - `settlement`'s layout checks are exposed per piece as well as per layout;
  - a `building` module for placement, removal and the roof fit;
  - piece edits in the edit log, beside cell edits.
- **`pbd-app`:**
  - the build mode's input on B;
  - the ghost preview and its reason text;
  - the palette on the hotbar;
  - the blueprint picker.
- **Saves:** the edit log gains piece edits, versioned like cell edits.
- **Docs:** `docs/mockups/build.html`.
- **Performance:** a placement check covers the pieces within a few cells of
  the ghost, run each frame while the ghost moves. It is small, but it cannot
  be measured in a cloud session (CLAUDE.md).
