# Proposal: yards that build things too big for a bench

## Why

The owner, reviewing the fabricator mockup (M3, the tracker's `reviews`,
2026-11-12): "Would like to see what more advanced fabricators look like for
building large items, cranes and 3d printers for assembling buildings".

The same day, in the conversation:
- "I need to see crains, 3d printers, shipyards (boats too) to build
  vehicles. I also want more mockups for planes, helicopters, vtols. make air
  vehicles these cute little chibi versions. they require a hangar, and the
  player has a ui similar to your ship builder that lets them put these
  together". Two reference pictures came with it: a chibi fighter with a
  shark mouth and a nose number, and a pair of utility helicopters.
- "yea, we need drydocks to build ships too, so mock that up as well. I would
  prefer ocean ships to be made of hex blocks as well so i, as a player can
  walk around as the ship sails about".

Today the largest thing a player can make is what a fabricator hands over in
one piece (M3). The ship builder (M4) makes spacecraft from 2 m modules on a
square grid in a shipyard. Nothing makes a building, a boat or an aircraft.

What the game already has to build on:
- **Walking on a moving deck is built.** `sail-the-cog` 3.1 and 3.2: the
  walker stands on a craft's deck in the craft's frame
  (`vehicles::tests::a_walker_rides_a_cog_under_way_through_a_turn`).
- **A hull of cells floats** (`pbd_core::vehicle::hull::float`). Its rule is
  that a buoyancy cell is at most a third of the beam.
- **Every lifting surface is one foil model** (`openspec/specs/player/vehicles`).
  That covers the Kestrel tiltrotor's wings and rotors, the Tern's sail and
  keel, and the cog's square sail.
- **The world's cell** is 2.833 m across and 1 m tall on every body
  (CLAUDE.md, "Hex size is fixed").

## What Changes

Three mockups. Each is a page with a 3D yard, lit by day and at night, and a
builder. No game code: that waits for the owner's approval and a request
(CLAUDE.md, "Write it up before touching code").

1. **M22 Cranes and Printers** (`docs/mockups/construction.html`), the
   advanced fabricators of the M3 note:
   - **an assembly bay** (a large fabricator) that makes items too big for a
     pack: a boat's mast, an aircraft's wing, a ship's engine, a building's
     wall panel;
   - **a hex printer**, a gantry over a plot that prints a building one 1 m
     layer at a time, onto the world's own cells;
   - **a tower crane and a gantry crane** that a player drives from a cab, to
     lift what the bay makes into place.
   A slider runs a print from bare plot to roof, and the crane can be swung by
   hand.
2. **M23 Drydock and Hex Ship Builder** (`docs/mockups/drydock.html`):
   - **a drydock**: a basin of hex columns cut below the sea, closed by a
     caisson gate and pumped dry. A ship is built on keel blocks inside it
     under travelling cranes, then floated out;
   - **ocean ships made of the world's hex cells**: 2.833 m across and 1 m
     layers, the same cell as the ground. The player walks the decks while
     the ship sails;
   - **a builder, one deck at a time**: a hex grid per layer from keel to
     castle, with a profile showing the waterline. Every number follows from
     the cells: mass, displacement, draft, freeboard, stability, sail area
     and hull speed. The checks include that the helm can be walked to from
     the gangway.
3. **M24 Hangar and Chibi Aircraft** (`docs/mockups/hangar.html`):
   - **chibi planes, helicopters and VTOLs**: big canopies, stubby wings and
     fat rotors, made of boxes and prisms with pixel textures;
   - **a hangar they need** to be built in and kept in, with an apron, a
     runway and a helipad;
   - **a builder like M4's**: a top-view grid with mirror painting, a live 3D
     preview, a livery (shark mouth, nose number, colours), and stats from
     the parts. The stats are stall speed, top speed, hover margin, balance
     (centre of mass against centre of lift) and range. The checks are the
     ones a pilot would feel: a helicopter without a tail rotor spins, and a
     plane whose centre of mass sits behind its wing is unstable.

## Capabilities

### New Capabilities

- `world/construction`: the assembly bay, the hex printer and the cranes.
- `player/ship-building`: drydocks, and hex-cell ships the player walks.
- `player/aircraft-building`: hangars, the aircraft builder and its parts.

### Modified Capabilities

None until the mockups are approved and a build is asked for.

## Impact

- Three new pages in `docs/mockups/`, each with:
  - its art script in `tools/art/`;
  - the game's own assets copied byte-identical under `<page>/game/`;
  - day and night screenshots in `docs/screenshots/mockup-<page>/`.
- The tracker lists M22 to M24; `docs/mockups/mockup-work.md` follows.
- M3's review note is answered by M22. M4's square module grid stays for
  spacecraft. Ocean ships use hex cells, on the owner's word.
- Saves: none yet. When built, a ship, an aircraft and a printed building are
  stored records and world edits through the durable path. The design says
  how each survives a change to its parts.
