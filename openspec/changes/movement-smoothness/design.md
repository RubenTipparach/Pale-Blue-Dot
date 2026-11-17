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

## 3. The walking camera stays raw (superseded by 6.5)

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

## 4. A camera-relative render origin

Proposed here; the owner's go-ahead came on 2026-11-17 ("Build floating
origin"). How it is being built, stage by stage, is section 6.

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

## 6. Building the floating origin (owner, 2026-11-17: "Build floating origin")

Built on `claude/floating-origin`, in this order, each stage measured before
the next. Sections 4.1 to 4.7 are the plan; this is what is built and the
decisions taken building it.

### 6.1 Three spaces, named

- **Physics space**: Avian's `Position`, and the walker's and the skiff's own
  `Transform` (Avian writes it). Planet-centred in play: the planet's centre
  there is `body.position - PhysicsFrame.origin`, 0. Unchanged.
- **Planet-local**: relative to the planet's centre. What every simulation
  stores (fish, line, herd, towns, drops) and what `up = p.normalize()` needs.
- **Render space**: every other `Transform` and `GlobalTransform`, the views
  and everything drawn. `render = physics - origin`, with `origin` a
  `DVec3` in physics space.

`PlanetRenderFrame` carries both: `center`, the planet's centre in render
space (`physics centre - origin`), and `origin`. `frame.physics_center()` is
`center + origin`. So the rules are:

- drawn (render space) from planet-local: `frame.center + local` in f64, then
  cast;
- planet-local from a drawn transform: `translation - frame.center` in f64;
- planet-local from an Avian `Position`: `position - frame.physics_center()`;
- render space from an Avian `Position`: `position - frame.origin`.

Every site that used `center` for a `Position` (`fish.rs`, `husbandry.rs`,
`vehicles.rs`, `audio/hooks.rs`) moves to `physics_center()`.

### 6.2 The origin and the rebase (stage 1)

- `RenderOrigin` lives in `PlanetRenderFrame.origin`. The camera's
  planet-local position is snapped to a 16 m grid (`ORIGIN_CELL_M`): a
  rebase happens when the camera leaves its cell, at most once a frame. At
  16 m the f32 step of anything within 32 m of the eye is 2^-19 m (2 µm).
- `rebase_render_origin` runs in `PostUpdate`, after every camera writer
  (walking, flight, vehicle, capture), and before `update_planet_frame` and
  transform propagation. It reads the active 3D camera's `Transform`, gets
  its planet-local point as `translation - center` in f64, and when the
  snapped cell differs:
  - `origin += delta`, `center -= delta`;
  - every **root** `Transform` that is not a physics body (no `RigidBody`)
    is moved by `-delta`. Entities spawned once at a place (towns, props,
    drops, lamps, the stars) stay where they are in the world without any of
    them knowing; entities rewritten each frame from planet-local data are
    right whether they were written before the rebase (and moved by it) or
    after it (with the new centre). `delta` is a multiple of 16 m, so the
    f32 subtraction is exact for every coordinate in range and repeated
    rebases never drift.
- `update_planet_frame` computes `center = body - PhysicsFrame.origin -
  origin`, so the two agree every frame.
- Physics and simulation do not move. No `Position`, velocity or simulation
  history is touched by a rebase.

### 6.3 The cameras (stage 2)

The walking (`follow_walker`), flight (`follow_flight_camera`) and vehicle
(`vehicles/view.rs`) cameras write `position - origin` (f64, then cast). The
capture cameras spawned at startup are placed in render space at the origin
they start with. Measured: held-tool shimmer from `PBD_MOTION_TRACE`, walking
and fishing, real time; the target is under 0.05 px rms.

### 6.4 Everything else drawn (stage 3) and the shaders (stage 4)

The audit of sections 4.4 and 4.5, file by file, with each converted
explicitly: per-frame writers from physics space (the moon, the skiff, the
vehicles' crew) and readers that took a drawn translation as planet-local
(`normalize()` as up, `length()` against the radius or the sea, cell
lookups, the audio listener). The eleven shaders already take the centre as
a uniform; each feeder is checked to write the render-space centre, and any
that sampled absolute world position is moved to planet-local so nothing
shifts on a rebase. `--render-offset` becomes a render-space offset that
works in play (6.6) and is the test of all of it.

### 6.5 The walking camera between physics steps (stage 5)

The owner's ask (2026-11-17) brings section 3's option in with the origin:

- the walker keeps its physics `Position` at the last two fixed steps
  (`StepHistory`), recorded after each step in physics space, which never
  rebases;
- `follow_walker` places the eye at `lerp(previous, current,
  Time<Fixed>::overstep_fraction())`, then subtracts the origin;
- the look is not touched: it stays this frame's raw mouse;
- a `Position` that changed outside the fixed loop (respawn, restore,
  climb-out, boarding), or a step longer than 2 m, snaps the history to it,
  so a teleport is never drawn as a glide.

The cost is up to one physics step (16.7 ms) of position latency, the price
of drawing between steps. A `--capture` runs one physics step a frame and
has no beat to smooth, so it draws the step itself (`DrawBetweenSteps`):
drawn between steps, a capture showed the body a step behind, 105 000
pixels of a still moved (found by the held-origin check of 6.7).

### 6.6 `--render-offset` in play, tests and checks (stage 6)

- `--render-offset x y z` no longer moves physics. It puts the render origin
  that far from the camera's cell, so the planet's centre and the camera are
  both away from 0 in render space. It is allowed in play.
- Tests: a camera far from the centre with origins at 0, at 4 800 m and at a
  random offset derives the same planet-local eye, up and clip; a rebase
  keeps every drawn point where it was (no jump); a walker crossing a cell
  draws continuously.
- Launch to `READY` with no `LATE_PIPELINE`.

### 6.7 No look change

Same-frame `--capture` stills before and after (`tools/capture_repeat.py
--against`) at a town by day and at night with lamps, the sea, under water, a
waterfall and flying at 1 km. A pixel difference beyond the repeat noise is a
bug.

### 6.8 Results (M4 Max MacBook Pro, 2026-11-17)

Real-time windowed traces, walking (`--walk-distance 1000 --turn 15`) and
fishing (`--fish --fish-play --hold-pointer`), the build before against this
one, each on a fresh world; plots and reports in
`docs/screenshots/floating-origin/`. The machine was not quiet (other agents
building, another game running), so frame time is not the measure.

| held-tool shimmer (rms / p50 / p99 px, 1 800 px tall) | before | after |
| --- | --- | --- |
| walking (1 342 / 1 247 frames) | 2.60 / 2.02 / 6.23 | **0.0046 / 0.0036 / 0.0118** |
| fishing (1 549 / 1 490 frames) | 1.33 / 0.94 / 3.23 | **0.0022 / 0.0016 / 0.0055** |

The target was 0.05 px; the trace's own A/B (the same frames re-rebased in
f64) agrees with what is drawn to a thousandth of a pixel. The camera sits
at most 12 m from the origin, where an f32 holds 1 µm.

Camera speed while walking, by physics steps in the frame:

| | 1 step | 2 steps |
| --- | --- | --- |
| before (raw step) | 10.3 m/s | 16.4 m/s |
| after (between steps) | 8.2 m/s | 7.9 m/s |

The beat is gone. On the 11 frames with a rebase in the walking trace the
camera moved by its walk (6 to 74 cm, in proportion to the frame's time), so
nothing jumps across a rebase.

**Stills** (`tools/capture_repeat.py --against`, 300 frames, GPU, 2880 x 1800;
pixels differing by more than 1/255):

| scene | before vs before (noise) | after vs before | after with the origin held (`PBD_HOLD_ORIGIN=1`) vs before |
| --- | --- | --- | --- |
| Ashenstead by day (walk) | 0 | 58 261 (1.1%), max 77 | 0 |
| Ashenstead at night, lamps | 0 | 63 254 (1.2%), max 160 | 0 |
| Holbrook lane at night | 0 | 5 905, max 221 | 16 (the noise) |
| harbour from 30 m | 0 | 109, max 14 | 0 |
| under water | 0 | 0 | 0 |
| waterfall (walk) | 16 | 53 | 16 |
| flying at 1 km | 0 | 7 | 0 |

Every difference is an edge or a texel boundary moved by under a pixel, and
the held-origin column shows they are the origin alone: with the origin held
at the planet's centre and every conversion still running, the stills match
the old build pixel for pixel. The old view matrix carried the camera's
4 800 m translation in f32 and so drew the eye up to half a millimetre off;
that error is the jitter, and a still taken at one instant shows it as a
sub-pixel shift of near geometry (the town walls are a metre from the eye).

`--render-offset 8192 -4096 2048` in play: a walk to `READY` with no
`LATE_PIPELINE`, the camera drawn round the walker as without; the held
tool's shimmer comes back (4.2 px rms at 9.4 km from the origin), which is
the offset being applied. Stills with the offset: under water 0 pixels
against no offset, flying at 1 km 2 115 (near terrain edges at the coarser
precision). It found one bug: the flight and walking cameras were spawned
in physics space, and a static flying capture never moves its camera.

Launches to `READY`: the four real-time traces and the offset walk, no
`LATE_PIPELINE`.

**Frame cost: owed.** The machine was not quiet; `perf_suite.py` before and
after is to be run on a quiet machine. A rebase walks every root transform
once per 16 m cell (every few seconds on foot) and marks them changed, so
that frame re-propagates and re-uploads them; the field-lit rows' world
bounds move too and are rewritten (`field_light.rs`).

### 6.9 Not converted, or left as they are

- The hooked float hangs off the walker's raw physics `Position`
  (`Line::step`), so while a fish is on and the angler walks, the float moves
  in physics steps against an eye drawn between them. The line's simulation
  is not touched here.
- The vehicle cameras and the flight camera are not drawn between steps (a
  craft is stepped on its own clock; the skiff is an Avian body).
- `desktop/scene.rs` stars are spawned once at the frame's centre and move
  with the rebase like any root.
- Bodies under Avian (the walker, the skiff) keep physics-space transforms;
  neither has a drawn child. A drawn rigid body would need its own render
  transform.

## 7. The hooked float and the line (owner, 2026-11-17)

The owner, on the fish before/after videos: "I'm noticing that the fish line
is not all smooth, it jumps around a bit". Measured on `claude/floating-origin`
(`787964cb`) with the fishing trace of section 1 (`--fish-play
--hold-pointer --time 10`, 45 s, one hook played out), the float on screen by
phase (px from the mean of its two neighbours each side, 1 800 px tall):

| phase | frames | rms | p50 | p99 |
| --- | --- | --- | --- | --- |
| floating | 1 183 | 0.64 | 0.018 | 0.086 |
| hooked | 733 | **14.4** | 1.49 | **86.7** |

The float is smooth while it waits. While a fish is on it is not, for two
reasons, both found frame by frame in the trace's planet-local float:

### 7.1 The swing switches on and off in one frame (the jumps)

`Line::step` (`pbd-core/src/fishing.rs`, `Phase::Hooked`) puts the float at
`feet + away * line_m`, with `away` the fish's heading turned by a swing of
`sin(clock * 1.7) * 0.12 * run`. `run` is the fish's pull while it runs and
0 while it rests, and it changes in one step when `run_s` runs out (and at
the strike, from nothing). So the swing angle drops to 0 or jumps to its
value in one frame, and the float with it, sideways across the water: **1.47,
1.19, 1.06 m ... 0.26 m in one frame** in the trace, at 14 m to 4 m of line,
with the float's distance from the angler unchanged. The line's far end and
the fish drawn at the lure jump with it. These are the 50 to 120 px spikes.

The rule is the simulation's (the float's point decides the landing and the
line's length), and is not changed here. The drawing takes the jump out:

- a draw-only `DrawnFloat` keeps an offset from the simulated float;
- on a frame where the line was hooked last frame and is hooked now and its
  `run` changed, or the frame the line goes from a bite to hooked, the
  offset takes up the whole of that frame's move (the float is drawn where it
  was), and from there it eases to nothing, critically damped (no kick in
  speed at either end), over about a third of a second
  (`FLOAT_EASE_PER_S` 14/s). The swing goes on at the line's first hooked
  step, the frame after the strike, so that frame is taken up too;
- the offset is kept along the water: the drawn float stays at the simulated
  float's height over the planet's centre;
- in every other frame and phase the offset only decays, and the line's
  leaving the water (caught, snapped, wound in) or any other phase clears it;
- the float, the line's far end and the hooked fish (drawn at the lure) all
  use the drawn float, so they stay together.

The cost is that for a third of a second after a run starts or stops the
float is drawn up to a metre from where the line's rules have it. Nothing
reads the drawn float.

### 7.2 The scripted hand flickers the button (the stutter)

Between the jumps, the float moved 3.5 cm, 3.5 cm, 0, 3.5, 3.5, 0 ... a 20 Hz
stutter (`hooked` p50 1.5 px, against 0.018 floating). That is the
`--fish-play` bot of section 5: it presses below a tension of 0.6 and lets go
above it, deciding every frame, so it holds two frames and lets go one. A
player holds the button down and lets it up. The bot now does the same, with
a band (hold from under 0.45 until over 0.75), so a video shows the reel a
player's hand would give. That is the instrument; the game's controls are not
touched.

### 7.3 The line leaves the rod's tip at rest, not the swayed rod

`draw::tackle` starts the line at `held::rod_tip()`, the tip of the rod in its
rest pose, but `held::hold` sways the rod by up to 6 mm right and 5 mm up
(`held.ron`). The rod's tip is 0.73 m in front of the eye, so the line's
start and the drawn tip drift up to 20 px apart and back over about four
seconds. The trace gets a column for it (the line mesh's start and the
swayed rod's tip, in camera space, as drawn). The line now starts at the
held rod's tip as posed this frame: its `Transform` (written in `Update`,
before `tackle` in `PostUpdate`) times its rest pose's inverse, applied to the
rest tip. The cast still leaves from the rest tip (`Angler.tip`, simulation).

### 7.4 Left as it is

- The hooked float hangs off the walker's physics `Position` (6.9); the bot
  stands still, so the trace does not show it.
- The cast's first frames are the float thrown at 15 m/s from the tip; their
  residual is the throw's own acceleration, not a step.

### 7.5 Results (M4 Max MacBook Pro, 2026-11-17)

The same 45 s fishing trace, real time, windowed, vsync on, each on a fresh
world; one fish hooked and landed in each. "Before" is this branch with the
drawing changes switched off in a local build (the trace's new columns need
the instrument); its hooked numbers match the run on `787964cb` itself
(rms 14.4, p99 86.7 px). Reports and plots: `docs/screenshots/fishing-line-smooth/`.

| | before | after, same scripted hand | after, steady hand (7.2) |
| --- | --- | --- | --- |
| hooked float jitter, rms / p50 / p99 px | 19.0 / 1.49 / 131 | 4.5 / 1.69 / 18.1 | **2.9 / 1.06 / 11.3** |
| hooked frames over 20 px | 38 of 793 | 6 of 790 | **0 of 788** |
| largest float move in one frame | 152 cm | 20 cm (an eased slide) | 19 cm (an eased slide) |
| line start from the drawn rod tip, rms / max px | 16.3 / 23.6 | **0.31 / 0.65** | 0.31 / 0.63 |

The 6 frames left over 20 px with the old hand are its 20 Hz stutter with
the float 4 m out; the steady hand takes them. What is left at p99 is the
reel starting and stopping, which a player's hand does too. The 0.3 px left
between the line's start and the rod tip is the rod model's tip against
the line's first vertex, constant, not a motion.

### 7.6 The bite from under the water (`--watch-hook`)

A recording instrument: with `--fish`, when the float lands an eye is put
under the water 0.85 m beside the lure, a little under it, and looks at it
for the rest of the cast; the HUD, the readout and the rod in hand are
hidden. Only the drawn frame moves: the walking camera is put back in
`First`, so the line, the cast and the creatures see the angler's eye, and
the line is placed from the rod before the view moves. The before build
(`d0f101b3`) got the same instrument as a local commit only.

Recorded with OBS (real time, windowed, `--no-vsync`, 60 fps), one file:
`fish-bite-underwater-before-after-2026-11-17-1080p60.mp4` in the
validation folder on Drive (before, after, then side by side aligned on the
strike, in real time and at a quarter speed). Read frame by frame:

- Both builds show the fish come to the lure, nibble and take it under the
  float. The game draws no hook: the fish noses an empty point 0.35 m under
  the float (`Line::lure`). The two runs hooked different fish (a pike-like
  fish before, a ray after): the line's draws differ.
- Before: about 0.15 s after the strike the float and the fish jump about a
  metre in one frame (the swing of 7.1), then race off.
- After: the float holds at the strike and glides off over the next frames;
  no frame jumps.
- In both builds the fish on the line is drawn standing on its tail under
  the float for 6 to 8 frames at the strike, then snaps level. That is its
  pose (built from its velocity), not this change; it is left for a
  follow-up.
