# Design: crab-breeze-lessons

## Context and method

The two games share an owner, a machine, an engine and a measuring habit.
Both use Bevy and Avian, both use `perf_suite.py`-style interleaved
sittings, and both run on the RTX 3070 desktop. Crab Breeze is on Bevy
0.20.0 and Avian 0.8.0 (`Cargo.toml:14`, `crates/cb-app/Cargo.toml`).
Pale Blue Dot is on Bevy 0.18.1 and Avian 0.6.1 (`Cargo.toml:12-13`).

This is a reading, not a sitting. Another agent was benchmarking on the
desktop, so nothing was built or run. The labels are the ones
`lighting-perf` uses:

- **Measured**: a number from a checked-in report.
- **Read**: a fact from the code. Crab Breeze references are to `main`
  `8f9ed158`, as `CB path:line`. Pale Blue Dot references are to `86361b1`,
  as `PBD path:line`.
- **Estimated**: neither, and said so every time.

Crab Breeze reports used:

- `docs/benchmarks/2026-10-25-desktop/report.md`: the world, fight and
  defence takes, with parts switched off.
- `docs/benchmarks/2026-10-26-free-roam/report.md`: the height cache.
- `docs/benchmarks/2026-10-28-city/report.md`: the capital, video memory,
  and the flights.
- `openspec/changes/night-frame/design.md`: the suburbs at noon and at
  midnight.
- `openspec/changes/city-frame/design.md`: warm-up, late pipelines and
  shadow ranges.
- `openspec/changes/gpu-instancing/proposal.md`: a trace of the render
  world's per-mesh costs.

Pale Blue Dot reports used:

- `docs/benchmarks/2026-11-12-lighting-model-clean/report.md`.
- `openspec/changes/lighting-perf/design.md`.
- `openspec/changes/town-frame-time/design.md`: the trace of the town walk.

## 1. Apples to apples: draws less, or does the same work more cheaply?

### The frame side by side (measured)

| | Crab Breeze | Pale Blue Dot |
| --- | --- | --- |
| Window | 1920 x 1080 (2.07 Mpx) | 1440 x 900 (1.30 Mpx) |
| Open-world p50, vsync off | 9.2-9.5 ms (world take, 10-25); 11.7-12.8 ms (east, flying, 10-28) | 9.8 ms (walk), 9.9-10.1 ms (clouds, storm), 6.7 ms (far-side) |
| GPU p50 there | 1-1.5 ms (free roam, 10-26); 1.5-2.5 ms (suburb, night-frame) | 5.3 ms (walk), 5.5 ms (clouds), 9.6 ms (rain-stroll) |
| Densest scene p50 | 13-15 ms (suburb, CPU-paced); 27-42 ms (capital, video memory spill) | 17-18 ms (town walk, `main`); about 9 ms (`town-frame-time`) |
| What limits it | CPU: the render world's per-mesh work, then shadows | GPU in rain and in flight; CPU in towns |

### What follows

- **The GPU is where they differ.** Pale Blue Dot's GPU frame costs about
  3-8 times Crab Breeze's per pixel. Crab Breeze has more pixels and far
  more meshes (2,000-8,000 drawn against Pale Blue Dot's few hundred town
  meshes plus indirect-drawn terrain), and still uses less GPU time.
- **The CPU is where they agree.** In each game's open world the uncapped
  frame is 9-10 ms. In each game's densest place it is CPU-paced, at 13-18
  ms. Crab Breeze got there through the techniques in section 3.

### Why Crab Breeze's GPU frame is small (read)

Each line is a picture choice, not an engine trick.

| Part | Crab Breeze | Pale Blue Dot |
| --- | --- | --- |
| Clouds | One mesh of camera-facing quads, one draw, re-sorted only when the eye has moved `resort_m` (`CB crates/cb-app/src/clouds_render.rs:1-21`, `:429-448`) | A volumetric march at 0.4 scale with a reprojected history and a depth-aware upsample (`PBD assets/config/weather.ron:146`, `planet_water.rs:186-203`) |
| Sky and haze | Bevy's `Atmosphere` with its lookup tables (`CB pilot.rs:226`) | A sky shell material and an SH9 probe (`PBD sky.rs`, `tone.rs:69-90`) |
| Fog | An analytic term in the water compose pass (`CB water_render.rs:14-16`) | Lit rain fog through shells, looping the lamps (`PBD water.wgsl`) |
| Reflections | A sky cube filtered once at load, then the generator removed (`CB sky_light.rs:11-17`, `:116-117`) | L8: a voxel walk and a screen trace on wet ground, a screen trace on the sea, and a 960 x 600 capture every frame (`PBD planet_ssr.rs:36`, `planet_surface.wgsl` ~1961-2057) |
| Anti-aliasing | `Msaa::Off` and FXAA (`CB pilot.rs:149-150`, `:193-194`) | `Msaa::Sample4` on an HDR target (`PBD desktop.rs:1655`) |
| Terrain shading | Bevy `StandardMaterial` and texture arrays | A 2,500-line fragment shader with the voxel light field, contact shadows and L8 (`PBD assets/shaders/planet_surface.wgsl`) |
| Occlusion | Bevy's GPU occlusion culling and a depth prepass (`CB pilot.rs:204-205`) | A custom GPU horizon cull and indirect draws (`PBD planet.rs:1659-1667`). This is already GPU-driven, and is ahead of Crab Breeze here |

**Verdict.** Crab Breeze is mostly faster because it draws less per pixel.
Matching that would mean changing Pale Blue Dot's look, and this change does
not propose it.

### Where Crab Breeze really does the same work more cheaply

1. Presenting with a deeper queue (section 2).
2. Batching many meshes under one material (rank 2).
3. Not recomputing what has not changed (ranks 3-5).
4. Keeping heavy rebuilds off the main thread (rank 6).

## 2. The half-resolution question (owner's suggestion)

- **Crab Breeze: no pass of its own runs at reduced resolution.** A search
  of its crates and shaders for downsample, upsample, half-res, quarter-res
  or a render scale finds only mip generation and Bevy's environment-map
  filter (`CB pbr_set.rs:25-26`, `sky_light.rs:161`). Its bloom is Bevy's
  own `Bloom` (`CB sun_flare.rs:126-141`), which works on a mip chain. Its
  fog is analytic and cheap at full resolution. So there is nothing there
  to port.
- **Pale Blue Dot already reduces its expensive passes.**

  | Pass | Resolution now | Measured cost |
  | --- | --- | --- |
  | Clouds | 0.4 scale, history-blended, depth-aware upsample (`PBD config.rs:673`, `planet_water.rs:1386`) | 0.40 ms p50 GPU in storm and clouds |
  | Reflection capture | 960 x 600 (`PBD planet_ssr.rs:36`) | not split from L8 yet |
  | Bloom | from a quarter-size image (`PBD bloom.rs:5`, `:331`) | 0.33-0.36 ms; tone, grade and bloom together 0.4 ms |

  Halving bloom again could save 0.1-0.2 ms at best (estimate), and
  `lighting-perf` rank 7, the bloom fold, already targets the same 0.1-0.2
  ms.
- **The one expensive full-resolution pass is L8's traces.** They cost
  2.2 ms p50, 3.7 ms p95 and 8.2 ms p99 on `rain-stroll`, and run per
  fragment inside `planet_surface.wgsl` and `water.wgsl`. Running the
  reflection trace in its own pass at half resolution would be the
  half-resolution step worth taking. It would write reflected colour and
  hit distance, then upsample through the clouds' depth-aware upsample,
  which keeps edges against depth.
  - Expected: about 40-60 % of whatever L8 costs after `lighting-perf`
    ranks 1-3 land (estimate).
  - Look risk: puddle edges, so it needs shots.
  - Order: it waits for `lighting-perf` 2.1-2.3, because the early exits
    may leave too little to be worth a new pass.

## 3. Ranked techniques worth porting

Rank is the measured cost the technique addresses, times the share an
estimate expects it to remove. The first column is the rank. Each line names
its evidence in both repositories and how it will be measured. All A/Bs use
`tools/perf_suite.py`, interleaved, two or more repeats, against the exe
before the change. They follow the `perf-measure` skill and CLAUDE.md:
nothing heavy runs beside them, and reports go in `docs/benchmarks/`.

### 1. Frame latency 1 against 2: measure it

- **PBD (read).** `desktop.rs:849-855` asks the swapchain for
  `desired_maximum_frame_latency = 1` on every run that is not a
  `--capture`. That includes every `perf_suite.py` run, which passes
  `--no-vsync` and not `--capture` (`tools/perf_suite.py:290`). It was set
  in the first prototype (`f958f08`). The documents describe it as "a queue
  hint, not a measured end-to-end latency guarantee"
  (`voxel-engine-foundation/design.md:99-103`,
  `docs/flight-controls.md:22-24`), and it has never been measured.
- **CB (read).** The window sets a present mode and no frame latency
  (`CB main.rs:146-176`), so it gets wgpu's default of 2.
- **PBD (measured).**
  - The town walk's trace puts `prepare_windows`, the swapchain wait, at
    2.4 ms of the render thread's 8.1 ms (`town-frame-time/design.md`,
    "Measured: the trace of a 100 m town walk").
  - In the lighting sitting, wall p50 sits 4-5 ms above GPU total in every
    variant of `storm` and `walk` (`lighting-perf/design.md`, "Where it
    goes").
- **Mechanism (read, not proven here).**
  - With a queue of one, the render world cannot acquire frame N's image
    until the GPU has finished frame N-1. So render-thread work after the
    acquire (queue, encode, submit) runs back-to-back with the GPU instead
    of beside it.
  - With a queue of two they overlap, and the frame approaches the larger
    of the CPU and GPU times instead of part of their sum.
  - Wall time 2x GPU time with a GPU-heavy picture is the shape that
    predicts.
- **Expected (estimate).**
  - 1-3 ms p50 in the GPU-heavy scenarios (`rain-stroll`, `storm`,
    `clouds`), and less in the CPU-bound `town`.
  - Fewer missed refreshes with vsync on, because a frame near 16.7 ms
    stops falling to 30 fps.
  - The price is up to one more frame of input-to-photon latency. Mouse
    look stays raw and unsmoothed (CLAUDE.md). Only the queue changes.
- **Measure.**
  - Instrument first: a `--frame-latency 1|2` launch flag, with the value
    written in the frame log's header. The default stays 1.
  - Then run `rain-stroll`, `storm`, `walk` and `town` with `--variant
    "lat1|<exe>|--frame-latency 1" --variant "lat2|<exe>|--frame-latency
    2"`, using the same exe and three repeats. Do it once uncapped and once
    with vsync on, counting refreshes missed.
- **Decision.**
  - If latency 2 saves more than the spread, the default becomes 2, and
    `--frame-latency 1` stays for players who prefer the lower latency.
    This is a feel change with nothing to screenshot, so it is a
    "recommendation taken (ask only with screenshots)". The owner judges
    the feel on the next gate.
  - If the A/B shows no saving, the finding is recorded and the setting
    stays.

### 2. Town meshes: one shared material, per-mesh data in a storage buffer

- **PBD (read).**
  - Every field-lit mesh owns a `FieldLitMaterial`
    (`field_light.rs:133`, `:197`).
  - Its `FieldUniform` is about 1.1 kB (`field_light.rs:83-110`). Most of
    it is the same for every mesh: `sun`, `centre`, `model` and the 18-vec4
    `sky_sh`.
  - Any change past a threshold rewrites the asset
    (`field_light.rs:384-399`). Each rewrite makes Bevy prepare the
    material again: a new uniform buffer and a new bind group.
- **PBD (measured).**
  - 577 such meshes stand in the town walk (`town-frame-time/design.md`,
    "Measured: the town walk").
  - `prepare_erased_assets<FieldLit>` plus `prepare_material_bind_groups`
    takes 1.45 ms, and Extract takes 1.45 ms, of the render thread
    (`town-frame-time/design.md`, trace table).
  - Decision 7 there cut the rewrites. It did not cut the per-mesh
    materials, which "also means nothing batches" (`:34`).
- **CB (read and measured).**
  - **Building texture arrays**: a building's opaque parts are one mesh and
    one material over three texture arrays (`CB building_arrays.rs:1-7`,
    `openspec/changes/building-texture-array`).
  - **Grass and people**: one shared mesh per level and one material, with
    each tile's or person's data in one storage buffer indexed by
    `MeshTag` (`CB grass.rs:1-13`, `:338`; `people_vat.rs:1-10`, `:353`).
    "Chunks of a level batch into one draw, so the draw count is fixed
    whatever the crowd."
  - The trace that motivated this (`CB gpu-instancing/proposal.md`): the
    render world spends its time per mesh entity per view. Material
    specialization took 2.2 ms, queueing 1.1 ms and shadow specialize and
    queue 1.4 ms.
- **Change.**
  - Split `FieldUniform` in two:
    - The view-wide part (sun, planet centre, model, SH sky) goes in one
      extracted uniform, written once a frame.
    - The per-mesh part (field corners, look, room lights) goes in a row of
      one storage buffer, indexed by `MeshTag`. `MeshTag` is in Bevy 0.18.
  - Town meshes that share a texture then share one `FieldLitMaterial`, so
    Bevy batches them, with multi-draw indirect where the GPU supports it.
  - A light or field change writes one row and never touches the
    material.
- **Expected (estimate).**
  - Most of the 1.45 ms prepare cost, and part of Extract and queue.
  - About 1-2 ms of render thread in towns.
  - Draw calls fall from one per mesh to one per texture per pass.
- **Look.** Unchanged: the same numbers reach the same shader. A capture of
  Ashenstead and Holbrook before and after must match pixel for pixel, or
  the difference is shown.
- **Measure.** Run `town` and `walk`. On the trace build
  (`--features trace`, `tools/trace_rank.py`), rank
  `prepare_erased_assets` and `batch_and_prepare` before and after.

### 3. A ground-height tile cache for repeated callers

- **PBD (measured).**
  - `column::surface_m` costs about 1.25 µs a call and was 60-88 % of every
    town bake (`town-frame-time/design.md`).
  - Off the column tier, `vehicles::ground_under`
    (`PBD crates/pbd-app/src/vehicles.rs:367`) asks the generator for every
    contact point of every substep, and resting craft stepped 2.9 ms a tick
    before decision 6 put them to sleep.
  - Decision 6 removed the resting craft's cost, not the cost of a craft in
    motion.
- **CB (measured).** `terrain-height-cache` keeps the lattice heights in
  16 x 16 tiles, filled the first time anything reads them
  (`CB openspec/changes/terrain-height-cache/proposal.md`, `Heightfield` in
  `cb-core`):
  - The mech step fell from 6-7 ms to 0.2 ms.
  - A shell's step fell from 0.43 ms to 0.01 ms.
  - Tree-streaming stalls of up to 380 ms went with them
    (`CB docs/benchmarks/2026-10-26-free-roam/report.md`, findings 2-3).
- **Change.**
  - A `pbd-core` tile cache over the generator's ground, keyed by cell and
    generator version. Derived state, never saved.
  - Used by vehicles, walking, the spawn search and the town bakes.
- **Invariant.** It is bit-identical to the uncached call, and that is a
  test. It is keyed by generator version, so an old save's world reads its
  own ground (CLAUDE.md "Saved games survive every change").
- **Expected (estimate).** It depends on how many craft move off the tier:
  0.5-1.5 ms a tick with a sailing fleet in view, about 0 when walking.
- **Measure.**
  - A new `harbour` scenario: sail out of a harbour with boats under way.
  - Also `town`. Report `VEHICLE_STEP` from the game's log.

### 4. Cloud-puff sites: rebuild on movement, not every frame

- **PBD (measured).** About 4,600 sites are rebuilt, culled and sorted
  every frame. That takes 0.35-0.4 ms of main thread
  (`lighting-perf/design.md` rank 4, `planet_cloud_puffs.rs:75-150`).
- **CB (read).** The billboard clouds are gathered and sorted again only
  when the eye has moved `resort_m`, because turning does not change the
  order. The wind moves the whole mesh by its transform
  (`CB clouds_render.rs:7-10`, `:429-448`).
- **Change.** This is `lighting-perf` task 2.4. Crab Breeze is
  corroborating evidence that rebuilding on movement works, not a separate
  task.

### 5. Spread or skip per-frame captures and casters

- **CB (read).**
  - The cockpit's environment cube draws one face a frame
    (`CB screens_cube.rs:132-156`).
  - The sky's environment map is filtered for `FILTERED_FRAMES` (3) frames,
    then the generator is taken off and the result kept
    (`CB sky_light.rs:33`, `:116-117`). Bevy re-filters every frame it is
    present.
  - Small things cast shadows only near:
    - grass from its first level only (`CB data/world/grass.ron:31`);
    - cars and people within 20 m (`traffic.ron:32`, `people.ron:63`);
    - window glass not at all.
  - Measured: the capital's GPU fell from 110-190 ms to 8-24 ms with
    dressing nearer (`CB docs/benchmarks/2026-10-28-city/report.md`,
    finding 3). Shadows cost about 1.5 ms of CPU at any hour
    (`CB night-frame/design.md` section 5).
- **PBD (read).**
  - The reflection capture runs every frame that reflections are on
    (`planet_ssr.rs:152-224`). `lighting-perf` rank 3 skips it when nothing
    reflects.
  - The shadow pass draws every standing town's whole triangle list with no
    per-cascade cull (`town-frame-time/design.md` suspect 7, and
    `planet_shadow.rs:433-447`, `:622-668`), with `cull_mode: None`
    (`planet_shadow.rs:276-282`). The far cascades are already redrawn on a
    stagger (about 2.75 of 4 a frame).
- **Change.**
  - Cull town casters per cascade by their bounds.
  - Give small town props (lamps, signs, clutter) a cast distance in data,
    as Crab Breeze does.
  - Redraw the outer cascade every second frame when the sun has moved less
    than its texel.
- **Expected (estimate).** 0.3-1 ms GPU in towns. Shadows are not priced
  in a town on this machine yet: `--no-shadows` on `town` is the first
  task.
- **Look risk.** Cascade edges, and a prop's shadow fading in at its
  distance. Needs shots at Ashenstead at 10:00 and 17:00.

### 6. Heavy rebuilds on a worker, keeping the old result until the new one lands

- **PBD (measured).** Every world open rebuilds the planet on the main
  thread in an exclusive system: `towns.rs` `settle` calls
  `planet::rebuild_planet`. That is a 6.7-7.5 s frame on a fresh world and
  5.1 s before a reopened world's first frame
  (`lighting-perf/design.md`, "the planet is built twice";
  `town-frame-time` task 3.6).
- **CB (read).**
  - The storm shade, a 93-115 ms main-thread rebuild every 16 s, moved to
    the async compute pool. The old shade stays bound until the new one is
    ready (`CB clouds_render.rs:566-604`;
    `CB docs/benchmarks/2026-10-28-city/report.md` finding 6).
  - Building dressing went the same way (`dress-off-thread`): no frame
    spends over 4 ms on it now, against 12-45 ms before
    (`CB docs/benchmarks/2026-10-25-desktop/report.md` finding 5).
- **Change.** This belongs to `town-frame-time` 3.6, and the pattern to use
  is the one above:
  - build the towns' ground in the first planet build, behind the loading
    screen, rather than rebuilding afterwards;
  - otherwise rebuild on the async pool and swap.
- **Expected (measured cost).** The 5.3 s frame goes. A reopened world
  reaches its first frame about 5 s sooner. p50 does not move.

### 7. Price 4x MSAA (a look decision)

- **PBD (read).**
  - `Msaa::Sample4` on an HDR camera (`desktop.rs:1650-1656`).
  - Every pass that reads depth reads a multisampled texture at sample 0
    (`water-composite/design.md:21-23`, `ssr_capture.wgsl:15`,
    `water.wgsl:127`).
  - The main pass writes four samples of `Rgba16Float` and resolves them.
- **CB (read).** `Msaa::Off` and FXAA on both cameras
  (`CB pilot.rs:149-150`, `:193-194`). It was never measured there.
- **Expected (estimate).** 0.5-1.5 ms of GPU at 1440 x 900, from resolve
  bandwidth and four-sample depth and colour. It also simplifies every pass
  that reads depth.
- **Look.**
  - This changes the picture. Pale Blue Dot's textures are nearest-sampled
    pixel art (CLAUDE.md), and FXAA softens texel edges as well as triangle
    edges. MSAA does not, so FXAA may be the wrong trade here.
  - The candidates are 4x, 2x, and off with FXAA. SMAA is a Bevy option on
    both versions.
- **Measure, then ask.**
  - Instrument: `PBD_MSAA=1|2|4` and `PBD_FXAA=1`, no default change.
  - A/B on `walk`, `rain-stroll` and `far-side`.
  - Shots of a shore, a town roofline and a tree line in each mode go into
    the owner survey (`owner-survey` skill) beside the numbers. This one is
    asked, because it comes with screenshots.

### 8. Late pipelines counted, and the loading screen gated on steady frames

This is for the in-flight loading-screen work.

- **CB (read).**
  - `perf_gate.rs:107` walks the render world's `PipelineCache` every
    frame. Each pipeline that finishes after `GAME_READY` is logged as
    `pipeline late:` with its label and shader defs, and counted on the
    exit line (`CB perf_gate.rs:1-9`, `:107-140`).
  - The loading screen lifts only after `STEADY_S` (1.0 s) of frames under
    `HITCH_S` (50 ms) once everything is in (`CB loading.rs:13-16`,
    `:113-126`).
  - `shader_warmup.rs` puts a few-centimetre copy of every (vertex layout,
    `StandardMaterialKey`, alpha mode) in front of the eye while loading
    (`CB shader_warmup.rs:1-13`, `:107-170`).
- **CB (measured).**
  - The counter found 107 late pipelines in a capital run, and 83 in the
    owner's night run.
  - The warm-up, measured old against new, showed **no effect on the
    median**. The late counts were 27 and 39 with it, against 52 and 23
    without: "the driver's own shader cache makes the count vary"
    (`CB city-frame/design.md:78-81`).
  - On Windows and Linux, Bevy compiles pipelines on worker threads
    (`CB streaming-hitches/proposal.md`, "Not changed"). The stall is the
    first frame that waits on one.
- **PBD (read).**
  - Pale Blue Dot's own passes (bloom, SSR capture, water composite,
    planet, shadow) queue their pipelines and skip the pass while
    `get_render_pipeline` returns `None` (`bloom.rs:258-266`).
  - So a late pipeline there shows as a missing pass (a frame without
    water or bloom), not a stall. Bevy's own material pipelines
    (`FieldLitMaterial`, `SkyMaterial`, `StandardMaterial` in vehicles and
    fish) can stall.
- **Change.**
  - The loading screen waits until the `PipelineCache` has no pipeline in
    `Queued` or `Creating` state, and then for 1 s of frames under 50 ms.
  - A late-pipeline counter and `pipeline late:` log lines go in the frame
    log and the suite's report, as a column.
  - Warm the town, vehicle and fish material keys by putting each first
    look in view behind the screen. Claim nothing for it until the counter
    says so.
- **Expected.**
  - p50 does not move.
  - p99.9 and max should improve, and so should the first seconds after a
    world opens. The suite's max column is dominated by the planet rebuild
    today (rank 6).

### 9. `codegen-units = 1` in the release profile

- **CB (read).** `[profile.release] lto = "thin"`, `codegen-units = 1`
  (`CB Cargo.toml:38-40`). The comment: "The game is CPU-bound ...
  whole-program optimisation for the build that is played and measured".
- **PBD (read).** `lto = "thin"`, `strip = true`, and the default 16
  codegen units (`Cargo.toml:19-21`).
- **Expected (estimate).** 0-5 % of CPU time. It mostly helps the
  main-thread-bound town walk. The release build gets slower, but the
  `fast` profile, which is what `run.bat` uses, is unaffected.
- **Measure.** Run `town`, `walk` and `storm`, building the same commit
  twice and interleaving the two builds. Keep it only if the saving
  exceeds the spread.

### 10. A frame budget for every frame, listing the offenders

- **CB (read).** Every frame after `GAME_READY` is judged against 16.7 ms.
  The exit line lists each frame over it, and `--perf-gate` makes the run
  fail (`CB perf_gate.rs:56-99`, `cb-core` `perf::FrameBudget`). The
  report lists every such frame with its cause (CB CLAUDE.md section 7).
- **PBD (read).** `perf_suite.py` reports percentiles. The planet-rebuild
  frame hid in `max` for weeks, until `lighting-perf` named it.
- **Change.** The suite adds an "over 16.7 ms" count and lists the 10
  worst frames per run, with their timestamps and the `SPENT` lines nearest
  them. There is no game-side gate. This is reporting only.
- **Expected.** It saves no time by itself, but it finds causes sooner.

## 4. Looked at and not proposed

- **Bevy's GPU occlusion culling** (`CB pilot.rs:204-205`). It culls Bevy
  meshes against last frame's depth pyramid. Pale Blue Dot's terrain, the
  main occluder, is drawn by its own pipeline and does not write Bevy's
  prepass depth, so towns behind hills would not be culled. Pale Blue Dot's
  own horizon cull already covers terrain. Revisit only if towns grow to
  thousands of meshes.
- **`VisibilityRange` dither LOD.** Crab Breeze took it from Pale Blue Dot
  (`CB lod_fade.rs:1-3`, `lod-crossfade/proposal.md`). There is nothing to
  bring back.
- **The 120 Hz frame cap** (`CB sim.rs:40-74`, `data/sim.ron:5`). It is a
  sleep-and-spin cap for smooth pacing, not for speed. Pale Blue Dot uses
  vsync. Reconsider only if uncapped play on a high-refresh display needs
  one.
- **Bevy mesh slabs.** Crab Breeze tried them and undid them: separate
  buffers stopped batching and cost every frame
  (`CB city-frame/design.md:86-89`).
- **Billboard clouds, Bevy's atmosphere, analytic fog.** Cheaper, but a
  different picture (section 1).
- **One-sided imports** (`CB one_sided.rs`): 1,139 of 1,723 glTF materials
  were double-sided there. Pale Blue Dot's town and fish materials set
  `cull_mode: None` deliberately: flames and cards (`towns.rs:1063`,
  `:1379`, `fish.rs:787`). Its shadow casters do the same for blades and
  walls (`planet_shadow.rs:278-282`). A census of closed solids among them
  is cheap. It is task 5.3, not a ranked item.

## 5. Order

Rank 1 first, because it is a flag and an afternoon's sitting, and it
changes the baseline every later A/B is read against. Then:

- ranks 9 and 10 (cheap, no picture change);
- rank 2;
- ranks 3, 5 and 6;
- rank 8, with the loading screen;
- rank 7 last, because it is a look question for the owner.

Rank 4 and the L8 half-resolution pass (section 2) ride on `lighting-perf`.
The Bevy 0.20 upgrade (`bevy-0-20`) comes after `lighting-perf` and the
loading screen land, so that its A/B is not confounded with theirs. The
latency A/B must be re-run on 0.20, because the swapchain path changes with
wgpu.
