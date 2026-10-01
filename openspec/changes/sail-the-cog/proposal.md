# Proposal: sail the cog

## Why

**The owner (survey T7, 2026-09-27): "Yup, you can use any boat you find."**
Asked whether that includes the cog, the big ship you walk on, where sailing
it means walking on a moving deck (T9): **"YES"**. The recommended timing,
which this change takes, was its own change after the roadmap.

The towns mockup's harbour has a cog, a ship 15 m long and 5 m in the beam
(`docs/mockups/towns.html`, `cog()`). You board it over a gangplank, walk its
deck, climb to its aftcastle, and look up at its furled sail. In the plan it is
a building that never moves:
- `tenebris-towns` 3d builds it as a fixed piece;
- `cities-in-the-world` lists "boats that sail" as a non-goal.

The game can sail small craft, but nobody can stand on anything that moves
(measured on `main`, 2026-09-27):
- **The Tern and the Loon are custom rigid bodies** in `pbd_core::vehicle`,
  not Avian bodies, and have no colliders.
- **The walker stands only on terrain.** `walking.rs` asks `PlanetContact`
  for ground. A walker on a boat's deck falls through it into the sea, and the
  boat sails on without them.
- **Aboard, the walker is parked.** Each tick it is set down at the craft's
  exit point with its body switched off. It is a passenger, not a person
  walking about.
- **Frames only translate.** `pbd_core::frame` says rotating and accelerating
  frames "are intentionally not silently approximated". CLAUDE.md requires
  that station passengers simulate in the station's local frame, but nothing
  implements it yet.

## What Changes

- **The cog becomes a craft.** It is a fourth kind beside the Kestrel, Tern and
  Loon, with the same models they use:
  - a hull of cells that floats on the drawn sea;
  - a square sail on a yard as one foil, trimmed by its braces;
  - a keel and a stern rudder as wet foils.

  Its deck, castles, stair, rail and gangway are the ship piece
  `tenebris-towns` 3d already specifies.
- **You walk on its deck while it sails.** A walker on the deck lives in the
  ship's frame. Standing, walking, the stair and the rail are answered in that
  frame by the same `stand` query the towns use. The frame's motion,
  including its turning, carries the walker with it. Stepping off the rail, or
  jumping from the castle, returns the walker to the planet's frame at the
  composed velocity.
- **The helm.** F at the tiller takes the helm and steers and trims the sail.
  F again lets go, and the ship sails on under the unattended policy while you
  walk about. Anyone else aboard walks freely.
- **A player who quits on the deck loads on the deck.** Today a player who
  quits aboard is loaded on foot beside the craft, which for a ship is in the
  sea. The walker's place on the deck is saved in the ship's frame.
- **Frames can turn.** `pbd_core::frame` gains orientation and angular
  velocity, composed with the angular term the frames spec already requires.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `player/vehicles`: the cog, a craft you walk on; a walker on a deck is
  saved on it.
- `player/walking`: standing and walking on a moving deck.
- `world/frames`: a local frame that turns. Built and moved into
  `openspec/specs/world/frames` with its tests (task 1.1, 2026-10-01).

## Impact

- **`pbd-core`:**
  - `frame.rs`: a turning local frame;
  - `vehicle`: `Kind::Cog`, its hull, square sail and spec, in
    `assets/config/vehicles.ron`;
  - the record's version goes up for the cog and the walker's deck position.
- **`pbd-app`:**
  - `walking.rs`: ground and walls from a deck in its craft's frame;
  - `vehicles.rs`: the helm, and aboard without being parked;
  - saves: the vehicles file with the deck position.
- **Depends on:**
  - `tenebris-towns` 1 and 3d: thin solids and surfaces in `stand`, and the
    ship piece;
  - the built `vehicles` change.

  It comes after the roadmap, at the owner's word.
- **Out of scope:**
  - other players aboard (multiplayer seats);
  - cannon, cargo and damage;
  - a crew of townsfolk working the ship;
  - landing a flying craft on a moving deck.
