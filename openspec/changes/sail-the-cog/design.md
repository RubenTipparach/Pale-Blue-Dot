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
- **Finding: the cog floated half a metre high (2026-10-01).** The game
  draws its sea `depth_offset_m` (0.5 m) under the sea level the layers are
  filled to (`cities-in-the-world`, 4.2b's finding). Step 1 cut the cog's
  waterline at the layers' sea level, so the quay shot shows its bottom
  above the drawn water.
  - The ship is cut with its waterline at the drawn sea, which the game
    passes to `build_town`, and its deck is 1.9 m over that.
  - The gangplank still runs from the pier's deck to the cog's. It now
    climbs 0.4 m, not the mockup's 0.9 m, because the pier stands 1.5 m
    over the drawn water.
  - Recommendation taken (ask only with screenshots).
- **Step 2 in detail (written 2026-10-01, before code).** Measured on this
  branch:
  - **The walker** is an Avian capsule. `drive_walker` sets its velocity
    from the keys. Avian moves it. `resolve_ground` then sweeps it from
    last tick's accepted position to the new one, in 0.2 m pieces, against
    the terrain and the towns' pieces (`walking::Structures`). Each piece
    answers in its own `Frame`, through `frame.local(p)`.
  - **So a piece moves rigidly when its `Frame` moves.** Nothing in a
    piece's solids or surfaces needs re-cutting. A deck is a piece whose
    frame is set each tick from the ship's pose.
  - **The cog's drawing is in the town's meshes**, baked in planet-local
    coordinates. A moving cog needs its own meshes, in its frame's
    coordinates, on its own entity, with a transform set from the same
    pose.

  How the walker lives on a deck (decision 2), in this codebase:
  - **Riding.** The ground query reports which piece holds the feet when a
    piece's floor wins over the terrain. A walker held by a moving piece at
    the end of a tick is on that deck.
  - At the start of the next tick, before it moves, the walker is carried
    by the deck's own motion over the tick: `p ← F_now · F_then⁻¹ · p`,
    with its heading turned by the same rotation. The same goes for the
    position the sweep starts from.
    - This is the local pose kept and composed. The walker's place on the
      deck is unchanged to rounding, so a turn neither slides nor jitters
      it.
    - It is not the rejected alternative, which added the deck's velocity
      and is only first-order in the turn.
    - Mouse look stays raw. The view turns with the deck because the
      heading is carried, never eased.
  - **Walking** is then the walker's own velocity, swept against the deck
    where the deck now is, exactly as on a town's floor. The rail is still
    in the deck's frame, so a walk into it is refused, never pushed through.
  - **Leaving.** When the feet stop being held by the deck (over the side,
    off the castle, a jump), the walker takes the deck's velocity at its
    point, `v + ω × r`, once, into its own. Landing on a deck takes it back
    out. So momentum is the composed velocity, as `world/frames` requires.
  - **No fictitious forces** (a non-goal): gravity stays the planet's.

  **The moving thing for step 2** is the moored cog swinging at its
  mooring on a script: it heaves 0.15 m on a 6 s period, rolls 2° on 7 s
  and swings 4° about its mooring on 23 s. The motion is small, known in
  closed form, and enough to prove the walker rides. Step 3 drives the
  same frame from the craft's integrator instead.
  - A `--cog-swing <scale>` flag scales it for tests and shots. The
    default, 1, is the swing a moored ship has. Recommendation taken (ask
    only with screenshots).

  **Verify** (the `player/walking` scenarios as app tests):
  - **Standing through a turn:** a walker set on the deck, the deck turned
    90° over 10 s, stays grounded at the same deck position to a centimetre.
  - **Up the stair under way:** a walker climbs the stair to the aftcastle
    while the deck swings at ten times the mooring swing.
  - **Over the side:** a walker walks off through the gangway and leaves
    with the deck's point velocity added.
  - **A still:** a capture of the walker on the swinging deck.
- **Step 2 as built (2026-10-01).**
  - **The cog is a deck.** `build_town` cuts the ship into its own meshes,
    apart from the town's, and records its piece. The game draws it on its
    own entity under the town's root and registers it in `decks::Decks`,
    keyed by its town and piece (`Towns::index`), so its place is found
    again as towns come and go.
  - **`move_decks`** sets each deck's frame from its swing (`Swing`: heave,
    roll, swing about its mooring, and a steady turn for tests) at the start
    of each physics step. The deck's piece answers in that frame, and the
    drawing's transform is set from the same frame.
    `--cog-swing <scale>` scales it, 1 by default.
  - **The walker's footprint reports which piece holds it.** A walker the
    deck held at the end of a tick is moved with the deck at the start of
    the next, before it walks.
  - **Measured: carrying the position itself drifts.** On the test planet,
    4.8 km out, an `f32` position steps every 0.49 mm. A deck turning a
    quarter round in ten seconds moves a point 2 m out about ten of those
    steps a tick, and each tick's rounding fell the same way as the last:
    the walker crept 9 cm in ten seconds.
    - So the walker's place on the deck is kept in the deck's own frame,
      where its numbers are small. Its position is made from that place
      each tick, and the place moves only when the walker does, by more
      than 2 mm.
    - It now holds to under a centimetre.
  - **Leaving.** The deck's way at the point the walker left is kept as
    drift. `drive_walker` writes the walker's velocity from the keys each
    tick, so a velocity added once would be gone a tick later. Drift is
    carried in the air and in the water, damped there as a fall is, and
    cleared on landing.
  - **Tests.**
    - `a_walker_stands_on_a_deck_through_a_quarter_turn`: held every tick,
      within 1 cm of where it was set, still facing along the deck.
    - `a_walker_climbs_the_cogs_stair_while_it_swings`: ten times the
      mooring swing (1.5 m of heave, 20° of roll, 40° of swing), onto the
      aftcastle with no tick in the air.
    - `a_walker_leaves_a_turning_deck_with_its_way`: it leaves through the
      gangway with 0.48 m/s of the deck's way, and lands with none.
    - The `decks` unit tests check that a carried point keeps its place
      over 500 ticks and that the drawing goes where the frame does.
  - **The cog casts its shadow from where it rests.** Its swing at a
    mooring is small.
  - **Not ticked yet:** tasks 3.1 and 3.2, and the `player/walking`
    requirement, are about a craft's deck while it sails. They tick when
    step 3 drives this same deck from the craft.
- **Step 3 in detail (written 2026-10-01, before code).** What it builds
  on, measured on this branch:
  - **A craft** (`pbd_core::vehicle`) is a spec in `vehicles.ron`, a
    `CraftState` per kind, and a `forces` function, stepped four times a
    tick in `FixedUpdate` (`step_vehicles`), before the walker's physics
    step.
  - **The Tern** is the model: a hull of cells floated on the sampled sea
    (`hull::float`, cells at most a third of the beam), its sail one foil
    whose angle the sheet stops, a keel and a rudder as wet foils, and
    resistance, windage, bilge and contacts.
  - **Kinds are matched** in 10 places in the core and 13 in the app:
    draw, view, chase camera, HUD, place, saves, harbour.
  - **A town's pieces** are numbered in `Structures` by town, and the
    numbers shift as towns come and go. A ship must not be one of them.

  The decisions:
  - **`Kind::Cog`, a spec of the Tern's shape.** Recommendation taken (ask
    only with screenshots).
    - The hull is the mockup's: 15 m by 5 m, 3.2 m deep, 1 m of sheer,
      floating 1.2 m deep. Its cells are 0.8 m, under a third of the beam.
    - Its mass comes from what that hull displaces at that draught, about
      45 t. Its parts put the ballast low.
    - **The square sail** is one foil on the yard, about 60 m², nearly
      square, quick to stall, and draggy. Its centre of effort is at the
      sail's middle, under the yard.
    - **The braces** turn the yard about the mast, at most 60° either way,
      at a fixed rate. A cog cannot point high: it makes good a track
      little closer than 70° off the wind (measured in part 1 as built,
      below).
    - A long keel and a stern rudder are wet foils.
    - Validation refuses a hull cell over a third of the beam (it already
      does) and a sail with no yard.
  - **`CogState { yard, tiller }`.** At the helm, W and S brace the yard
    round and A and D steer. F at the tiller takes the helm and lets go of
    it. A cog nobody steers keeps its yard and tiller as they were.
  - **Its deck is cut once, in the craft's own frame** (`cog::ship_in` at a
    frame at the origin), and held by the craft, not in `Structures`.
    - The walker asks the towns' pieces and the craft's decks.
    - `GroundState::on` names which: a town piece by its number, or a craft
      by its entity.
    - Each tick the deck's frame is the craft's pose. Its `then` is taken
      before `step_vehicles` and its `now` after, so `ride_decks` carries
      the walker by exactly the motion the craft made.
  - **It is drawn from the same cut**, the ship's meshes in the craft's
    frame, on the craft's entity. The harbour's piece and the craft are one
    model.
  - **The harbour's cog becomes a craft**, made once when the fleet first
    meets the harbour, as the boats are, with its berth `(site, COG)`.
    - **Moored, it keeps its berth on the step 2 swing.** Its integrator does
      not run, so the gangplank meets its deck. A moored ship's lines and
      fenders are not worth simulating.
    - **Casting off (T at the helm) hands it to its integrator** at the
      swing's pose and velocity. Making fast again (T within a few metres of
      its berth, slow) puts it back on the swing there. A cog at sea
      anchors as a boat does.
    - Once the craft is made, the town is cut without its ship, and the
      gangplank stays the town's.
  - **The stern lantern** is part of the cog's drawing, lit from dusk as a
    lamp is. It is no lamp block, because those cannot sail.
  - **Verify.**
    - Core: the spec validates, and a cog's sail follows its braces against
      the apparent wind.
    - Core: a scripted reach, beat and run log speed and heel. The heel
      stays under 15° on the reach.
    - App: a harbour's cog is made once, moored at its berth, and still
      boarded up its gangplank.
    - App: cast off, take the helm and sail a reach. Let go of the helm and
      walk to the aftcastle while the ship sails and turns. These are the
      `player/walking` scenarios, which move into the main spec then.
    - Shots: the cog under sail from the pier, and from its deck.
- **Step 3, part 1 as built (2026-10-01): the cog as a craft.**
  - **Built.**
    - `Kind::Cog` with its `CogSpec` in `vehicles.ron`, regenerated from the
      defaults: the hull, the square sail on its yard, the keel and rudder,
      the helmsman and the seat. A cog's yard is saved in its record.
    - The square sail is one foil whose chord runs along the yard; W and S
      brace the yard, A and D steer.
    - It is drawn from the harbour's own cut: `cog::sailing` is the ship in
      the craft's frame without its yard, and `cog::rig` is the yard with
      its sail, set at sea and furled at a mooring or an anchor. The app
      turns the rig by the yard's angle, on a pivot at the mast's step.
    - The HUD says whether the sail luffs, draws or is stalled, and the
      bindings table has a COG group.
  - **Finding: the flat foil pointed too high.** The first measure of the
    windward work read the telemetry at one instant. At 20° off the wind it
    reported 0.58 m/s made good to windward, which no sail can do. Traced
    second by second, the cog was in irons and driven astern. Going astern
    turned the rudder's effect round, so the helmsman spun it, and the keel
    carried its sternway round to windward. So the sail's signs were right
    and the measure was wrong. Measured instead as the mean over 60-120 s
    of a held course, the sail as a flat foil drew with the wind at 14° to
    the yard. Its best course to windward was 50° off the wind (0.61 m/s
    made good). That is a fore-and-aft rig's pointing, not a square sail's.
  - **The luff: the physics the flat foil lacked.** A square sail's weather
    edge is a free leech, not a spar. With the wind close along the yard it
    falls in, and the sail lifts nothing until the wind is well across it.
    So below `luff_deg` (15°) of wind to the yard the sail's lift is gone,
    and from `fill_deg` (30°) it lifts as the foil it is, blended between.
    Its drag stays whole throughout: a luffing sail flogs. Recommendation
    taken (ask only with screenshots); both angles are in `vehicles.ron`.
  - **Measured with the luff** (8 m/s wind, the mean over 60-120 s of a
    held course, the best brace for each course; the instrument is
    `print_the_cogs_polar`):

    | Off the wind | Made good to windward | Along its bow | Leeway | Wandered |
    | ---: | ---: | ---: | ---: | ---: |
    | 40° | +0.25 m/s | 0.85 m/s | 17° | 32°: cannot hold it |
    | 50° | +0.34 m/s | 1.05 m/s | 13° | 10° |
    | 60° | **+0.44 m/s** | 1.58 m/s | 9° | 6° |
    | 70° | +0.37 m/s | 2.10 m/s | 5° | 5° |
    | 90° | -0.26 m/s (leeway) | 2.21 m/s | 4° | 3° |

    Its best course to windward is 60° off with the yard braced hard. Its
    leeway there makes a track 69° off the wind, which is what a square
    rig does. It reaches at 2.2 m/s and runs at better than 2 m/s.
    `the_cog_reaches_runs_and_cannot_point_high` holds these facts.
  - **Not built yet**, in this order:
    - the deck held by the craft, so the walker rides it under sail;
    - the harbour's cog made a craft, moored on the swing, with cast off
      and make fast;
    - the stern lantern;
    - the app scenarios and the shots.
  - The frame cost is not measured: this is a cloud session, so
    `perf_suite.py` was not run.
- **Step 3, part 2 in detail (written 2026-10-01, before code): the deck
  under sail.** What it builds on, measured on this branch:
  - The walker's pieces are `Structures`, a list the towns rebuild as they
    come and go. `GroundState::on` names a piece by its place in that list.
    The moored deck (`Decks`) finds its place again by `(site, n)`.
  - Town pieces are planet-local and the walker's frame is too. A craft's
    pose is planet-local `f64`, and the vehicles reach the walker's frame by
    adding `PlanetRenderFrame::center`, which is nought outside an offset
    test.
  - Craft step in `FixedUpdate` (`step_vehicles`). The walker's step runs
    after, in the physics schedule: `move_decks`, `ride_decks`,
    `drive_walker`, and later `resolve_ground`.

  The decisions, recommendation taken (ask only with screenshots):
  - **`CraftDecks`**, a resource beside `Structures`: one deck per cog in
    the world, by its entity. Each holds `cog::sailing`'s solids, cut once
    in the craft's frame, and the frame they stand in. A system run first
    in the walker's chain sets each deck's frame from its craft's pose
    after the craft's step: `then` is the last tick's frame, `now` this
    one's. A deck whose craft is gone is dropped.
  - **`Piece`**: `GroundState::on` and the footprint name a town piece by
    its place, or a craft by its entity. The walker's queries (stand, a
    wall, a ceiling) ask the town pieces and then the craft decks.
  - **One motion for both decks.** The moored swing and the sailing ship
    both move a deck from `then` to `now`, and `ride_decks` carries the
    walker by it either way. The deck-local place (`on_deck`) is kept as it
    is for the moored deck.
  - **Leaving the helm** puts the walker at the seat's exit on the
    aftcastle with the ship's velocity as its drift. So its first tick
    does not leave it behind a deck making 2 m/s.
  - **Verify.** App tests, on a cog sailed by its own physics with nobody
    at the helm:
    - a walker set on its deck stays on it, at its place, through a turn;
    - a walker climbs the stair to the aftcastle under way;
    - a walker steps over the rail into the sea, keeping the ship's way
      until the water takes it off;
    - leaving the helm puts the walker on the aftcastle, riding.
- **Step 3, part 2 as built (2026-10-01): the deck under sail.**
  - **Built as written above.** `CraftDecks` holds each cog's deck, and
    `move_craft_decks` runs first in the walker's chain. `Piece` names a
    town piece or a craft. `Motion` is the one carry both kinds of deck
    use. Leaving the helm keeps the ship's way as drift.
  - **Found: the tiller recentred with nobody at the helm.** The yard kept
    its place, but the tiller still followed an empty helm back to amidships.
    The design says a cog nobody steers keeps both as they were, so the
    tiller now moves only with someone at the helm. The braces test pins it.
  - **Found: a heeling deck tipped the walker.** The deck carried the
    body's centre rigidly. As the ship heeled about 1° in its turn, the
    body tipped with it and was then stood upright again, and the feet crept
    1.8 cm through a 90° turn. A standing person keeps their feet still and
    their body upright. So the deck now carries the feet, and the body
    stands over them along the planet's up. The moored deck's tests pass
    unchanged. Through the turn the feet now move under a centimetre.
  - **Measured in the app tests:** coasting from 4 m/s with its tiller hard
    over and nobody at the helm, the cog turns through 90° in about 45 s.
    The test app has no air, so these are coasting turns, not sailing ones.
    The core tests sail it.
  - The walking requirement ("A walker stands and walks on a moving deck")
    is in `openspec/specs/player/walking` with the tests that prove it.
  - **Not built yet:** the harbour's cog made a craft, with cast off and
    make fast; the stern lantern; an app reach in a wind; the shots.
- **Step 3, part 3 in detail (written 2026-10-01, before code): the
  harbour's cog becomes the craft.** What it builds on, measured on this
  branch:
  - A harbour's boats are made once by `moor_harbours` when the fleet
    first meets the harbour. Each is tagged with its berth `(site, n)`, and
    a craft far off is stowed as its record.
  - T at the helm casts off a moored craft or anchors it.
  - The town stands its ship as a piece in `Structures`, on the step 2
    swing (`Decks`), drawn by `spawn_cog`.

  The decisions, recommendation taken (ask only with screenshots):
  - **The cog's berth is `(site, COG)`**, `COG` being `u32::MAX`, past any
    boat's number. `moor_harbours` makes it as it makes the boats. It is
    made at its berth's rest pose, which `pieces::cog::berth` gives from
    the chart: the waterline's middle on the drawn sea, and the bow. Its
    mooring is a bollard, not an anchor.
  - **On its swing.** A cog moored by a bollard at its own berth is not
    stepped. Each tick its pose is the step 2 swing's about the berth's
    rest frame, scaled by `--cog-swing`, and its velocity is the swing's.
    So its deck (part 2) carries the walker exactly as the piece's did. A
    `Berthed` component caches the rest frame, worked out once from the
    held town.
  - **Casting off** (T at the helm) drops the bollard. The integrator then
    takes the cog at the swing's pose and velocity.
  - **Making fast** (T at the helm) takes the bollard again when the cog is
    within 4 m of its berth, slower than 0.8 m/s and within 30° of its
    berth's heading. It then eases onto the swing over 3 s, so it does not
    jump. Anywhere else, T anchors it as it anchors a boat.
  - **The town stands without its ship.** The cog is always the craft now.
    The ship's place in `Structures` is kept empty, so the towns' numbering
    holds, and it is not drawn. The gangplank stays the town's. The town's
    `Decks` machinery stays, tested, for any town piece that moves.
  - **The ship no longer casts the town's shadow.** Craft do not cast in
    the town shadow pass, and the cog is a craft now. That is noted, not
    fixed here.
  - **Saves.** The cog's record carries its berth and its bollard, so a
    loaded cog is back on its swing. A world made before this change has no
    cog in its fleet, so `moor_harbours` makes one the first time it meets
    each harbour, at the same berth.
  - **Verify.**
    - Core: `cog::berth` agrees with the piece's frame and `cog::bow`. The
      town's gangplank lands on the craft's cut stood at the berth, and a
      walker walks up it, across the deck and up the stair.
    - App: every harbour's cog is made once at its berth, on a bollard, on
      the drawn sea; asked again, it makes none.
    - App: a moored cog follows its swing to a millimetre. Cast off, it
      sails on its own physics. Made fast 2 m off its berth, it eases
      onto the swing. T far from its berth anchors it.
- **Step 3, part 3 as built (2026-10-01): the harbour's cog is the craft.**
  - **Built as written above.**
    - `moor_harbours` makes each harbour's cog with its boats (`cog_for`),
      at `pieces::cog::berth`'s rest pose, on a bollard, tagged
      `(site, COG)`.
    - `swing_moored_cogs` rides it on its swing after the craft step, and
      the step skips it. Its velocity is the swing's, and it counts as at
      rest, so the save does not keep rewriting it.
    - T at its helm casts it off. T within 4 m of its berth, slow and
      heading as it lies, makes it fast and eases it back over 3 s.
      Elsewhere T anchors it.
    - The town keeps the ship's place empty and does not draw it.
  - **The swing's clock** is the fixed step's elapsed time, so the swing is
    the same instant the test names. The moored-deck `Decks` keeps its own
    clock for town pieces.
  - **Proved by:**
    - core `the_craft_cog_stands_at_its_berth_and_is_boarded_up_the_towns_gangplank`
      (the craft's cut at `berth` is the piece's place, and the town's
      gangplank boards it);
    - `towns::tests::every_harbours_cog_is_made_once_at_its_berth`, over
      every harbour on the shipped seed;
    - `vehicles::tests::a_harbours_cog_rides_its_swing_casts_off_and_makes_fast_again`
      (on its swing to a millimetre; cast off it sails off; made fast 2 m
      off its berth it does not snap but eases on; 25 m off, T anchors);
    - `a_moored_cog_is_saved_on_its_bollard_at_its_berth`.
  - **A world made before this change** has no cog in its fleet. So
    `moor_harbours` makes one at the berth the first time it meets each
    harbour, which is the same path as a new world's.
  - **Not built yet:** the stern lantern, an app reach in a wind, and the
    shots.
- **Step 3, part 4 in detail (written 2026-10-01, before code): the stern
  lantern.** What it builds on, measured on this branch:
  - A craft takes its light from the planet's light field, which only lamp
    blocks feed, and a lamp block cannot sail.
  - A town's rooms carry their own lights (`RoomLights`): each a place in
    the planet's frame, a reach and a band of height, packed onto the
    room's field-lit material once and summed in `field_lit.wgsl`. A
    candle's lights only by night.
  - A town's flame is drawn unlit (`FLAME_RGB`).
  - The street lamps burn while `DuskLamps` says the dusk lamps are lit
    where the player is.

  The decisions, recommendation taken (ask only with screenshots):
  - **The lantern** is the mockup's `lantern()` where the mockup puts it on
    the cog: a 2.6 m post on the aftcastle, 86% of the deck's half-length
    aft on the middle line, with an arm out to starboard. A small timber
    cage hangs from the arm 2.3 m up, with a flame inside. It is cut by
    `pieces::cog::lantern` in the craft's frame. The flame is drawn unlit
    and shown only while the dusk lamps are lit, as a street lamp burns.
    (First cut as a short post at the stern rail; moved to the mockup's
    form when its shots were taken beside the game's, before the game's
    were.)
  - **Its light** is the mockup's lamp (power 0.7) as a candle: by night
    only, 5 m, warm, lighting the aftcastle and the helm down to the main
    deck. It is a `RoomLights`
    entry on each of the ship's meshes. Each frame its place is moved to
    where the lantern is, and a `RoomLights` that changes is packed onto
    its material again.
  - **Verify.** Core: the lantern stands on the aftcastle and has its flame.
    App: the flame is shown with the dusk lamps and hidden without them,
    and the light's place goes with the ship when it moves.
- **Step 3, part 4 as built (2026-10-01): the stern lantern.** Built as
  written above.
  - `pieces::cog::lantern` cuts the post, the cage and the flame, and
    `stern_light` gives its light.
  - The app draws the flame unlit and shows it while `DuskLamps` is lit.
    Each of the ship's meshes carries the light (`CarriedLight`), moved
    each frame to where the lantern is. `repack_room_lights` packs a
    changed `RoomLights` onto its material again.
  - Proved by core `the_cogs_stern_lantern_stands_on_its_aftcastle` and app
    `a_cogs_stern_lantern_burns_from_dusk_and_its_light_goes_with_it`. How
    it looks at 22:30 is for the shots.
- **The shots' instrument (written 2026-10-01, before code).** A headless
  capture can board a craft (`--aboard`) but cannot cast off or set a
  sail. Two capture flags change nothing a player can do:
  - `--aboard cog` waits for the harbour to make its cog, where it used to
    give up on the first frame.
  - `--sail <deg>` casts the boarded cog off, braces its yard `deg` off
    square, and gives it 2 m/s of way along its bow. A capture of a few
    seconds then shows it under way, sail set, rather than gathering way
    from rest.

  The shots are:
  - the cog under sail, in the chase view;
  - the cog from its helm, under sail;
  - the moored cog from the quay at 22:30, its lantern lit.
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
