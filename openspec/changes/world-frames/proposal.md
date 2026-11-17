## Why

The owner, 2026-11-17, on the walk trace after the render origin
(`movement-smoothness` design 6): "its not the camera thats the problem, the
math still shows: 4436.446780", and "think about it, your translations still
have limited precision". Then: "it is extremely important to build this for a
solar system scaled world", "we will be walking on planets, flying spaceships
around the solar system", "build it for the future".

The owner was right and `movement-smoothness` design 4 was wrong. It kept
"physics and simulation planet-centred ... collision needs 0.5 mm, not 0.005".
But the simulation does not only *collide* at the planet's scale, it
*integrates* there: every `Position += v dt` of the walker, every fish's step,
the float's bob, is stored as an f32 about 4 800 m from the planet's centre,
where an f32 holds 0.488 mm. The render origin subtracts after the rounding;
it cannot put back what the store lost. Measured (design 2): a 1 m/s walk's
16.7 mm steps are off by 0.11 mm at p50 and 0.91 mm at worst, 1.5% of the step
on average; a fish's 30 Hz step is off by 0.23 mm; the float's bob by 0.38 mm.
That is the owner's jitter, and it is the CLAUDE.md invariant broken: "render
**and simulate collision** in bounded local f32 frames. Subtract the origin
before casting."

## What Changes

- **A frame tree in `pbd-core` (`world_frame`)**: the system frame, body
  frames (inertial and turning), vessel frames that can hang from any frame;
  each with an f64 pose over time; exact conversion of a position and a
  velocity between any two frames, through their lowest common ancestor,
  with every turning frame's `ω × r`. Built and tested now, for the solar
  system the game is heading to; the game has one body in it today.
- **`WorldPos { frame, pos: DVec3 }`**: a long-lived position names its frame.
  There is no `From`/`Into` between it and a local `Vec3`.
- **`SimOrigin { frame, origin }`**: the local origin Avian, the simulation and
  the renderer share. Local positions are f32 relative to it. Its methods are
  the only conversions: `to_world`, `to_local`, `up_at`, `radius_at`,
  `with_radius`, `at`, `query_point`. A rebase moves it by whole grid cells
  (16 m on foot, grown with speed), and a handoff moves it into another frame,
  converting positions, velocities and orientations together.
- **The home planet's simulation runs in local space**: the walker (Avian
  body, ground resolution, climbs, decks), the skiff (Avian body, terrain
  protection, route and tour), the fish schools and the line. The origin
  follows the player, rebasing at a tick boundary; a rebase is O(simulated
  things): Avian bodies plus registered hooks. It moves no render transform:
  the render origin moves with it.
- **Guards**: the types; `ORIGIN_LEAK <site> <value>` in a debug build for a
  local value past 2 km; guard tests at the planet's radius, at a thousand
  times it, and on a body 1 AU out; a source check that fails on a new
  `position.normalize()`-style "up" outside the origin.
- **Measurement instruments**: `PBD_STEP_TRACE` (one row per physics step) and
  `tools/step_graph.py`; `PBD_WALK_AXIS_SCALE` for a slow scripted walk;
  `PBD_HOLD_SIM_ORIGIN=1` holds the origin at the planet's centre for the A/B.

Not in this change (design 8): ship gameplay, other bodies' content,
system-scale drawing of distant bodies, the vehicles' own simulation (already
f64), decks as vessel frames, and terrain queries as altitudes. The herd,
drops, decks (in local space), the towns' floors (f64) and the line's sag
follow in a second pass (design 10). Saves keep their format (the pose is the home body's frame).

## Capabilities

### Added
- `platform/world-frames`: frames, the local origin, rebase and handoff, and
  the guards.

### Modified
- `movement-smoothness` design 4 and 6: the claim that the simulation may
  stay planet-centred is withdrawn (its design 9).

## Impact

- `crates/pbd-core/src/world_frame.rs` (new), `world_frame_guards.rs` (tests),
  `fauna/school.rs`, `fishing.rs`, `orbit.rs` (`PartialEq`).
- `crates/pbd-app/src/sim_origin.rs` (new), `step_trace.rs` (instrument),
  `walking.rs`, `flight_view.rs`, `fish.rs`, `planet_contact.rs` (an f64
  floor), `motion_trace.rs`, `drops.rs`, `towns.rs`, `vehicles/place.rs`,
  `desktop.rs`, `desktop/{map_screen,missions,showcase}.rs`.
- `tools/step_graph.py` (instrument); `crates/pbd-app/tests/origin_guard.rs`.
- Saves: unchanged on disk; every saved point is converted at the boundary.
- Frame cost: a rebase is a few dozen subtractions every 16 m on foot.
  Not measured with `perf_suite.py` here (design 7).
