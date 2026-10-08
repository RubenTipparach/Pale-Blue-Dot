# Design: town-frame-time

## What has landed since the last measured suite (2026-09-26)

| Change | Date | What it adds to a frame |
| --- | --- | --- |
| `sun-shadows` | 09-30 | Four 2048² cascades, about 2.75 redrawn a frame; each a GPU cull, 3 indirect planet draws, and every standing town's whole triangle list with no per-cascade cull (`planet_shadow.rs:433-447`, `:622-668`) |
| `cities-in-the-world`, `tenebris-towns` | 09-29 to 10-07 | Field-lit town meshes, one material each; the vertex bake; room lights |
| `smooth-weather` | 10-08 | The shown pair: two cube maps uploaded per pair move, a blend compute pass, rain map sampled from both states |
| `compass-bar` | 10-08 | About 40 UI nodes written every frame |

`town-flyover`'s design already records 8.1 ms GPU over open sea against
1.3-3 ms in the 09-25 flights (`town-flyover/design.md`, "Frame time").

## Suspects, read from the code (not measured)

### Main-thread work that grows with towns: the likely city drop

1. **The town light bake, `field_light.rs:376-439`.** The bake key is the fine
   set's `Arc` pointer and its light version (`:388-391`). A new fine set
   lands every few seconds while walking, and dusk and dawn relight the whole
   tier (`planet_lod.rs:1061-1093`). Every landing near a town therefore
   re-bakes every standing town: up to `BAKE_BUDGET` = 200,000 vertices in one
   frame. Each vertex calls `column::surface_m` (`:397`), which runs the full
   terrain generator and the settlement ground, then a field lookup. Then the
   whole mesh is re-uploaded. The design priced this as "a slot lookup and an
   interpolation" (`cities-in-the-world/design.md`). The generator call was
   never timed.
2. **`light_from_the_field`, `field_light.rs:235-318`, every frame, every
   field-lit mesh.** Eight corners, each a `surface_m` and a field lookup. The
   material is rewritten when the sun has moved by 1e-4 (`:308`). With
   `DAY_S` 2880 that happens every few frames, so every town material is
   prepared again (new uniform and bind group) many times a second. One
   material per mesh also means nothing batches.

### Weather: the likely stutter and the abrupt cloud changes

3. **Pair moves, about once a second.** `planet_weather.rs:315-320` uploads
   six cube textures (from, to, and to again straight into the cubes): about
   1.2 MB, 590,000 `half_bits` conversions and six allocations. Only `to` is
   new. `fill_rain_map` (`weather.rs:286-287`) runs `precipitation_map` on
   both states, 8,192 `Atmosphere::sample` calls on the main thread, and
   allocates each frame (`:291-296`). This fits a 1 Hz blip.
4. **Held steps, then a burst.** Stepping stops while a fine set is rebuilt
   and catches up by up to 30 steps in one task (`atmosphere.rs:487-535`,
   `MAX_STEPS`). That is about 5 ms a step on the async pool, competing with
   LOD and render for cores. On screen, the weather freezes and then plays up
   to 7.5 times fast for the 4 s blend cap.
5. **The flow-map drift reads the wind.** The noise offset is wind × phase ×
   120 s (`clouds.wgsl:228-241`), so a change in wind slides the detail by
   Δwind × up to 120 s. After a 30-step burst the clouds visibly slide.
6. **Snaps.** The rain/snow kind flips at t = 0.5 (`weather.rs:196-203`). The
   clock-jump snap over 60 s (`atmosphere.rs:136`) is intended. The cloud-top
   probe clip (`clouds.wgsl:489-503`) moves the march's far end when the
   cloud top moves.

### Broad GPU cost: where "the hundreds" went

7. **Shadows** (item 1 of the table): priced by `--no-shadows` as a variant.
8. **Clouds**: the 09-25/26 cloud work already took the clouds scenario from
   183 to about 145 fps (`docs/benchmarks/2026-09-26-*`). The march's cost
   grows with cover, so frames fall as weather closes in. Priced by
   `PBD_NO_CLOUDS=1`.

## The instruments (measurement only, no behaviour change)

Each one only logs. None changes what is drawn or simulated.

- `planet_lod::spent()` (logs `SPENT x ms <what>` over 2 ms) wrapped around:
  `bake_town_outsides` (with vertices baked), `light_from_the_field` (with
  materials rewritten), `planet_weather` upload and blend (with bytes), and
  the rain-map refill.
- One log line per weather publish: steps taken, seconds held by a LOD
  rebuild, and the step and map split of `step_copy`.
- Frame-log columns: `baked_verts`, `mats_touched`, `air_gen`, `blend_t`,
  `cover`, `rain`. `tools/frame_graph.py` marks pair moves and stops leaving
  "Weather here" out of spike attribution.
- `tools/perf_suite.py` scenario `town`: `--walk --at 19.644 59.178
  --walk-distance 400 --rain 1`, which walks through Ashenstead (walled, the
  shipped seed's tour town 4) under forced rain at 10:00. It is checked like
  the others: a run that never came within the town's patch is invalid.

## The sitting

On release builds with nothing else running, interleaved. The 2026-09-25
baseline exe is run with its own assets.

| Variant | What it prices |
| --- | --- |
| `before` (09-25 baseline exe) | where "the hundreds" were |
| `after` (main) | now |
| `after, --no-shadows` | sun shadows |
| `after, PBD_NO_CLOUDS=1` | the cloud passes |

Scenarios: `town`, `clouds`, `storm`, `walk`, two repeats each. Then
`frame_graph.py` on the `town` and `storm` runs lines each frame over 16.7 ms
up with the `SPENT` and publish lines.

## Measured: the town walk (2026-10-08)

The owner's machine, release build, windowed, `--no-vsync`. The walk starts
at Ashenstead with `--walk --at 19.644 59.178 --walk-distance 400 --rain 1
--time 10`, and runs for 101 s.

| | Measured |
| --- | ---: |
| Frame, p50 / p95 / p99, after 30 s | 15.7 / 19.4 / 61.4 ms (64 fps at p50) |
| GPU total, p50 / p95 | 8.1 / 9.8 ms: the frame is CPU-bound |
| Frames over 150 ms | 34. Two follow each fine-set landing (every ~3 s) in the town, then one each once the walk has left it |
| Share of the run spent in those frames | 11% |

The instrument (`SPENT` around the bake and the corners, split by step):

| Step | Cost | When |
| --- | ---: | --- |
| Town bake, 37 meshes / 178,341 vertices | 359-366 ms: ground 222-226, locate 33, sample 80-82 | every landing |
| Town bake, 20 meshes / 163,854 vertices | 239-447 ms: ground 203-393, locate 7-10, sample 8-12 | the next frame |
| `light_from_the_field`, 577 meshes | 6.4-10.1 ms, of which 0-127 materials rewritten | **every frame** |

`column::surface_m`, the terrain generator's ground, costs about 1.25 µs a
call. It is 60-88% of every bake. In the corners it is 4,616 calls a frame
(577 meshes × 8), about 5.8 ms, for an answer that never changes while a
mesh stands still. Suspects 1 and 2 are the town drop. Suspects 3 to 8 are
not yet measured, and are taken up after these.

## Decision: suspects 1 and 2 (no change to the picture)

1. **Corners.** Each field-lit mesh keeps the eight points its corners last
   read, ground-clamped, with the key of the field they were read from: the
   fine set, its light version, and whether contact serves it. The ground is
   computed again only when the mesh's bounds move by more than 1 mm, as for
   a ship. The field is read again only when the key or a point changes. The
   numbers handed to the material are the ones it gets today.
2. **Bake points.** A town's outside keeps the ground-clamped point each
   vertex reads, computed once at its first bake. Its positions are in the
   planet's frame and the town does not move. A re-bake is then locate and
   sample only.
3. **Bake only what the field can reach.** A `ColumnTier` records its anchor
   and reach. Off the tier, `sample` answers the open sky and no lamp
   (`planet_column.rs`). So a town whose bounding cap lies wholly outside the
   tier's reach, plus a 10 m margin, bakes to exactly `(1, 0)`. If it was
   already baked that way, it is not baked again.
4. **Spread a bake.** The bake budget is time, not vertices: meshes are baked
   in turn until 4 ms has been spent in the frame. A town near the player then
   re-bakes over a few frames, not in one. Each mesh is still swapped whole,
   so a mesh never mixes two bakes. The only visible difference is that a
   dusk relight reaches every mesh of a town over a fraction of a second, not
   in one frame.

## Next: what the remaining 10 ms is (2026-10-08)

After decisions 1-4 the town walk runs about 10.4 ms a frame against 6.3 ms of
GPU time. The frame is still CPU-bound, and none of the `SPENT` lines says
where. The instrument is a build feature, `trace`, which turns on Bevy's
`trace_chrome`. Every system and render stage then writes a span to a Chrome
trace file, and a script adds up each span's time per frame over a
town walk. It changes no behaviour. The spans cost time themselves, so it
ranks costs and does not measure them; a fix is measured on the ordinary
release build, as above.

### Measured: the trace of a 100 m town walk

From the `trace` build (`fast` profile), the 1,186 frames between 25 s and
36.6 s, median frame 10.8 ms. The spans cost time themselves, so this ranks
costs; it does not measure them.

| Span | ms a frame | Where |
| --- | ---: | --- |
| Main schedule, all of it | 8.1 | main thread |
| `vehicles::step_vehicles` (in `FixedUpdate`) | 1.8 | holds the main schedule |
| `towns::stand_in_range` | 1.7 | main thread |
| Physics (`PhysicsSchedule`) | 0.7 | main thread |
| `field_light::light_from_the_field` | 0.34 | main thread |
| `weather_maps::blend` + `weather::rebuild_shower` | 0.69 | main thread |
| Render schedule, all of it | 8.1 | render thread, beside the next main |
| `prepare_windows` (waits on the swapchain) | 2.4 | render thread |
| `prepare_erased_assets<FieldLit>` + `prepare_material_bind_groups` | 1.45 | render thread |
| Extract | 1.45 | between the two |

The main and render threads each take about 8 ms, so both must come down.

## Decision: the next three (2026-10-08)

5. **`stand_in_range` copies six town templates every frame.** A `Template`
   is a plain struct: its buildings, its wall cells and its ground grid.
   The system clones all six, with the kits, before asking whether any town
   needs cutting. It clones them only when one does. No change to behaviour.
6. **`step_vehicles`: about 2.9 ms a tick, on craft sitting still.**
   - Harbour boats past 1,500 m are not the cost: they are already stowed as
     records and not stepped (`harbour::stow_and_wake`).
   - The instrument (`VEHICLE_STEP`, each kind's mean step every 5 s) found
     about 20 craft in range of Ashenstead, each stepped every tick with 4
     substeps:

     | Craft | ms a step | Stepped a tick | ms a tick |
     | --- | ---: | ---: | ---: |
     | Rowboat | 0.12 | 11 | 1.3 |
     | Tern | 0.19 | 6 | 1.1 |
     | Loon | 0.23 | 2 | 0.45 |
     | Kestrel | 0.04 | 1 | 0.04 |

   - The Kestrel, 16 m away on the column tier, steps 3-5 times faster than
     the far craft. Off the tier, every contact point of every substep asks
     the terrain generator for the ground (`vehicles::ground_under`).
   - Nothing about a craft at rest changes from one tick to the next. So a
     craft that nobody is aboard, that has been at rest for `REST_S` (the
     `rested` the save already keeps), and that is more than 30 m from the
     camera **sleeps**. It is stepped once every 30 ticks (0.5 s) to see whether
     anything has moved it: wind, a storm's sea, a dig under it. A step that
     finds it moving wakes it, and it is stepped every tick again. Boarding it
     or coming within 30 m does the same.
   - What changes: a resting craft answers a push up to half a second late,
     and only when it is more than 30 m away. Recommendation taken (ask only
     with screenshots).
7. **Town materials are prepared again every few frames.**
   `light_from_the_field` rewrites a material when the sun has moved by 1e-4
   (0.006°). With `DAY_S` 2880 the sun crosses that every 45 ms, so 50-125 of
   577 materials are rewritten a frame. Each rewrite makes Bevy prepare the
   material again (a new uniform buffer and bind group, `prepare_erased_assets`).
   The threshold becomes 2e-3 (0.11°): each material's sun is at most 0.11°
   behind the sky's, and each is rewritten about once a second. The meshes
   still cross it at different moments, so the rewrites stay spread across
   frames. Nothing on screen changes by a visible amount; recommendation taken
   (ask only with screenshots).

### Measured: decisions 5-7 (2026-10-08)

The town walk, `before` (decisions 1-4) against `after` (1-7), interleaved,
two runs each, in one sitting (`tools/perf_suite.py --scenarios town`). The
owner was using the machine during it, so the spread is wider than in the
earlier sitting.

| | before | after |
| --- | ---: | ---: |
| Frame p50 | 10.43-10.52 ms | **9.08-9.18 ms** |
| Mean fps | 82 | **98** |
| p95 | 16.2-16.4 ms | 15.0-15.2 ms |
| p99.9 | 131 ms | 45 ms |

Craft stepped a tick fell from about 20 to under 2. Only the Kestrel, 16 m
from the walk's start, steps every tick. Their cost fell from about 2.9 ms a
tick to about 0.15 ms. The frames slower than 14 ms (7%) are not periodic:
they are the spread of an uncapped frame (2-20 ms), not hitches with a cause.

## Candidate fixes (each waits for its number)

| Suspect | Fix | Changes the picture? |
| --- | --- | --- |
| 1 | Key the bake on what the town's cells read: the light version, plus a hash of the fine slots the town's patch covers, not the set's pointer. Take the ground height from the town's own stored ground (already laid out with it), not the generator. Bake off the main thread and swap the attribute in whole. | No: same values, computed less often |
| 2 | Read the corners' ground from the town's record. Rewrite a material only when its field moved past 1e-3 or the sun moved past about 0.1°. Share one material per town where the room lights allow. | No |
| 3 | Upload only `to` on a pair move (the previous `to` is already on the GPU as `from`). Drop the direct cube write once the blend pipeline is ready. Keep the rain map's buffer. | No |
| 4 | Let stepping continue during a LOD rebuild at a low thread priority, or cap a catch-up at a few steps a publish. | Burst playback slows: shown with before-and-after shots |
| 5 | Drift the flow-map phase by the integral of the wind, not wind × phase. | Yes: before-and-after shots |
| 7 | Cull town casters per cascade, and redraw the far cascades less often. | Possibly, at cascade edges: shots |

A fix that changes a picture goes to the owner with screenshots before it
lands. The rest land with the sitting that proves them.
