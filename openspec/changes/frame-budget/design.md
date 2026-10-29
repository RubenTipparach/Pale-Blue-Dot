# Design: frame-budget

## 0. Data and method

Source: `output/perf/2026-10-28-cloud-puffs/` and `...-cloud-puffs-storm/`,
which hold `<scenario>_<variant>_<n>.csv` with the `.log` beside it. The
machine is the Apple M4 Max on Metal: a 1440 x 900 window, 2880 x 1800
pixels, `--no-vsync`, the release build at `db25fe2` plus the uncommitted
`cloud-puffs`, the clock pinned at 10 h, and the first 10 s of each run
dropped. Each slice was read with `tools/frame_graph.py <csv> --log <log>`
plus short Python over the CSV columns (`wall_ms`, `rebuild_s`,
`fine_version`, `clearance_m`, `route_m`). Nothing was run on the GPU for
this write-up.

"Measured" means read off a log. "Inferred" means argued from the code and
consistent with the logs, but not isolated by an A/B run.

## 1. The storm rows are a backgrounded window

### 1a. Measured

The runs in start order, clouds off or not. "In 15-18.5" is the share of
frames between 15 and 18.5 ms; "longest" is the longest fine-set rebuild.

| Start | Runs | p50 | In 15-18.5 | Longest rebuild |
| --- | --- | ---: | ---: | ---: |
| 16:06-16:11 | all six `clouds` (march, puffs, off) | 8.9-18.4 | 12-31% | 1.3-1.4 s |
| 16:12:53 on | every `storm` and `cloud-hop` run, any variant | 16.7-17.4 | 59-83% | 1.4-9.9 s |

- The storm march variant draws 10.5-14.8 ms of measured GPU cloud and its
  p50 is 17.4 ms. Storm-off's p50 is 16.9 ms, so the frame is paced, not
  bound by work.
- `cloud-hop` has no storm forcing and flies at ~100 m, not ~430 m. It shows
  the same 60 Hz median and the same multi-second rebuilds.
- In each affected run, about 30 s after launch (storm `off_0` at 31.5 s,
  `cloud-hop_off_0` at 30.1 s) the frames ramp 26 → 73 → 127 → 224 → 356 →
  650 ms. From then on every rebuild takes 3.4-7.5 s where the same requests
  had taken 0.4-0.8 s (`fine set N landed after`). A landing frame for a
  0-column set rises from 77-85 ms to 480-580 ms. Main thread and workers
  slow down together, by about the same factor.
- The two march runs inside the window (`storm_march_1`, both
  `cloud-hop_march`) are 60 Hz paced but keep rebuilds of 1.4-1.5 s. The
  slowdown is therefore not one-to-one with the focus loss.

### 1b. Mechanism

- **60 Hz pace (from code):** Bevy 0.18's `WinitSettings::default()` is
  `game()`, with `unfocused_mode: UpdateMode::reactive_low_power(1/60 s)`
  (`bevy_winit-0.18.1/src/winit_config.rs:18-22`). `crates/` sets no
  `WinitSettings`. The affected runs' mean frame period is 17.1 ms, and a
  quiet stretch of `storm_off_0` (24.0-25.05 s) runs at 16.65 ms.
- **The 600 ms phase (inferred):** macOS App Nap throttles a background,
  occluded process: timer coalescing and lower QoS, which moves threads onto
  the efficiency cores. That slows the main thread and the rebuild threads
  alike, which is what the logs show. It is not proven. Showing it needs the
  focus log and App Nap held off (2.1).
- **An amplifier (inferred):** once a rebuild is slow the player is further
  from its anchor at the next request (`moved` 170-190 m in the focused runs,
  250-310 m after onset). The new set's bands also span the replaced
  partition (`cover_ring`, `planet_lod.rs:916-920`), so each slow build asks
  for more.

So storm and clouds were not compared like for like. The 2026-09-25 RTX 3070
baseline puts the real clouds-off storm premium at about 0.6 ms (3.6 against
3.0 ms p50).

## 2. The tail on the focused flight (`clouds`, off, runs 0 and 1)

| Frames | n | p50 | p95 | max | Over 33 ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Landing (0-2 frames after the version bump), above 200 m | 63 | 10-11 | 84-85 | 86-89 | 19 |
| Landing, below 200 m | 50 | 30 | 89-93 | 94-103 | 20-23 |
| Rebuild in flight, above 200 m | 1,107 | 8.0-8.1 | 14.5-14.8 | 21-28 | 0 |
| Rebuild in flight, below 200 m | 523-535 | 16.6-16.7 | 34-35 | 186-248 | 29-32 |
| Neither, above 200 m | 1,860 | 8.5-8.6 | 15-16 | 30 | 0 |
| Neither, below 200 m | 497-499 | 16.2-16.5 | 31.8-32.5 | 43-47 | 16-18 |
| Towns' ground install | 1 | 9,167-9,171 | | | 1 |

Without the landings and the start-up frame, the run's p95 / p99 go from
26.5 / 42.5 to 24.6 / 34.7 ms. A rebuild in flight costs nothing measurable at
altitude: 14.5 against 15 ms p95. Below 200 m the frame is slow whether or
not a rebuild is running.

### 2a. The landing hitch is the town bake

The cost of the first frames after each landing in `clouds_off_0`, against
the towns standing then (`... stands` and `... is out of range` in the log):

| Time | Towns standing | Frames after the landing |
| --- | --- | --- |
| 13-31 s | Holbrook, Linenleigh, Huatzico | 73-94 ms, then 42-48 |
| 32-35 s | Linenleigh, Huatzico | 85-89, then 3-5 |
| 36-41 s | Huatzico | 20-25 |
| 42-47 s | none | 2-14 |
| 48-50 s | Dunmouth | 31-58 |
| 51-57 s | Dunmouth, Ashenstead, then Mirhan | 72-103, then 30-50 |

Mechanism (from code). `bake_town_outsides` (`field_light.rs:376-437`) keys
each town's bake on `(Arc::as_ptr(&fine.set), light_version)`
(`:391-394`). Every landing makes a new `Arc`, so every standing town's every
vertex is re-baked: a `surface_m` (terrain noise plus the ground lock, 3c)
and a `field_at` for each. The new `UV_1` attribute is then inserted, and the
mesh is re-uploaded. `BAKE_BUDGET` is 200,000 vertices a frame (`:353`), and
a walled town is about 150,000. The table fits about 0.5 µs a vertex:
Linenleigh alone takes the first frame, the two villages the second, and once
Holbrook leaves, Linenleigh and Huatzico fit in one. Most of these bakes
change nothing. A set landed from 430 m has no columns, and a town outside
the tier reads open sky from both the old set and the new.

The render world's `fine-set upload` (`SPENT 3.0-9.2 ms`) lands in the same
frames. It is the floor left once the bake is gone.

### 2b. Near the ground

Frames with no rebuild and no landing, by clearance (`clouds_off_0`, p50 /
p95 ms):

| Clearance | 0-200 m | 200-240 | 240-320 | 320-400 | 400+ |
| --- | --- | --- | --- | --- | --- |
| p50 / p95 | 16-20 / 30-44 | 15.6 / 23 | 11.6-11.7 / 18-38 | 9.2-9.6 / 15-17 | 8.4-8.7 / 12-17 |

What turns on below each step (from code, untimed):

- **Below 200 m, in rain.** Every run's weather here is `cover 1.00, rain
  1.00`, and `rain_lod_alt_m: 200` sets the gate (`weather.rs:458`).
  - The CPU shower mesh is rebuilt every frame (`rebuild_shower`,
    `weather.rs:738-838`): up to `shower_max_streaks` 1,700 near streaks plus
    `rain_max_cell_streaks` 9,000 shaft streaks. That is about 43k vertices
    of position, normal and colour, around 1.7 MB a frame re-uploaded.
  - A `PlanetContact::sample` runs per near streak (`weather.rs:612`), and a
    `BTreeMap` of rain cells is built every frame (`:635-665`).
  - The lens drops pass draws three drop layers at full resolution
    (`water.wgsl:745`).
  - The compose pass's lens blur adds 8 taps per pixel past 28 m while the
    lens is wet (`water.wgsl:520-530, 596-601`).
- **Below the cloud base (300 m above the sea).** The rain volume marches 16
  steps at full resolution, with two `gnoise3` per step at 8 hashed corners
  each, plus a bilinear storage-buffer read of the rain map
  (`water.wgsl:1402-1462`). It runs whenever the rain map is non-zero and the
  eye is under the base (`planet_water.rs:1187-1189`).
- **Below 300 m.** Level 11 and the column tier are live (the
  `bands_are_measured_from_the_players_height` test), with trees, towns and
  the finest terrain draws.

Which of these costs what is not known. The instruments in 2.2 settle it.

### 2c. The start-up freeze

`towns::settle` (`towns.rs:668-718`) installs 46 towns' ground and calls
`rebuild_planet` (`planet.rs:353-381`) on the main thread. That takes 9.16 s.
The first build of the same work (`build_planet`, base L7 plus fine L8-L11)
took 0.81 s, so the rebuild is about 11x dearer.

Mechanism (inferred). Once ground is installed, every `surface_m`
(`pbd-core/src/column.rs:430-433`) and every `ground::top` goes through
`ground::with` (`pbd-core/src/settlement/ground.rs:443-452`). That takes a
read lock on a process-wide `RwLock` and compares the whole `TerrainConfig`
by value, on every height read, from every build thread at once. A shared
reader count bounced between 16 cores is the usual way a read lock costs more
than the work it guards. The same path is on every fine-set rebuild and every
town-bake vertex.

### 2d. The lock, measured (2026-10-28)

`towns::tests::print_the_cost_of_a_height_from_every_core` (an instrument,
`--ignored`): 400,000 heights over the sphere through `column::surface_m`,
from one thread, then from each of 16 threads at once (M4 Max, release):

| Ground installed | 1 thread | each of 16 at once |
| --- | ---: | ---: |
| none | 468 ns | 577 ns (1.2x) |
| every village (20) | 487 ns | **8,403 ns (17.3x)** |
| the towns' lookup alone (`Ground::at`, no lock) | 4 ns | |

With ground installed, sixteen threads reading heights are slower than one
thread doing all of them: the read lock's shared count serialises every core.
The towns' own lookup is 4 ns. So 2c's inference holds, and the same cost
sits under every fine-set rebuild once a world's towns stand.

**Change (4.4a, taken):** each thread keeps its own `Arc` of the installed
ground and the generation it was taken at; `install` bumps a global
generation. A height reads the generation (one atomic load, no shared write)
and takes the lock only when it has moved. The ground read is the same
`Arc<Ground>`, so every height is bit-identical.

### 2e. After the lock and the bake, measured unlocked (2026-10-28 19:00)

`output/perf/2026-10-28-frame-budget`, M4 Max, focused, screen held awake,
two runs each, interleaved; `before` is the first puffs build
(`pbd-app-puffs-v1` with its own shader), `after` has the lock (2d) and the
bake (4.3) fixes. ms:

| Scenario | Variant | p50 | p95 | p99 | p99.9 | max |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| clouds | before | 10.5 | 29.6 | 47.1 | 93.5 | 9,365 |
| clouds | after | 10.6 | 23.5 | 34.3 | 39.8 | 48.9 |
| storm | before | 11.0 | 28.7 | 47.1 | 101.9 | 9,987 |
| storm | after | 10.8 | 25.0 | 33.9 | 41.1 | 44.6 |
| walk | before | 9.3 | 17.8 | 29.3 | 87.7 | 9,691 |
| walk | after | 8.9 | 16.9 | 20.6 | 22.8 | 34.2 |

The 75-110 ms landing frames are gone (p99.9 88-102 to 23-41) and the
world-open freeze no longer reaches the measured part. Against the owner's
hard 16.7 ms (proposal), every scenario still fails. The frames still over
it, by what the frame was doing (`after` runs; "landing" is the version bump
and the two frames after):

| Run | Over 16.7 ms | Near the ground or on foot | Landing | Rebuild in flight | Aloft, quiet |
| --- | ---: | ---: | ---: | ---: | ---: |
| clouds | 16-18 % | 440-458 (median 19 ms) | 36-40 (20-24) | 22-40 | 52-100 |
| clouds, clouds off | 12 % | 410 | 38 | 3 | 23 |
| walk | 5-7 % | 1,000-944 on foot (18.4) | 24-32 | 104-137 | |

So the ground frame itself is now the budget's main enemy: a frame of
16-19 ms near the ground in rain, then the landings (20-25 ms). The rain's
parts are priced next (`PBD_PASS_OFF=rain|lens|shower`, 4.2).

## 3. The steady frame at altitude

Above 320 m with nothing landing: p50 8.4 ms, p95 15-16 ms, clouds off. What
runs there:

- The scene at 5.2 Mpx.
- The composite's full-screen passes, each a `post_process_write`
  ping-pong over the whole target: compose plus cap, and rain or lens when
  needed (`planet_water.rs:1518-1808`).
- The main thread's per-frame systems.

At Rgba16F every full-screen read plus write moves about 83 MB. Bandwidth
alone puts the few passes left at altitude well under 1 ms on this GPU
(inferred), so the larger share is likely the scene and the CPU. `gpu_total_ms`
is 0 on Metal: Bevy's pass spans need timestamps inside passes, which Apple
GPUs do not offer. Only the clouds' pass-boundary timer
(`planet_cloud_timer.rs`) reports.

Hidden by the pinned clock (`desktop.rs:911`, so `Sun::running` is false):

- `light_from_the_field` (`field_light.rs:233-316`) reads 8 corners per
  field-lit mesh every frame, each a `surface_m` plus a `field_at`, and calls
  `get_mut` on the material when the sun has moved by more than 1e-4. With
  the clock running, that is every material of every standing town, every
  frame (inferred).
- The atmosphere steps on the pool (`atmosphere.rs:487-533`) and publishes
  on the main thread.

None of this is in the suite's numbers.

## 4. The ranked plan

Each item names its evidence, its expected effect, its risk, and the
`tools/perf_suite.py` run that measures it. Every comparison is old exe
against new exe in one sitting, focused (4.1).

### 4.1 Measure a focused window (instrument; first)

- **What:**
  - Log `WindowFocused` and `WindowOccluded` changes to the frame log as
    `FOCUS` lines.
  - Under `--frame-log`, set `WinitSettings { unfocused_mode: Continuous }`.
    It is an instrument flag, so play is unchanged.
  - The suite marks a run invalid if focus was lost in the measured part,
    and holds off App Nap for the child process. Prefer an
    `NSProcessInfo` activity in the game under `--frame-log`; the suite
    printing a warning is the fallback.
  - Re-run `storm` and `cloud-hop`.
- **Why:** 1a. Without this, the next storm comparison measures the
  desktop.
- **Effect:** the storm-off row should fall to near the `clouds` row (p50
  about 9-10 ms, no 600 ms frames). If the 600 ms phase survives a focused
  run, it is a game bug, and the throttle theory is dropped in writing.
- **Risk:** none to play.
- **Measure:** `--scenarios storm,cloud-hop --variant "off||PBD_NO_CLOUDS=1"`,
  with the window kept in front.

### 4.2 GPU time per pass on Metal, and a shower switch (instrument)

- **What:**
  - Pass-boundary timestamps, as `planet_cloud_timer.rs` takes them, for
    compose plus cap, rain, lens, lens mist, and the main opaque pass.
    Write them as new frame-log columns.
  - Add `PBD_PASS_OFF=shower`, which skips `rebuild_shower`, beside the
    existing `rain`, `lens` and `mist` (`planet_water.rs:1429-1445`).
- **Why:** 2b and 3 cannot be split without it.
- **Effect:** none on its own. It decides the order of 4.5 and 4.6.
- **Measure:** `clouds` and `walk`, with variants
  `PBD_PASS_OFF=rain`, `=lens`, `=shower`, `=rain,lens,shower`, and none.

### 4.3 Bake a town only when its light can have changed

- **What:**
  - Key the bake on what it reads: the tier's light version, plus whether
    the town's bounds (with `BAKE_FRONT_M` and the ground margin) meet the
    tier's column region in the old set or the new one. A town outside both
    keeps its bake.
  - Move the vertex loop to the async pool. Its inputs are the mesh's
    positions and normals, an `Arc` of the set and of the contact. Swap the
    attribute in when the job finishes, and ease the light from old to new
    over a few frames so it never steps (priority 1).
- **Why:** 2a. 42 of 91 frames over 33 ms on the flight; 75-103 ms first
  frames with towns standing, 2-8 ms without.
- **Effect:** landing frames fall to the upload floor of 3-9 ms. The flight's
  p99 goes from 42.5 to about 35 ms (computed from the logs with the landings
  removed).
- **Risk:**
  - A real light change near a town (a dig, a lamp, dusk) must still
    re-bake. Hold that with a test: a set landing with an edit inside a
    town's margin re-bakes it, and one landing 2 km away does not.
  - Baking off-thread is derived render state, so determinism and saves are
    untouched.
- **Measure:** `clouds` and `walk` (the walk passes towns), old exe against
  new.

#### 4.3, measured and taken (2026-10-28)

`SPENT` lines round the bake, the `scenic` route by `--capture` (the first
30 s, before a locked screen's throttle could set in):

- Before: a vertex 0.43-0.47 µs; Linenleigh's 158,229 vertices **67-75 ms**,
  the next town 24-39 ms in the frame after, on **every** landing, sets of
  0 columns included. The lock fix (2d) does not touch it: the bake runs on
  the main thread alone and the height is the cost.
- Change: a town whose cap no column of the set comes within two tile widths
  of reads open sky from any set (`light::sample`'s `OPEN`), so it is baked
  open once and left; a town a set reaches is baked in slices under a 2 ms
  budget a frame into a side buffer and swapped in whole
  (`ColumnTier::reaches`, `BakeSource`, `BAKE_BUDGET`).
- After: the largest bake in any frame **3.1 ms** (median 2.3 ms of those
  over 2 ms, with a 2,048-vertex slice; the slice is now 512 for less
  overshoot). Holbrook at 21:00, old exe against new: the same frame.
- Not yet measured: the mesh re-upload when a bake's `UV_1` is swapped in
  (the whole mesh's vertex buffer, on the render thread).

### 4.4 No freeze opening a world; cheaper height reads everywhere

- **What:**
  - (a) Take the installed `Arc<Ground>` once per build job and pass it to
    the height path, or cache it per thread with a generation counter. Do
    not take the `RwLock` per read. Compare `TerrainConfig` once, by a
    digest, not per call.
  - (b) Run `settle`'s `rebuild_planet` as a pool task, the way the fine
    set is. The old planet stays drawn until the new one lands, and the
    towns stand when it does.
- **Why:** 2c. 9.16 s against 0.81 s for the same build. The same path is
  on every rebuild and every bake vertex.
- **Measured after (a), 2026-10-28:** the install rebuild **9.08 s to
  1.28 s** (`46 towns' ground installed and the planet rebuilt in`), the
  first fine set landing in 0.2 s. 1.28 s is still one frame, so (b) stays
  required by the 16.7 ms rule.
- **Effect:**
  - (a) Expected to bring the install rebuild back toward the first build's
    0.8 s, and to shorten every fine-set rebuild (logged `landed after`).
    That is inferred; measure it before claiming it.
  - (b) Removes the freeze from the frame entirely.
- **Risk:**
  - (a) is a read-path change. The heights must be bit-identical, held by
    the existing generator digest tests and a new test that a planet built
    with and without the snapshot path is byte-identical. Saved games are
    untouched.
  - (b) needs the "set begun on another planet" epoch guard the fine set
    already has (`planet_lod.rs:1238`).
- **Measure:**
  - The `planet rebuilt ... in X s` and `landed after` lines.
  - `walk` and `clouds` max frame.
  - Before (a), a measurement test: `build_planet` at 1 thread and at N, with
    and without ground installed. It names the lock's share.

### 4.5 Rain near the ground on the GPU

- **What, after 4.2 says which part costs:**
  - (a) The shower as one instanced draw. The vertex shader places streak
    `i` from `streak_seed(i)` and the clock, which is what the CPU does now,
    so nothing new is invented. It takes the ground radius from a small
    per-frame table of shaft cells, not a contact per streak.
  - (b) The rain volume's streak noise from a baked tileable 3D texture, as
    `cloud-budget` did for the cellular noise, or the volume at
    `cloud_render_scale` with the clouds' depth-aware upsample.
  - (c) The lens blur at half resolution.
- **Why:** 2b. Below 200 m the frame is p50 16 ms against 8.4 above.
- **Effect:** the ceiling is the gap, about 8 ms at p50 near the ground. How
  much is rain and how much is the near terrain is unknown until 4.2.
- **Risk:**
  - The streaks must look the same. Take before-and-after captures at 2 m
    and 150 m in rain.
  - A baked noise must tile without a seam (`cloud-budget`'s tiling test).
- **Measure:** `walk`, and `clouds` on its first and last kilometre.

### 4.6 A 3D render scale on Retina (owner's choice, with screenshots)

- **What:** render the 3D scene and the composite at a scale of the window's
  pixels (1.0, 0.75, 0.5), upsampled once, with the UI at native.
- **Why:** 3. 5.2 Mpx is four times the RTX 3070 bench's pixel count, and
  the quiet frame at altitude is over the 8.3 ms p95 already.
- **Effect:** the full-screen GPU share scales with pixel count. 0.75 is
  0.56 of the pixels (inferred).
- **Risk:** softness. The terrain's nearest-point sampling is a CLAUDE.md
  invariant and upscaling blurs it. That is why this goes to the owner with
  screenshots at each scale, and is not taken as a recommendation.
- **Measure:** `clouds` at each scale.

### 4.7 Per-frame work the pinned clock hides

- **What:**
  - A `clouds-running` suite scenario: the same route with the clock
    running (no `--time`; the start hour set by whatever the launch offers
    without pinning it). It shows the atmosphere steps and the sun-driven
    material churn.
  - Then `light_from_the_field` only for meshes that moved or whose field
    changed, with the sun term moved to a shared uniform rather than written
    into every material.
- **Why:** 3. Unmeasured, but every field-lit material is written every frame
  while the sun moves.
- **Risk:** the light must stay continuous. Make no change before the
  scenario shows a cost.
- **Measure:** the new scenario, old exe against new.

### Ruled out or deferred

- Fewer rebuild threads at altitude. A rebuild in flight costs nothing
  measurable above 200 m (14.5 against 15 ms p95), so this is not where the
  tail is.
- Loading terrain less often, or coarser near bands. That trades detail
  near the camera for frame time, and invites pop-in (priority 1).
