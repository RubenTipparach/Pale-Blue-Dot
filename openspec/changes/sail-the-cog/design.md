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

**6. The order of building (written 2026-10-01).** The owner, on the
harbour mockup: "We're good on the mockup already. Approved. Implement in
game". The cog is fourth in that order, after the street lamps, the moored
boats and the dressing (`cities-in-the-world`, "4d as built"). What it
builds on, measured on this branch:
- **The harbour stands in the game without its cog.** The export leaves the
  cog out, with its stern lantern, its barrel and crate, and the gangplank.
- **The pieces it needs are partly built:**
  - the lofted hull (`pieces::dressing::Hull`, the mockup's `hullSection`);
  - the ramp surface, which is the gangplank's;
  - thin solids and surfaces, flights, rails and the walker's `stand` over
    them (`tenebris-towns` 1 and the towns' slice 2b).
- **`LocalFrame` still only translates**, and the walker still stands only
  on fixed pieces (`walking::Structures`) and the terrain.

So it comes in four steps, each with its shots. Recommendation taken (ask
only with screenshots).
1. **The cog moored.** The ship piece (`tenebris-towns` 3d) stands at its
   berth by the main pier, as the mockup draws it, cut on the harbour's
   chart in its own frame as the light is:
   - the hull, the deck following its plan, the rail with its gangway gap;
   - the castles and the stair to the aftcastle;
   - the mast, the yard with its sail furled, and the shrouds;
   - its barrel and crate;
   - the gangplank, a ramp from the pier up to the deck.

   It is derived from the template, as the dressing is, and never saved. It
   does not move. Verify: core walker tests up the gangplank, across the
   deck and up the stair to the aftcastle, and shots beside the mockup's
   `cog` and `castle` views.
2. **A turning frame and a moving deck** (tasks 1.1, 3.1, 3.2). `LocalFrame`
   gains its orientation and angular velocity. The walker keeps a local pose
   on a deck. It is tried first on the moored cog made to bob and swing at
   its mooring, where the deck's motion is small and known.
3. **The cog as a craft** (tasks 2.1 to 2.3). `Kind::Cog`, drawn from the
   same piece. Each harbour's cog is made once at its berth, as the moored
   boats are (`cities-in-the-world` 4.2b), and the step 1 piece stops being
   cut. Nothing saved has to move, because step 1 saved nothing.
4. **The helm and the saves** (tasks 4.1, 5.1). Task 5.2 becomes the step 3
   handover: there is no building cog in any save to turn into a craft.

- **Step 1 as built (2026-10-01).**
  - The export writes the cog (where it lies, its heading, its gangway's
    side) and its gangplank. Nothing else in the harbour changes, and the
    village and walled town re-export byte-identical.
  - `pieces::cog` cuts it at the sea's surface in its own frame, from the
    mockup's sizes:
    - the hull, lofted by the dressing's `Hull`;
    - the deck as a floor over a solid hull down to the keel;
    - the rail round the deck with the gangway's 1.4 m gap;
    - the castles, solid, railed but on the waist's side;
    - the eight-riser stair to the aftcastle;
    - the mast, the yard with the sail furled, and the shrouds;
    - a barrel and a crate.
  - **The gangplank** is the ramp the slip is, from the pier's deck up to
    the cog's.
    - The mockup ends it 2 cm over the deck's edge. Cut in two frames on the
      chart, the plank's end and the deck's edge met within a millimetre,
      and the walker found nothing underfoot there.
    - So a landing the walker stands on, not drawn, laps 30 cm onto the
      deck at its height, as a pier's stretches lap.
  - The harbour's cells over the water take in the cog's plan and the
    gangplank. A harbour stored before this charted none, so its cog is left
    out and counted with the dressing off its chart.
  - `a_walker_boards_the_moored_cog_and_climbs_to_its_aftcastle` walks from
    the main pier up the gangplank, across the deck and up the stair onto
    the aftcastle. `every_harbour_lays_on_its_sea_and_cuts` holds each of
    the six harbours' cogs standing.
- **The stern lantern waits for step 3.** A town's lanterns are lamp blocks
  in the voxel field, which cannot sail. A light carried on a craft is part
  of the craft's drawing.

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
