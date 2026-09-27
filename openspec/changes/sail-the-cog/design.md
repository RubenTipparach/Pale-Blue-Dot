# Design: sail the cog

## Context

See `proposal.md` for the owner's words. What the design has to work with,
observed on `main` (2026-09-27):

- **Craft** (`crates/pbd-core/src/vehicle/`):
  - `Craft` carries a `RigidBody` (`body.rs`), a custom semi-implicit
    integrator stepped four times a tick, because stiff buoyancy needs every
    force re-evaluated per substep.
  - `hull.rs` floats a hull of cells against the sampled sea: buoyancy,
    heave damping, resistance and bilge water.
  - `foil.rs` is one model for sail, keel, rudder and wing.
  - The Tern's boom swings free with the apparent wind until the sheet stops
    it (`tern.rs`).
  - Poses are `f64` in the planet's frame.
- **Aboard** (`crates/pbd-app/src/vehicles.rs`, `walking.rs`):
  - one `Aboard` seat;
  - F boards and leaves;
  - the walker's Avian body and collider are disabled while aboard, and it is
    set at the exit point each tick.
- **Walking.** The walker is an Avian capsule whose ground comes from
  `PlanetContact::stand` in `resolve_ground`. Nothing moving is ever ground.
- **Frames** (`pbd_core::frame`): `LocalFrame` translates only. `world/frames`
  requires composed motion to include the angular term.
- **Records** (`vehicle/record.rs`): `VehicleRecord` holds the ID, kind,
  frame (only `planet:0`), pose, velocities, bilge, mooring and owner, at
  `RECORD_VERSION` 1. Occupancy is not saved.
- **The cog, as designed** (`tenebris-towns` design 10, the harbour):
  - 15 m by 5 m, the deck 1.9 m over the water, as a step solid;
  - a rail with a gangway;
  - castles 1.6 m over the deck, and an eight-riser stair to the aftcastle.

## Goals / Non-Goals

**Goals:**
- Sail the cog from its helm, with the sail and rudder acting as the Tern's
  do.
- Walk anywhere on its deck and castles while it sails, heels and turns, with
  no slide, jitter or fall.
- Step off it and swim, jump off the castle, and board it from a pier or out
  of the water.
- Quit on its deck and load on its deck.

**Non-Goals:**
- **Fictitious forces on the walker.** A walker on the deck does not feel the
  ship's acceleration or its turn as a push. The deck carries it, as a floor
  in a game does. A heel is a slope underfoot, nothing more.
- **Other players aboard, a townsfolk crew, cannon, cargo and damage.**
- **A mesh collider for the hull.** The walker meets the deck's pieces, not
  the hull's skin.

## Decisions

**1. The cog is a craft of the kind the Tern is, scaled.**
- `Kind::Cog`, with a hull spec, a sail spec and a seat spec in
  `vehicles.ron`, validated like the others.
- **The square sail** is one foil on a yard. The yard turns about the mast
  with the braces: W and S brace it round, A and D steer the tiller. The
  foil's centre of effort is at the yard.
- **The keel and a stern rudder** are wet foils sampled at their own points.
- **The hull** is cells from the same shape function the mockup lofts, at a
  cell size within `hull.rs`' limit (`beam / 3`).
- Everything is tuned on a scripted run: a reach, a beat and a run, with
  speed and heel logged. The cog is slower and steadier than the Tern.

**2. A walker aboard lives in the ship's frame.** This is CLAUDE.md's passenger
rule, built for the first time.
- `LocalFrame` gains an orientation and an angular velocity. `to_global` and
  the velocity composition add the `ω × r` term, which `world/frames` already
  names.
- The walker keeps a local pose (position, yaw, vertical velocity) in the
  ship's frame. Its moves, steps and falls are resolved against the deck in
  that frame, then composed to the planet's frame for drawing and the
  camera.
- Mouse look stays raw (CLAUDE.md). The view turns with the ship because the
  frame it is in turns. It is not eased toward anything.
- *Alternative:* carry the walker by adding the deck's velocity each tick in
  the planet's frame. Rejected: at 15 m from the pivot, a turn's velocity
  differs across the deck, and the walker slides and jitters on every turn.

**3. The deck is the ship piece, asked in its own frame.**
- The deck, castles, stair, rail and gangway are `tenebris-towns` 3d's ship
  piece: thin solids and surfaces. It is built once, in the ship's frame.
- The walker's ground query asks the deck of any craft in reach before the
  terrain. It transforms the query point into the craft's frame and back. The
  contact index is the same one a town uses.
- A walker is aboard while the deck is its floor. Leaving the deck (off the
  rail, down the gangway, off the castle) hands it back to the planet's
  frame at its composed velocity. Climbing onto the deck from the water or a
  pier hands it in.

**4. The helm is a seat you can leave while the ship sails.**
- F within reach of the tiller takes the helm. The walker is parked at the
  helm as today's seat parks it, and the vehicle camera is available.
- F again lets go. The ship keeps its yard and tiller as they were, which is
  the cog's unattended policy.
- Other walkers are passengers (decision 2), not occupants.

**5. The save keeps the ship, and the walker on it.**
- The cog's record is the Tern's with its kind.
- A walker who quits on a deck is saved with the ship's ID and a local
  position. It loads there, even if the ship was left sailing, since the ship
  is saved where it came to.
- `RECORD_VERSION` goes to 2. A version 1 file reads with no deck positions.

## Risks / Trade-offs

- [The heel makes the deck a slope steeper than the walker stands on] → The
  walker's step and slope rules apply in the ship's frame. The rig is tuned so
  the cog heels under 15° on a beam reach. Past that, a walker slides to the
  lee rail and stops against it, which is what the rail is for.
- [Walking into the rail at speed pushes through] → The move is swept in the
  ship's frame, where the rail is still. The walker never sees the rail move.
- [Composing through a turning frame loses precision far from the planet's
  origin] → Poses stay `f64` until the local frame, as CLAUDE.md requires. The
  deck is at most 8 m from the ship's origin.
- [Frame cost] → One more ground query per walker, for craft in reach only.
  It is not measured in a cloud session. `perf_suite.py` gets a `sail`
  scenario.

## Migration Plan

- The vehicles file gains a version. Old files read without deck positions.
- A world that has a harbour's cog as a building (`cities-in-the-world`)
  gets it as a craft at the same mooring when this lands. The building piece's
  place becomes the craft's anchored pose.
- Rollback: the previous build refuses a version-2 vehicles file and asks,
  as it does for any newer save.
