# movement-smoothness: design

## 1. The instrument

`PBD_MOTION_TRACE=<csv>` (`crates/pbd-app/src/motion_trace.rs`) writes one row
per rendered frame, from `Last`, after transforms propagate, so it reads what
the frame draws. `PBD_MOTION_TRACE_SECONDS=<s>` ends the run that long after
`READY`. Each row holds:

- the frame's wall time and dt;
- the fixed physics steps run in it, and the fixed clock's overstep;
- the camera's position and rotation;
- the walker's physics `Position`;
- the float, and one fish followed by its school's seed and its own id;
- the held tool, the float and the fish in camera space, as the GPU gets them:
  an f32 view matrix (the inverse of the camera's world matrix) times an f32
  world position;
- the held tool again with the render origin rebased to the camera (snapped to
  a 16 m grid). Its world point is rebuilt in f64 from the camera and the
  pose, the origin is subtracted, and only then is it cast to f32. This is the
  A/B for cause 1: one run, one set of frames, both origins.

`tools/motion_graph.py` reads the CSV and measures, in physical pixels (a
1440x900 window at 2x, 1 800 px tall, Bevy's 45° vertical field of view):

- the held tool's **shimmer**: how far its drawn-minus-pose error moves from
  one frame to the next. A constant error cannot be seen; a changing one is
  the jitter.
- for the float and the fish, each frame's distance on screen from the mean of
  its two neighbours each side. Smooth motion leaves this near 0; a thing that
  steps while the camera glides leaves a sawtooth.
- the drawn fish's own speed, frame by frame, and how many frames it did not
  move at all.

The runs were real-time and windowed, with vsync on, on the M4 Max MacBook Pro
(the Liquid Retina XDR display, ProMotion to 120 Hz). The machine was **not
quiet**: other agents were building (load average 20 to 33), so frames ran 20
to 43 ms on average. Frame time is not the measurement here. Precision error
does not depend on the frame rate, and the stepping shows at any rate between
the physics and school clocks.

- Walk: `--world mtrace-walk --walk-distance 1000 --turn 15`
- Fish: `--world mtrace-fish --fish --fish-play --hold-pointer`

Plots and text reports: `docs/screenshots/movement-jitter/`.

## 2. Findings

### (a) Precision

- **The render origin is the planet's centre.**
  - `PlanetRenderFrame.center = body.position - PhysicsFrame.origin`
    (`planet.rs:182`). The planet is stationary at 0
    (`CelestialScene::planet_at_origin`), and `PhysicsFrame.origin` is
    `-render_offset`, zero in play. `--render-offset` is refused outside a
    static capture (`desktop.rs:1420`).
  - Physics uses the same planet-centred frame. The walking camera's
    translation is the body's `Position` plus the eye height
    (`walking.rs:1863`).
- **There is no floating origin and nothing rebases.**
- **Magnitudes:**
  - camera: 4 802 to 4 876 m from the origin in the traces;
  - `PLANET_RADIUS`: 4 800 m;
  - player, float and fish: the same order;
  - f32 step there: 2^-11 m = **0.488 mm**.
- **Where it bites: the standard mesh path.** `GlobalTransform` is f32, so a
  mesh's `world_from_local` carries the 4 800 m translation. The vertex shader
  does `clip_from_world * (world_from_local * v)`, so each vertex is rounded
  at 4 800 m and then cancelled against the view's own 4 800 m. This path draws:
  - the held tool and hand;
  - the float and the line;
  - the fish, the animals, drops and props;
  - `field_lit.wgsl`, which is a fragment extension on that same vertex path.
- **Already safe:** the terrain and water are drawn body-relative
  (`PlanetRenderFrame::camera_and_clip`, `planet.rs:193`). It subtracts the
  centre in f64 before anything reaches f32.

| run | held tool shimmer, origin at planet centre (rms / p50 / p99 px) | same frames, origin rebased to camera |
| --- | --- | --- |
| walk + turn (1 255 frames) | **2.56 / 1.95 / 6.15** | 0.0044 / 0.0035 / 0.0099 |
| fishing, standing (1 599 frames) | **1.04 / 0.71 / 2.76** | 0.0016 / 0.0012 / 0.0037 |

These are physical pixels on an 1 800 px tall view; at 1080p, multiply by 0.6.
The tool shimmers by one to six pixels against a smooth camera, even standing
still, because its sway alone moves it across the 0.49 mm grid. Rebasing takes
the shimmer off by a factor of about 600. Things drawn metres away are
displaced by the same 0.5 mm, which is a tenth of a pixel or less at that
distance, so the shimmer is worst on what is near and thin: the rod, the hand
and the line leaving the tip.

### (b) Timing

- **Physics:**
  - Avian runs at `FIXED_HZ` = 60 (`lib.rs:103`), 4 substeps;
  - no `TransformInterpolation`, so nothing is interpolated or extrapolated;
  - the walking camera follows the raw `Position` (`follow_walker`,
    `walking.rs:1848`), so it moves by whole physics steps;
  - in the walk trace, physics steps per frame were 0: 16, 1: 1041, 2: 182,
    3+: 18;
  - a sprint at 21 ms frames (16.4 to 17.9 s of the walk) shows the beat:
    the camera moved 5.4 m/s on the 54 frames with one step and 10.4 m/s on
    the 18 with two. Anything moved on the frame's own dt (fish swimming,
    animals, the float on the waves) is then seen to shake against the world
    by that difference.
  - The owner reports the camera moves fine. It stays as it is (section 3).
- **Fish schools:**
  - they step on their own clock, `spawn.step_hz` = 30 (`fauna.ron`), in `fish`
    (`fish.rs`, the `while fishery.step_s >= step` loop);
  - before this change they were drawn at the raw step;
  - the fish followed in the fishing trace **did not move in 588 of 1 594
    frames**, and its per-frame speed spread from 0.64 to 3.41 m/s
    (p10 to p90, a ratio of 5.3);
  - the fish held at the lure was placed by the same step, a step behind the
    float, which moves every frame.
- **The line, the float and the bite** (`Line::step`) run on the frame's dt in
  `Update`. The float is smooth: 0.065 px from its neighbours at p50.
- **Animals** (`husbandry::step`) run on the frame's dt. Not stepped.

### (c) Order

| system | schedule | moves | reads |
| --- | --- | --- | --- |
| `read_walking_input`, `turn_for_capture` | `RunFixedMainLoop` before the fixed loop | look (raw mouse) | input |
| `drive_walker`, `resolve_ground` | `PhysicsSchedule` (60 Hz) | walker `Position` | |
| `follow_walker` | `PostUpdate`, before propagate | walking camera `Transform` | walker `Position` this frame |
| `held::hold` | `Update` | tool pose, **a child of the camera** | |
| `fish::draw::tackle` | `PostUpdate`, after `follow_walker` and `update_planet_frame` | float, line | the camera's `Transform` this frame |
| `fish` | `Update` | line, float, schools (30 Hz) | the camera's `GlobalTransform` from last frame (the cast's start and the shy creatures' eye only) |
| `draw::schools` | `Update` | school meshes | schools |
| `husbandry::draw` | `Update` | animals | the herd |

Nothing that is drawn reads a frame-old camera. The tackle did, and it was
fixed before this change. The held tool is parented to the camera, so it moves
with it in the same propagation. `fish`'s one-frame-old camera only seeds the
cast's start and the shyness test, and neither is drawn against the camera.

### (d) Rebase

Nothing rebases. `PhysicsFrame` is advanced each tick (`publish_simulation_state`),
but for a stationary planet with no frame velocity its origin is constant, so
there is no spike to find. A floating origin (section 4) brings a rebase, and
whatever history exists then must be moved with it.

### Clocks (the f32 time question)

None of these is a large f32:

- **The sun's clock and world time:** `SimulationClock.seconds` and
  `Sun.clock.seconds`, both f64.
- **The sea's wave phases:** folded in f64 and wrapped to 2π before the cast
  (`pbd-core/src/sea.rs:285`). The GPU table is the same
  (`SeaTable::gpu`), and the float and line read the same phases.
- **The fish poses:** the clock wrapped to 3 600 s before the cast
  (`fish.rs`, `build`).
- **Session clocks:** `held::hold`'s sway and `PlanetClock` read Bevy's
  `Time::elapsed_secs()`, which is session time, not world time. Two hours in
  is 7 200 s, an f32 step of 0.49 ms, which moves a 1 rad/s sway by 5e-4 rad.
  Not visible.

## 3. The walking camera stays raw

A camera drawn at the raw physics step moves in whole steps. Easing it between
steps (the fixed clock's overstep) would cost up to one step of latency for the
position. The owner says the camera moves fine, so this change does not touch
it. The trace does show the 1- and 2-step beat (section 2b). If it is seen, on
a 120 Hz display or while walking past moving things, the fix is to
interpolate the position between the last two physics steps by
`Time<Fixed>::overstep_fraction()`, never the look, which stays raw. Its
history is kept in the physics frame, which never rebases, and it snaps on a
teleport (respawn, restore, climb-out). Recommendation: do it with the render
origin (section 4), since both write the camera's translation.

## 4. Proposed: a camera-relative render origin

**Not built.** It waits for a go-ahead.

The rule in `CLAUDE.md` is: "render and simulate collision in bounded local
f32 frames. Subtract the origin before casting". The render side breaks it,
and only the render side needs to change:

- **Physics and simulation stay planet-centred.** Over 35 sites take `up` as
  `position.normalize()`. Fish, line, animals and the walker are planet-local
  by design. Moving the physics frame would touch all of them and gain
  nothing: collision needs 0.5 mm, not 0.005.
- **A `RenderOrigin(DVec3)`.** It is the camera's planet-local position
  snapped to a 16 m grid, moved only when the camera leaves its cell. Render
  space is `planet-local - origin`. The trace's A/B used exactly this.

### Staged plan

1. **The origin, and the frame that already carries it.**
   - Set `PlanetRenderFrame.center = -origin` (an f64).
   - About 29 files already draw through `frame.center + local`, among them
     fish, line, animals, towns, drops, water, clutter and trees. They follow
     with no edit.
   - The terrain and water already subtract the centre in f64.
2. **The cameras.**
   - `follow_walker` writes `(position - origin)` (f64, then cast).
   - The same for the flight camera (`flight_view.rs`) and the vehicle cameras
     (`vehicles/view.rs`).
3. **Avian's sync.**
   - A drawn rigid body's `Transform` is `Position`, so it would land 4 800 m
     out. Turn off position-to-transform for drawn bodies and write their
     `Transform` from `Position - origin`.
   - The walker's body is not drawn. The skiff, the vehicles and the drops
     are.
4. **The audit.** Every reader of a camera or a drawn `GlobalTransform` that
   treats it as planet-local without subtracting `frame.center`. These are
   the files that query cameras today:
   `audio/mod.rs`, `clutter.rs`, `compass.rs`, `desktop/craft.rs`,
   `desktop/digging.rs`, `desktop/eye_path.rs`, `desktop/farm.rs`,
   `desktop/fruit.rs`, `desktop/kitchen.rs`, `desktop/map_screen.rs`,
   `desktop/scene.rs`, `desktop/showcase.rs`, `desktop/water.rs`, `drops.rs`,
   `field_light.rs`, `flight_view.rs`, `held.rs` (`light_held`), `loading.rs`,
   `place.rs`, `planet_lod.rs`, `planet_shadow.rs`, `planet_splash.rs`,
   `planet_water.rs`, `repro.rs`, `sea.rs`, `settings.rs`, `tone.rs`,
   `towns.rs`, `trees_taken.rs`, `vehicles*.rs`, `walking.rs`,
   `water_flow.rs` and `weather.rs`. Those that subtract `frame.center`
   already are correct; the rest are fixed or pinned by a test.
5. **The shaders.** Eleven shaders take a planet centre as a uniform:
   `clouds`, `field_lit`, `hex_terrain`, `hex_faces`, `map_live`,
   `planet_visibility`, `rain_splash`, `sky_atmosphere`, `stars`,
   `planet_surface` and `water`. Each must be fed the render-space centre,
   not 0. Any that assumes 0 is found by running the existing offset-planet
   test (`--render-offset`) in play, not only in a static capture.
6. **The rebase moment.**
   - The origin moves in `PostUpdate`, before `update_planet_frame` and
     before propagation, once per frame at most.
   - Every render transform is written from planet-local f64 in the same
     frame, so nothing is drawn half in the old frame.
   - Physics does not move, so no physics state or velocity is rebased.
   - Any interpolation history kept in render space is moved by the same
     delta.
7. **Verify.**
   - The trace's held-tool shimmer drops from 2.6 px rms to under 0.01 px.
   - `--render-offset` captures match their baseline.
   - Launch to `READY` with no `LATE_PIPELINE`.
   - `perf_suite.py` before and after: the rebase is a few f64 subtractions a
     frame.

## 5. Built here: the fish are drawn between their steps

- `School::before` keeps the positions from before the last step.
  `School::drawn(i, alpha)` lerps from there to the last step.
- `Fishery::step_alpha` is the fraction of a step the school clock has run
  since its last step.
- `draw::schools` builds every creature's body round `drawn`. A fish held at
  the lure is drawn at the lure, which `Line::step` updates every frame.
- A caught fish is taken out of `before` with the rest of its arrays.
- The simulation is unchanged: the line still reads the raw positions.
- The fish are drawn up to one 33 ms step behind the simulation. That is the
  price of drawing between steps, and nothing is aimed at a drawn fish.

**After** (the fishing trace again):

- the followed fish did not move in **106 of 1 297 frames**, down from 588 of
  1 594;
- its per-frame speed spread from 0.56 to 1.91 m/s (p90/p10 3.4, down from
  5.3), which is the fish's own changes of pace;
- its median distance on screen from its neighbours fell from 2.39 px to
  1.33 px.

**What is left, and why it is not this change:**

- The still frames that remain are almost all while a fish is hooked. There
  the float sits at `feet + away * line_m` (`Line::step`, `Phase::Hooked`).
  The scripted player (`--fish-play`) holds the button below a tension of 0.6
  and lets go above it, frame by frame. On the frames it lets go of a resting
  fish (`run` 0), `line_m` does not change and the float stands still. That is
  the bot's on-off hand, not the drawing; a held button reels smoothly.
- The hooked float hangs off the walker's raw physics `Position` (`feet`). If
  the angler moves while a fish is on, the float moves in the walker's
  physics steps. That goes with the camera interpolation in section 3.
