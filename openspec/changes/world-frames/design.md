# world-frames: design

## 1. The finding

`movement-smoothness` built a render-only floating origin and kept physics and
simulation planet-centred (its design 4: "collision needs 0.5 mm, not
0.005"). That was wrong, and the owner said so: "its not the camera thats the
problem, the math still shows: 4436.446780".

The number is the walker's physics `Position.x`. An f32 between 4 096 and
8 192 holds steps of 2^-11 m = **0.488 mm**. The walk trace's drawn x after
the render origin, 4.4467773, is exactly f32(4436.4467773) - 4432: the render
origin subtracts after the store has rounded. Everything integrated in the
planet's frame is rounded there every step:

- the walker: Avian's `Position += v dt` (4 substeps), and then the ground
  resolution's `up * (support + h)`, all in planet-centred f32;
- the skiff (an Avian body);
- the fish (`School::step`: `p += v dt`, then a clamp `p *= r'/r`);
- the line (`Line::step`: the float's flight, and its bob `up * (surface -
  dip)`).

A 1 m/s walk moves 16.7 mm a step. Its error is a fixed fraction of a
millimetre, so slow and small motion is hurt most: a fish moving a few mm a
step, a float bobbing mm, a body at rest.

CLAUDE.md already says what to do: "Store system coordinates and orbital time
as f64; render and simulate collision in bounded local f32 frames. Subtract
the origin before casting. ... Rebase poses, interpolation history, physics
state, and frame-relative velocities together at a tick boundary."

## 2. Measured before (M4 Max MacBook Pro, 2026-11-17)

**The instrument** (a measurement instrument, CLAUDE.md "Write it up before
touching code"): `PBD_STEP_TRACE=<csv>` (`step_trace.rs`) writes one row per
fixed physics step, after the step: the walker's stored `Position` and
`LinearVelocity` at full f32 precision (`{:?}` prints the shortest digits that
read back to the same f32), the physics frame's origin, and the walker's
planet-frame point rebuilt from them in f64. `tools/step_graph.py` compares
each step's move with `v dt` over the steps of a steady walk (speed within
10%, velocity changed under 1%). `PBD_WALK_AXIS_SCALE=0.125` makes the
`--walk-distance` script walk (not sprint) at 1/8 of the 8 m/s walk: 1 m/s.

Run: `PBD_WALK_AXIS_SCALE=0.125 PBD_STEP_TRACE=steps.csv pbd-app --world
<fresh> --walk-distance 25`, on the column tier (ticks after it lands).

A slow walk, the build before (`claude/fish-pose-glitch` at `761c143d`):

| tick | dt | x (stored) | y (stored) | z (stored) | world dx | v dt x | error |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 80 | 0.016666667 | 4434.2583 | 1987.8191 | -230.91805 | -0.977 mm | -0.866 mm | 0.111 mm |
| 81 | 0.016666667 | 4434.258 | 1987.8192 | -230.93472 | -0.488 mm | -0.866 mm | 0.399 mm |
| 82 | 0.016666667 | 4434.257 | 1987.8191 | -230.95135 | -0.977 mm | -0.866 mm | 0.164 mm |
| 83 | 0.016666667 | 4434.256 | 1987.8191 | -230.968 | -0.977 mm | -0.866 mm | 0.111 mm |

x moves by whole 0.488 mm steps (one, two) against a true 0.866 mm; y flips by
0.12 mm (its own f32 step) where it should not move. Over 119 steady steps:
**|move - v dt| p50 0.111 mm, p99 0.906 mm, max 0.917 mm; 1.49% of the step
on average, 5.5% at worst.**

The same numbers for the fish and the float come from the guard tests
(design 6), run with the origin at the planet's centre (the old simulation):
a fish's 30 Hz step is off by **0.23 mm**; the float's bob by **0.38 mm** and
its drift along the water by **0.11 mm** a step.

## 3. Options

**(a) One shared local origin.** Physics, simulation and rendering in f32
local to an f64 origin near the player, rebased by whole cells at a tick
boundary. What CLAUDE.md prescribes. Avian stays f32 (fast, and the
`f32`/`parry-f32` features the project builds with). Every "up =
`position.normalize()`" becomes "up through the origin", in f64. Saves are
converted at the boundary and do not change on disk. Blast radius: every
reader and writer of a simulated position, about 30 files.

**(b) Avian `f64`/`parry-f64` and f64 simulation.** Positions in f64 at the
planet's scale: 1e-12 relative, nothing to rebase. But every Avian type
changes (`Position(DVec3)`, `Rotation(DQuat)`, forces, collider sizes), parry
in f64 is slower, every shader and render transform still needs an f32 local
frame, so the origin is needed anyway, and at system scale f64 itself runs
out (30 µm at 1 AU), so a frame hierarchy is needed anyway. It moves the
problem rather than removing it.

**(c) A hybrid**: f64 positions in `pbd-core` only, Avian planet-centred f32.
The walker is an Avian body, so its integration stays rounded: it does not
fix the case the owner saw.

**Chosen: (a)**, as CLAUDE.md prescribes and as the solar-system scale needs
(design 9): local f32 is exact near the player at any scale, and only the
origin has to be good at the large one. Avian stays f32 in a local frame.

## 4. The frames (`pbd_core::world_frame`)

- **`FrameTree`**: node 0 is the system frame (inertial, the root). A node has
  a parent, a kind and a motion:
  - kinds: `System`, `BodyInertial` (a body's centre, axes fixed),
    `BodyFixed` (axes turning with the ground), `Vessel` (a ship, a
    station);
  - motions: `Rails(CircularOrbit)` (the ephemeris's orbits), `Spin { axis,
    rate, phase }`, `Vessel { pose: LocalFrame, at }` (set by the game each
    tick, carried at its velocities between).
  - `in_parent(id, t)` is the frame's pose in its parent as a `LocalFrame`
    (the existing type, with its `ω × r` composition).
  - `convert(FrameState, to, t)` takes a position and a velocity from any
    frame to any frame, **through their lowest common ancestor only**: two
    frames near each other far out in the system never pass through
    system-sized numbers.
  - `reparent_vessel(id, parent, t)` moves a vessel under another frame,
    keeping its place and motion: docking, leaving a sphere of influence.
- **`WorldPos { frame, pos: DVec3 }`**: a position with its frame. Stored as
  `(frame, [f64; 3])`. Every long-lived position is one of these; new code
  never keeps a planet-frame point as an f32 `Vec3`.
- **`SimOrigin { frame, origin: DVec3 }`**: the local space. A local `Vec3`
  is `origin + v` in `frame`, axes `frame`'s. Its methods are the only
  conversions between local and world (no `From`/`Into` exists):
  - `to_world(v) -> WorldPos`, `to_local(WorldPos) -> Vec3` (f64, cast after);
  - `up_at(v)`, `radius_at(v)` (f64), `with_radius(v, r)` (a body stood or
    held at a radius, in f64), `at(up, r)`;
  - `query_point(v) -> Vec3`: an f32 point in the frame for a query that is
    content with half a millimetre (a terrain column, a town wall, the sea);
    it is asked, never stored or integrated; `from_query_point` for a query's
    answer (a deck's place).
- **Rebase** (`rebase_for(anchor, cell) -> Rebase { shift, delta }`): the
  origin snaps to a grid cell (`ORIGIN_CELL_M` = **128 m**, the owner: "16 m
  is kinda small, 128 m probably good") once the anchor is more than 0.75
  cell (96 m) off it on an axis, so standing on a boundary never thrashes:
  after a rebase the anchor is about 32 m from the new origin and must walk
  64 m back to cause another. A local coordinate stays within 96 m of the
  origin, where an f32 holds 7.6 µm.
  The cell is a power of two, so every shift is exact in f32; a shifted value
  rounds only to the f32 step at its own small size (a micrometre at 16 m).
  `cell_for_speed(speed, dt)` is 128 m on foot and doubles until a step is
  under a quarter cell: 128 m to 1.9 km/s, 1 024 m at 10 km/s, so a fast ship
  rebases every few ticks, not every tick, and nothing assumes rebases are
  rare.
- **Handoff** (`SimOrigin::handoff(tree, to, t) -> Handoff`): the old local
  space as seen from the new, at one tick: `state(pos, vel)`,
  `rotation(q)`, `angular_velocity(w)`. Positions, velocities (with the
  turning terms) and orientations go through it together, so nothing jumps.
  A turning frame is handed to only near its body: far from it the ground's
  `ω × r` is millions of m/s and an f32 velocity holds metres a second of it.

Tests (`world_frame.rs`): a rebase keeps every world position and moves by
whole cells; a slow step at 4 800 m and 4 800 km is exact in local space
(and off by > 10 µm planet-centred); conversions round-trip between any two
frames of a system with a planet at 1 AU, its turning surface and a second
planet at 1.52 AU; a converted velocity is the rate of the converted
position; a walker in a ship at 10 km/s steps exactly with a rebase every
tick, and is seen from the planet at the ship's speed plus its own; the trip
surface → planet → system → second planet → system → planet → surface keeps
position (1 mm, f64 at 1 AU) and velocity; an arrival hands off to a turning
surface without a jump; a vessel reparented keeps its place and motion.

## 5. In the game today (`sim_origin.rs`)

- `WorldFrames`: the system and one body, the home planet (`HOME`,
  `BodyInertial`, stationary at the system's origin). The planet does not
  turn today; when it does, its ground is a `BodyFixed` child and the local
  frame of a walker is that.
- `SimFrame(SimOrigin)`: the one local origin. **Physics space is its local
  space.** `PhysicsFrame.origin` (the system point of physics zero, which
  gravity and the sky read) moves with it in the same call, as does the render
  origin.
- **The rebase** (`follow_player`, `PreUpdate`: between frames' fixed loops, a
  tick boundary, once a frame): the anchor is the walker while walking, else
  the piloted skiff, else the active camera. `apply` moves:
  - every Avian body's `Position`, `Transform` and `GlobalTransform` (Avian
    reads a body's `GlobalTransform` back into its `Position` where they
    differ by 0.01 mm and the `Position` did not change);
  - every registered `RebaseHook` (`add_rebase_hook`): the walker's
    `GroundState.previous`, `StepHistory` and a climb in progress; the
    skiff's `ProtectedPosition`; the fishery (the line's float, every school's
    anchor, fish, step-before positions, homes, goal, lure and held points),
    the drawn float's last place and the under-water watch's eye;
  - `PhysicsFrame.origin += delta`, `SimFrame.origin += delta`, and the render
    origin `-= delta`, so render space (physics less the render origin) is
    unchanged and **no render transform is walked**. The render origin still
    follows the camera on its own 16 m grid (`movement-smoothness` 6.2); it is
    now an offset inside the local space, not a second world origin. A rebase
    costs O(simulated things).
  No velocity is touched: the local frame does not move in its frame.
- **Teleports** (`setup_walking`, `restore`, `place_walker`, `drop_walker`,
  `teleport_pilot`, the reset key) call `recenter_on_point` *before* writing
  the position, so a teleport across the planet writes a small local number
  and nothing is rounded at the planet's scale.
- **The walker** (`walking.rs`): `drive_walker`, `ride_decks`,
  `resolve_ground`, `sweep`, `water_climb`, `Climb::at`, `follow_walker`
  take up, radii and stood positions through the origin. The ground queries
  (`footprint_in`, `support_past_sliver`, `rise_normal`, `Sea::state`, the
  towns' `Meets`) take planet points from `query_point`. One more f32 had to
  go: the support a body is stood on. The facet floor is computed in f64 and
  was cast to an f32 radius, a stair of 0.488 mm on a tilted facet; the
  contact now carries `floor_precise: f64` and the walker stands on that.
- **The skiff** (`flight_view.rs`): Avian body in local space; terrain
  protection through the origin; the route and the tour steer by planet
  points; the readout reports planet points.
- **The fish and the line** (`pbd-core`): `School::{spawn, step, scent}` and
  `Line::step` take the `SimOrigin`; their positions are local; `rebase(shift)`
  moves them. The `Water` is asked at planet points. The app's fishery puts
  its feet, eye, rod tip and new schools into local space and draws from it
  (`render_from_physics`).
- **Every other reader of the walker** (towns' doors, the map, missions,
  drops, the showcase bot, the capture scripts, vehicle placing) takes its
  planet point from `query_point`. Sites that already used
  `PlanetRenderFrame::physics_center()` were right as they stood: the
  planet's centre in physics space is `-origin`, and it moves with a rebase.
- **Persistence**: the saved pose is the home body's frame, as before; it is
  written from `query_point` and read through `to_local`, so saves are
  unchanged on disk and round-trip to the bit (test). Recording the frame id
  with each saved point (`WorldPos`) is a format change, made when a second
  frame can be saved in (design 9).
- **The A/B**: `PBD_HOLD_SIM_ORIGIN=1` holds the origin at the planet's
  centre (the simulation before this change); `PBD_HOLD_ORIGIN=1` holds the
  render origin too (the build before `movement-smoothness` 6).

## 6. Guards (how the math stays in the floating origin)

1. **Types.** A world position is a `WorldPos`, a local one a `Vec3` (Avian's
   `Position`); the only conversions are `SimOrigin`'s methods.
2. **`ORIGIN_LEAK`.** `world_frame::check_local(site, v)`: in a debug build a
   local value past `LOCAL_BOUND_M` (2 km) prints `ORIGIN_LEAK <site> <value>`
   once per site. Called on the walker (`drive_walker`), the angler (`fish`),
   each school's step and the line's float. Free in a release build.
3. **Guard tests** (`world_frame_guards.rs`, `walking::sim_origin_tests`):
   a school stepped at 4 800 m and 4 800 km, on a body at the system's origin
   and at 1 AU; the float's bob at both radii; a rebase of the school and the
   line; the walker's slow walk in the game's own plugins with the origin
   following, across a rebase; a rebase that moves no body and changes no
   velocity; a saved pose's round trip. Each also measures the planet-centred
   case and asserts it is worse than 10 µm, so the test shows what it guards.
4. **Source check** (`crates/pbd-app/tests/origin_guard.rs`): reads the
   converted simulation's sources (walking, flight view, fish, the origin,
   the school, the line; test modules left out) and fails on a
   `position.normalize()`, `accepted.length()`, `float.normalize()`-style
   "up" or radius. A query parameter is named `point`, not `position`, so it
   reads as what it is.

Proposed for CLAUDE.md (the owner's file; not edited here): "Simulated
positions are local f32 relative to the one `SimOrigin`; up, radius and
planet points go through its methods; anything stored is a `WorldPos` in its
body's frame. A planet-centred f32 is a query, never a state."

## 7. Results after (M4 Max MacBook Pro, 2026-11-17)

The same slow walk, this branch:

| tick | dt | x (stored) | y (stored) | z (stored) | world dx | v dt x | error |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 95 | 0.016666667 | 2.2457511 | 3.8189092 | -7.167817 | -0.8667 mm | -0.8667 mm | 0.38 µm |
| 96 | 0.016666667 | 2.2448845 | 3.8189068 | -7.1844616 | -0.8667 mm | -0.8668 mm | 0.39 µm |
| 97 | 0.016666667 | 2.2440176 | 3.8189044 | -7.201106 | -0.8669 mm | -0.8668 mm | 0.38 µm |
| 98 | 0.016666667 | 2.243151 | 3.8189023 | -7.2177505 | -0.8667 mm | -0.8669 mm | 0.43 µm |

| | before | after | `PBD_HOLD_SIM_ORIGIN=1` |
| --- | --- | --- | --- |
| slow walk, \|move - v dt\| p50 | 0.111 mm | **0.25 µm** | 0.162 mm |
| slow walk, p99 | 0.906 mm | **0.86 µm** | 0.399 mm |
| slow walk, max | 0.917 mm | **0.86 µm** | 0.399 mm |
| 8 m/s walk, p50 / max | | 1.85 / 2.3 µm | |
| fish step (guard test) | 0.23 mm | **0.78 µm** (1.0 µm at 4 800 km) | |
| float bob / drift (guard test) | 0.38 / 0.11 mm | **0.34 / 0.48 µm** (4 800 km) | |

The 8 m/s walk crossed 5 rebases: at each, the stored point jumped by a cell
(15.9 m) and the planet-frame point moved by the step's `v dt`, to 1 µm on
the steady ones. The held origin reproduces the old error, which shows it is
the origin alone. Before the column tier lands, the coarse ground still
answers with a little noise (0.2 to 0.4 mm on a tilted coarse facet's
queries); it is gone once the tier is under the walker, which is all of play.

(The tables above were taken with a 16 m cell; with the 128 m cell the same
guard tests give 2.1 µm a step at 1 m/s and 7.4 µm at 8 m/s, where the walk
reaches the far side of a cell, and the 8 m/s walk in the game 3.0 µm p50,
8.6 µm p99, 10.5 µm max over 1 610 steady steps: the f32 step at 96 m.)

### 7.1 Fishing (`--fish --fish-play --hold-pointer`, 45 s, real time)

`tools/motion_graph.py`, before and after, each on a fresh world (the
machine was busy; these are behaviour runs, not repeats of one another):

| | before | after |
| --- | --- | --- |
| float on screen, jitter p50 / rms (px) | 0.041 / 4.84 | **0.019** / 4.58 |
| hooked float, jitter p50 / p99 (px) | 0.72 / 10.7 | 0.65 / 10.1 |
| fish on screen, jitter p50 / p99 (px) | 0.43 / 10.5 | 0.44 / 9.5 |
| line start from the drawn rod tip, rms (px) | 0.31 | **0.00** |
| held tool shimmer rms (px) | 0.0021 | 0.0019 |

The angler stands still, so no rebase happens while fishing; what is left
in the fish's and the hooked float's jitter is their behaviour (turns and
runs), which design 8 of `movement-smoothness` owns. The float's resting
jitter halves. The line now starts at the rod tip's render point itself.
`motion_graph.py`'s "camera distance from the render origin" reads the
render origin as a planet point and is wrong after this change (it reports
4.8 km); the held tool's shimmer shows the eye is near it.

### 7.2 Across a rebase (128 m cell, an 8 m/s walk)

`PBD_REBASE_SHOTS=<dir>` (a measurement instrument in `desktop.rs`) saves
the frames each side of a rebase. One walk, `--walk-distance 330`:

- physics, from `PBD_STEP_TRACE`: at the rebase tick the stored point jumps
  127.92 m (the cell) and the planet-frame move is the step's, the same
  error as the ticks either side (a fall: 2.60 mm a step against `v dt` read
  after the step, on every falling tick, rebase or not);
- drawn, from `PBD_MOTION_TRACE`: the body moved 0.17, 0.72, 0.19, **0.68**,
  0.13, 0.67, 0.13 m on the frames round the rebase frame (bold), the
  camera's render point 0.17, 0.75, 0.18, **0.78**, 0.12, 0.59, 0.14 m: the
  walk's own one-step and two-step frames, no jump;
- pixels, consecutive frames (mean absolute difference, 0-255): 8.0, 16.2,
  8.9, **18.1**, 5.8, 11.8, 6.2, 11.6. The frames alternate between one and
  two physics steps; the rebase frame sits in its parity's band (16.2, 18.1,
  11.8, 11.6), no pop.

What a rebase can and cannot disturb:

- **Physics.** Avian bodies' `Position`, `Transform` and `GlobalTransform`
  move together; velocities are untouched. Avian's contact anchors are
  relative to each body's centre of mass and its warm-start impulses are
  translation-free; the world-space contact point and every collider's AABB
  are rebuilt from the positions in the next step's narrow and broad phases.
  The game has no joints, no static colliders (terrain is the walker's own
  ground resolution, not Avian) and its two bodies never sleep
  (`SleepingDisabled`). Measured: no spike at the rebase tick.
- **Visuals.** Render space is physics space less the render origin, and the
  render origin moves with the shift, so a sim rebase changes **no** render
  transform, no view matrix, no `PlanetRenderFrame::center` and nothing a
  shader reads: the GPU's history (clouds, SSR, rain, shadow cascades,
  motion) sees nothing happen. The shaders keep sampling planet-relative
  coordinates (`movement-smoothness` 6.4), so no texture swims. The
  interpolation histories kept in local space move with it: `StepHistory`,
  every school's step-before positions, `DrawnFloat`'s last float, the
  watch's eye; `DrawnHeadings` holds directions only. The line mesh and
  the school meshes are rebuilt each frame.
- **Cost.** A rebase walks the Avian bodies (two) and the hooks (the walker,
  the skiff, the fishery: a few hundred subtractions); once per 96 m on foot.

### 7.3 Launches

- A fresh world, a slow walk, an 8 m/s walk, fishing: to `READY` with no
  `LATE_PIPELINE`, no panic.
- A world saved by the build before (`so-throwaway-fish-before`, copied):
  opened by this build, played 8 s and saved: the walker's saved position is
  the old one to the bit (`(-2354.408, 1953.1152, 3701.4177)`); the craft
  are where they were (their f64 orientations differ in the 16th digit, the
  water's own drift over 8 s).

## 8. Not converted (limits)

- **Decks** (`decks.rs`, the cog's deck): a deck's frame is planet-local f32,
  so a walker on a deck holds the deck's half millimetre. The fix is the
  vessel frame of design 4, a deck as its own frame.
- **Towns' solids** are planet-local f32 queries: a wall's edge is where it is
  to half a millimetre, which collision does not mind.
- **The vehicles** (`vehicles.rs`) already simulate in f64 planet-local
  (`craft.body.position: DVec3`) and reach physics space through
  `physics_center()`, so they are right as they stand.
- **The herd** (`husbandry`) and **drops** are planet-local f32: animals and
  drops are stepped and drawn at half a millimetre. Their conversion is the
  same pattern (pass the origin, store local, add a hook), left for a second
  pass.
- **The line's sag** is laid in planet points (draw only).
- **Terrain queries** answer f32 radii (columns, facets, the sea). The walker
  is stood on the f64 facet floor; the rest compare at half a millimetre.
- **Frame cost**: not measured with `perf_suite.py`; a rebase is a few dozen
  subtractions per 16 m on foot.

## 9. Solar-system scale

The owner: "we will be walking on planets, flying spaceships around the solar
system". What this change builds is the machinery; what follows is how it is
used as the game grows, so the types do not need redoing.

- **The hierarchy.** System frame (orbits, f64) → body frames (planet, moon,
  station; f64 relative to the body's centre, inertial or turning with the
  ground) → vessel frames (a ship, hanging from whatever it is near) → the
  local frame (f32 near the player). f64 relative to a body's centre holds
  1e-9 m at planet scale; relative to the system it holds 30 µm at 1 AU and
  1 mm at 30 AU, fine for orbits and never used for a surface or a walk.
- **The local frame follows the player at any speed.** The cell grows with
  speed (`cell_for_speed`), and a rebase is O(simulated things), so a ship at
  km/s can rebase every few ticks.
- **A ship is its own frame.** People walking inside a moving ship simulate in
  the ship's local frame (CLAUDE.md: "Station passengers simulate in the
  station's local frame"): the walker's `SimOrigin.frame` is the ship's, so
  walking in a ship at 10 km/s is as exact as on the ground (tested).
- **Handoff along a trip.** Surface (`BodyFixed`, turning) → orbit
  (`BodyInertial`) → interplanetary (the ship's vessel frame, reparented
  from the body to the system when it leaves the sphere of influence) →
  arrival (the vessel reparented to the other body, then the walker handed to
  its turning surface near the ground). Each handoff is at a tick boundary;
  positions, velocities (with `ω × r`), orientations, interpolation history
  and physics state go through one `Handoff` together.
- **Rendering at system scale.** Everything is placed camera-relative,
  computed in f64 then cast (the render origin of `movement-smoothness` 6, an
  offset in local space); a reverse-Z infinite far plane; distant planets and
  stars as scaled stand-ins at a fixed far distance.
- **Terrain at large radii** samples its noise in body-relative f64, or
  relative to a local tile origin, and answers heights as altitudes (small
  f32) rather than radii: an f32 radius at 4 800 km holds half a metre.
- **Storage beyond f64.** `WorldPos` can become an integer grid cell plus an
  f32 offset (big_space / Unreal LWC style), or i64 fixed point, without
  touching call sites, because every site converts through `SimOrigin` and
  `FrameTree`.
- **Saves.** When a second frame can be saved in, each saved point records
  its frame (`WorldPos` serialises as `(frame, [f64; 3])`); old saves, whose
  points are the home body's frame, migrate on open by naming `HOME`.

## 10. The second pass: the herd, drops, decks, towns' floors, the line

Written before the code, as CLAUDE.md asks. After design 3-7 the owner's
jittery animals were still there: the herd was one of design 8's "not
converted" list. This pass takes that list down to what is left on purpose.

### 10.1 What was still planet-centred, and how each goes

| what | where | before | after |
| --- | --- | --- | --- |
| an animal's feet, goal | `husbandry.rs` `Beast` | planet f32: `pos += facing v dt`, then `pos = d * ground(d)` | **local f32**, stood on the ground in f64 (`with_radius`, `floor_precise`); a rebase hook moves `pos` and `goal` |
| a herd's range | `Range::Around`, `Range::Pen` | unit directions | unchanged: directions, not points; a goal is laid from them through the origin (`at`) |
| the player's feet and eye, to the yard | `World::feet`, `eye` | planet f32 | `feet_local`, `eye` local for the step and the aim; `feet` stays a query point for cells and herds |
| a drop | `pbd_core::drops::ItemDrop.position` | planet f32, pulled in planet f32 | **body-frame f64** (stored data, design 4: "a long-lived position is a `WorldPos`"); pulled in f64; drawn as `centre + position` in f64. Its save text (`{:.3}` mm) is the same bytes, read as f64 |
| a moored deck | `decks.rs` `Deck.then/now` | planet f32 frame; the heave is rounded to 0.488 mm | **local f32 frame** (`now_local`), swung about the rest frame moved into local space in f64; a hook moves `then`/`now`; the town piece's planet frame is set from it in f64 for its other readers |
| a deck under sail | `CraftDecks` | `craft_frame` from `physics_center()`, which since design 5 is **local**, but its solids were asked with planet points: a walker could not stand on a sailing cog away from the planet's centre origin (a bug design 5 brought in) | its motion is local (as it was); its solids are asked in local space (10.3) |
| a town floor, stair or ceiling | `BuildingSolids::stand`, `ceiling` | an f32 planet radius: a walker on a town floor stood on a half-millimetre stair, as on a facet before `floor_precise` | the point in the building's frame and the floor's radius in f64 (`Frame::local_f64`, `world_f64`; `stand_in`, `ceiling_in`) |
| a town wall | `holds`, `push_normal` | a planet f32 point | the point in the building's frame in f64; the wall itself is generated data in its building's f32 frame (static; it does not move or integrate) |
| the line's sag | `fish.rs` `line_points` | laid in planet points (tip and float rounded to 0.488 mm, so its end missed the drawn float by up to that) | laid in local points, lifted over the water through `with_radius`; the water is asked at query points |

### 10.2 The herd

The step is split so it can be tested on its own: `stride(beast, origin,
ground, dt)` moves the feet by `facing * speed * dt` in local space and stands
them on `ground(up)` (an f64 radius, the column's floor or the facet's
`floor_precise`) through `SimOrigin::with_radius`. Everything the step
compares (the player's feet, a goal, the flee and follow distances) is local.
A cell lookup (`World::cell`) and the ground are queries at
`query_point(pos)`. The drawing is `render_from_physics(frame, pos)`; the
up and right it builds the rig on come from `up_at`. The yard is stepped
in `Update` after the rebase (`PreUpdate`), and holds no interpolation
history (the rig is rebuilt every frame), so the hook moves only `pos` and
`goal`.

`Seen` (the showcase bot's view of an animal) keeps a planet point, from
`query_point`, because the bot reasons in planet points (`Sight`); it is a
query.

### 10.3 The pieces the walker stands on

A piece's solids are kept in one of two spaces: the towns' (planet, the
generated frames every other reader uses: rain shelter, doors, lights) and
the decks' (local, the frames the decks move). `Meets` asks each in its own
space through the origin, in f64: the walker's point is `to_world(local)` or
the local point itself, the building-frame point is `Frame::local_f64`, and
the floor's radius is the length of `world_f64` (plus the origin for a local
piece). A moored deck's piece in `Structures` stays planet for the other
readers; `Meets` takes that deck's local frame from `Decks` instead.

A walker on a deck keeps its place in the deck's own small coordinates
(`on_deck`, as before); the carry is now `now_local.world(on_deck)` in local
space, so the deck's motion reaches the walker without a planet-scale cast.

### 10.4 Drops

A drop is a few seconds to five minutes of stored state and the save names
its place, so it is a body-frame f64 point (design 4), not a local one: no
hook, and nothing moves at a rebase. The magnet pulls it in f64 toward the
chest's planet point (`to_world(chest)`). The save writes `{:.3}` of each
coordinate exactly as before (the decimal expansion of the same value), and
reads it as f64: an old save's line reads to the same text it was written
from.

### 10.5 What stays, and why

- **Terrain queries** (columns, facets, the sea, the towns' generated
  frames) answer at query points: they are asked, never integrated, and the
  answers a body stands on are f64 (`floor_precise`, the towns' f64 radii).
  Terrain at a planet thousands of times larger needs altitudes, not radii
  (design 9); not needed at the home planet's 4.8 km.
- **Pure-draw consumers that go through the render frame**: fences and
  incubators (`draw_fences`, laid relative to their first corner, then
  placed by the render frame), the towns' meshes, the vehicles (f64). They do
  not move, so they cannot jitter, and they are placed in f64.
- **The aimed ground** (`AimedGround`, from the dig's march) is a query point
  for a fence's edge or an incubator's cell.

### 10.6 Tests (the guards, extended)

- An animal stepped at 1 m/s and 8 m/s across a rebase, at 4 800 m and
  4 800 km: each step's planet-frame move is its `facing * speed * dt` to
  1e-5 m; planet-centred, it is off by > 1e-5 m (the before).
- A drop and a moored deck survive a rebase unchanged in the planet frame
  (to 1e-6 m), and the deck's next carry is continuous across it.
- A walker stands on a sailing cog's deck with the origin far from the
  planet's centre (the bug of 10.1).
- `tests/origin_guard.rs` reads `husbandry.rs`, `husbandry/breed.rs`,
  `drops.rs`, `decks.rs` too.
