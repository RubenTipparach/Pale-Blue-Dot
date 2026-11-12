# Design: lighting-perf

## Context

`lighting-model` (PR #27, merged as `f15a647` on 2026-11-11) added eight
parts to the frame: L1 sun colour from the air, L2 sky probe (SH9), L3
contact-hardening shadows, L4 sun bounce in the voxel field, L6 tone, grade
and bloom, L7 a lamp bake, L8 reflections (a voxel walk and a screen trace
on wet ground, a screen trace on the sea, and a half-resolution capture of
last frame that both traces read), and the lit rain fog.

All the numbers below come from three sittings on the owner's desktop on
2026-11-12, on a quiet machine (`docs/benchmarks/2026-11-12-lighting-model-clean/`):
RTX 3070, Vulkan, i7-9700F, 1440 x 900, `--no-vsync`, release builds, 2
repeats interleaved. They replace the sitting of 2026-11-11, which ran beside
mockup agents rendering on SwiftShader. Its model costs agree with these
within 0.3 ms at p50, but its tails and cold starts do not.

- **A/B**: `before` = `db25fe2` (main before the lighting merge, with its
  own assets), `lighting` = `f15a647`, `today` = `f15a647` with
  `PBD_LIGHTING=today` (the old model in the same exe). Clock at 10:00.
  Plus the `town` walk.
- **Pricing**: `f15a647` with one part off at a time, on the two scenarios
  where the model costs the most (`rain-stroll`, `storm`). Clock at 10:00.
- **Night**: `rain-stroll` at 22:00, the case where the lamps light the lane.

**Measured** means read off those runs. **Read** means read from the code
at `f15a647`. **Estimated** means neither, and is said so each time.

## What the model costs (measured)

p50 / p95 / p99 in ms, median of 2 runs; `today` is the same exe with the
old model.

| scenario | today | lighting | delta p50 | delta p95 | spread p50 (max of variants) |
| --- | ---: | ---: | ---: | ---: | ---: |
| rain-stroll (A/B) | 11.54 / 14.30 / 16.21 | 14.27 / 18.48 / 28.47 | **+2.73** | **+4.18** | 0.12 |
| rain-stroll (pricing) | 12.39 / 15.55 / 18.36 | 14.33 / 18.69 / 26.86 | **+1.94** | **+3.14** | 0.82 (`today`) |
| storm (A/B) | 9.01 / 13.05 / 18.18 | 10.06 / 16.50 / 25.05 | +1.05 | +3.45 | 0.17 |
| storm (pricing) | 9.22 / 13.63 / 19.75 | 10.05 / 16.60 / 25.74 | +0.83 | +2.97 | 0.11 |
| clouds (A/B) | 8.96 / 13.06 / 18.78 | 9.89 / 16.33 / 25.65 | +0.93 | +3.27 | 0.04 |
| far-side (A/B) | 5.65 / 9.78 / 12.55 | 6.67 / 12.00 / 16.75 | +1.02 | +2.22 | 0.03 |
| walk (A/B) | 9.09 / 10.71 / 12.10 | 9.83 / 12.53 / 15.33 | +0.74 | +1.82 | 0.17 |
| town (A/B) | 17.69 / 24.45 / 28.02 | 18.11 / 26.09 / 32.61 | +0.42 | +1.64 | 1.29 (`today`) |

The town walk is CPU-bound (17-18 ms wall against 6-12 ms of GPU) and noisy;
its delta is inside its spread.

## Where it goes (measured, pricing sitting)

Cost of a part = the whole model minus the model with that part off.

| part off | rain-stroll p50 / p95 / p99 | storm p50 / p95 / p99 | GPU total p50 (stroll / storm) |
| --- | ---: | ---: | ---: |
| L8 reflections (`PBD_REFLECT=0`) | **2.18 / 3.68 / 8.24** | **0.64 / 2.49 / 4.30** | **-2.47 / -1.04** |
| tone + grade + bloom (`PBD_NO_TONE=1`) | 0.43 / 0.47 / 1.18 | 0.40 / 0.36 / 1.83 | -0.25 / +0.10 |
| bloom alone (`PBD_NO_BLOOM=1`) | 0.36 / 0.32 / 0.25 | 0.33 / 0.21 / 1.26 | -0.02 / +0.02 |
| L3 soft shadows (`PBD_SOFT_SHADOWS=0`) | -0.13 / -0.14 / -0.61 | 0.00 / -1.19 / 0.56 | (none) |
| L4 + L7 (`PBD_BOUNCE=0;PBD_LAMP_BAKE=0`) | -0.16 / 0.23 / -0.93 | -0.02 / -0.02 / 1.21 | (none) |
| clouds (`PBD_NO_CLOUDS=1`, for scale) | 0.15 / 0.52 / 2.00 | 0.71 / 1.13 / 1.75 | (0.40 clouds) |

- **L8 is the model's cost on foot in the rain: all of it.** With it off
  the stroll reads 12.15 ms, the old model 12.39 ms (whose two runs spread
  0.8 ms; the A/B's 2.7 ms is the cleaner measure of the whole).
- **L8 is most of the storm's tail**: p95 +2.5 of +3.0 ms, p99 +4.3 of
  +6.0 ms.
- **On the stroll the frame is GPU-bound in L8.** GPU total (the sum of
  Bevy's top-level render spans) falls 2.5 ms with L8 off; wall time falls
  2.2 ms. Whether the terrain's main pass is one of those spans is not
  verified (the code reader thought it is not; a 2.5 ms fall in GPU total
  says some L8 work is spanned). The split is task 1.1.
- **The cost depends on where the eye looks** (the stroll's frame graph,
  `rain-stroll.png`): the lane is walked in 7 s legs, one way at 15-18 ms,
  back at 11-13 ms; the old model holds 11-12 ms both ways.
- **L3, L4 and L7 cost nothing measurable by day** on this machine, as on
  the Mac. Their p99 differences are of either sign and inside the p99
  spread between repeats.
- **Bloom + tone cost 0.4 ms p50**, of which bloom is 0.33-0.36 ms.
- **The CPU half of the frame did not move**: wall p50 sits 4-5 ms above
  GPU total in every variant of storm and walk, and is the same with and
  without the model. The new model adds GPU work, not main-thread work,
  except the small items in rank 4-6.

## Night (measured)

`rain-stroll` at 22:00, the lane lit by Holbrook's lamps, two repeats each:

| variant | p50 | p95 | p99 | GPU total p50 |
| --- | ---: | ---: | ---: | ---: |
| new | 14.03 | 19.69 | 29.84 | 9.46 |
| today (old model) | 11.64 | 14.68 | 16.32 | 6.57 |
| L4 + L7 off | 13.78 | 19.04 | 28.79 | 9.31 |
| L8 off | 11.65 | 15.30 | 20.06 | 7.03 |

- **The night is the day's picture**: the model costs 2.39 ms p50, and L8
  is 2.38 ms of it (p95 4.4, p99 9.8 ms).
- **The lamp bake and sun bounce cost 0.25 ms at night** (L4 + L7 off),
  about twice the 0.13 ms spread: small, possibly real where lamps are lit.
  The lit rain fog, which loops the lamps per shell, has no toggle of its
  own; it is inside what is left once L8 and L4 + L7 are off.

## Not the lighting model: the planet is built twice on every open (measured)

On a quiet machine every run still has one frame of 6.7-7.5 s. The game's
log and `tools/frame_graph.py` put it on `towns.rs` `settle`, an exclusive
system that calls `planet::rebuild_planet` (5.3 s) on the main thread to
install the towns' ground. On a fresh world that is a frozen frame about 9
s after the window opens, beside a 2.6-3.0 s frame where the towns are laid
out. A reopened world pays the same rebuild before its first frame (13.0 s
to first frame against 9.3 s). So every open generates a planet (5.1 s) and
throws it away. It is in `db25fe2` too. It belongs to `town-frame-time` 3.6
("a fresh world freezes for about 18 s"), not here; it is recorded so the
next reader does not take the `max` column for the lighting model.

## Ranked optimizations

Rank = measured cost of the part x the share of it the fix is expected to
remove. The expected share is an **estimate** from the code read; the cost
is measured. Each is checked for the look with `--capture` shots, same
place, same clock, old and new side by side, and measured after with the
suite: `rain-stroll`, `storm`, `walk`, `far-side`, interleaved against
`f15a647`'s exe.

### 1. L8 terrain trace: end the walk when nothing can be hit

- **Measured**: L8 costs 2.2 ms p50 / 8.2 ms p99 on the stroll (the whole
  model 2.7 ms in the A/B), 0.64 / 4.3 ms in the storm. Its terrain, sea and capture shares are not split
  yet (task 1.1); the stroll is in a town lane where the wet ground fills
  the lower half of the screen, so the terrain trace is the likely bulk
  (**estimate**).
- **Read** (`planet_surface.wgsl` `reflect_trace` ~1961-2057, gate at
  ~2541): every wet fragment with `fr > 0.03` walks up to 72 voxel steps;
  each step reads `columns[slot]` twice (once directly, once in `solid_at`),
  up to 6 `centres`, two sphere tests and a `COLUMN_RUNS` loop. A ray that
  finds nothing then runs the 32-step screen trace too. A puddle seen from
  eye height reflects up into open air, and a rising ray only stops when it
  leaves the column tier's layers, one layer per metre, up to 48 m away. So
  the common case, the miss, is the most expensive one. That matches the
  direction-dependence: looking along the lane toward open sky reads
  slower.
- **Fix**: stop the walk once the ray is rising and above the highest solid
  layer of the column it is in and its neighbours (a per-column top the
  tier already has, or one more `u32` per record), and skip the screen
  fallback for a rising ray that has left the tier's top, where the screen
  can only show sky that `env` already is. Gate both traces by the
  `rain_near` distance that already fades the rings
  (`wet-near-detail`).
- **Expected**: most of the terrain trace's misses end within a few steps
  (**estimate**: a third to two thirds of the terrain share; 0.8-1.8 ms p50
  on the stroll).
- **Look risk**: low. A ray that stops early returns what it returns today
  on a miss (`hit.a = 0`, the SH sky). The one place it could differ is a
  ray that rises above a low roof and would have met a taller one further
  off; the top must be the max over the neighbourhood the walk can reach.
  Check: `--capture` the lane at Holbrook (`--at 29.69673 1.19287`, rain
  1, 10:00 and 22:00) looking both ways, and a puddle under a wall.

### 2. L8 sea trace: only where it can show something

- **Read** (`water.wgsl` `sea_ssr` ~577-605, gate ~542): every above-water
  sea pixel with `fresnel > 0.02` runs a 32-step march (a mat4 projection
  and a `textureLoad` per step). The step grows `t += 0.1 + 0.14 t`, so 32
  steps reach about 53 m and the 120 m `reach` is never hit; nearly every
  ray runs all 32 steps unless it leaves the screen. From any height, sea
  more than a few hundred metres off reflects a 53 m segment that covers a
  few pixels, most of which hit the "within 3 texels of home" `continue`.
- **Measured**: not split from L8 yet. The storm (a flight, over sea for
  part of the route) has L8 at 0.64 ms p50 / 2.5 ms p95, which is all sea
  and distant wet ground; that bounds this fix's saving in flight.
- **Fix**: skip the trace where the projected 53 m covers fewer than ~4
  pixels (one projection of the end point), and let the step count fall
  with distance. Optionally trace at half resolution.
- **Expected** (**estimate**): most of L8's flight cost, 0.3-0.6 ms p50 and
  1-2 ms p95 in storm.
- **Look risk**: low; what is dropped is sub-pixel reflection. Check: a
  `--capture` of a harbour from 30 m and from 300 m, before and after.

### 3. L8 capture: skip when nothing reflects

- **Read** (`planet_ssr.rs` ~161-224): a 960 x 600 Rgba16F capture (4.6 MB
  written) of last frame's colour and depth every frame reflections are on,
  wet ground or sea in view or not, and a buffer and bind group made every
  frame.
- **Fix**: skip the pass, and mark the history invalid (`ssr_eye.w = 0`),
  when the weather is dry and no sea is on screen; keep the buffer.
- **Expected** (**estimate**): 0.1-0.3 ms on dry walks and inland flights
  (the dry `walk` costs +0.93 ms p50 for the whole model).
- **Look risk**: none when nothing reads it. The first wet frame after a
  dry spell has no history: one frame of SH-sky reflection, which the
  trace's miss already shows.

### 4. Cloud-puff sites rebuilt every frame (CPU)

- **Measured**: the game's own log, `PUFFS built ... build X ms`, every 120
  frames: 0.39 ms mean, 0.77 ms max in the storm; 0.35 ms on far-side.
- **Read** (`planet_cloud_puffs.rs` ~75-150): about 4,600 sites with 8 map
  taps each, rebuilt, culled and sorted every frame.
- **Fix**: rebuild only when the weather maps, the cloud clock step or the
  eye's cell changes; cull and sort the kept list.
- **Expected** (**estimate**): most of 0.35-0.4 ms of main-thread time a
  frame. It is CPU time, which is the larger half of the frame in flight
  (wall 9.6 ms against GPU 5.4 ms in storm).
- **Look risk**: none if the rebuild keys are complete. Check: a cloud-hop
  capture sequence, before and after, frame for frame.

### 5. Small per-frame CPU costs

- **Read**: `near_lamps` (`planet.rs` ~693) allocates and sorts every tier
  lamp every frame; `reflections_on`, `new_model_on`, `shadow_spread`
  (`config.rs` ~1358-1392) read environment variables on every call,
  several times a frame. Cache both.
- **Expected** (**estimate**): tens of microseconds. Do it only beside
  another change in these files.

### 6. Lamp bake on every landing (not measured as a cost)

- **Measured**: L4 + L7 off saves nothing measurable by day (-0.16 / -0.02
  ms p50, inside the spread) and 0.25 ms at night in a lamp-lit lane (about
  twice the spread).
- **Read** (`planet_bounce.rs` ~84-159, `planet.rs` ~1001-1029): the lamp
  bake reruns on every `fine.version`, even with no lamps in reach, uploads
  up to ~4.3 MB, and restarts a 0.5 s cross-fade in which the shader reads
  both halves.
- **Fix**: skip the bake and the fade when no lamp is in the set's reach;
  a "no lamps" flag lets the shader skip `lamp_near`.
- **Ranked low**: no measured cost. It may matter for landings (pop-in,
  priority 1) because the bakes share the async pool with terrain
  generation; that is for `detail-fade`, not here.

### 7. Bloom: one fewer full-screen pass

- **Measured**: bloom 0.33-0.36 ms p50, 0.2-0.3 ms p95.
- **Read** (`bloom.rs` ~276-361): ten passes; the first downsample reads the
  full-resolution HDR target and a separate composite does a full-
  resolution read-modify-write; 10 uniform buffers and bind groups are made
  every frame.
- **Fix**: sample bloom's level 0 inside `tone_grade.wgsl` and drop the
  composite pass; make the uniform buffers once.
- **Expected** (**estimate**): 0.1-0.2 ms.
- **Look risk**: none; the same sum in another pass. Check: a capture of a
  lamp at night and the sun's disc, before and after, pixel-diffed.

### Not ranked

- **L3 soft shadows**: no measured cost on either machine. Leave it.
- **Lit rain fog** (`water.wgsl` ~1548-1663): per pixel, 2 kinds x 6 shells,
  each shell looping all 8 lamps with two `atan`, up to 96 lamp iterations.
  No toggle prices it alone; in the storm, L8 (0.64 ms) and tone + bloom
  (0.40 ms) already account for the whole model's 0.83 ms p50, so it is not
  a measurable cost by day; at night in the lamp-lit lane the model less
  L8 leaves about 0.0 ms for it and everything else.
- **The flight tails** (50-450 ms frames in fine-set landings, town bakes
  and the fresh world's 6.3 s planet rebuild after the warm-up): present in
  every variant and in `before`. Not the lighting model's.

## Decisions

1. **Instruments before fixes.** Task 1 lands first and the L8 split is
   measured; the ranking above is re-ordered on the split if it disagrees.
2. **One fix per measurement.** Each of ranks 1-4 is its own A/B against
   `f15a647`, so a saving is attributed.
3. **The look is the gate.** A fix that moves a pixel in the check shots
   is shown to the owner with the shots before it is kept.
4. **Recommendation taken (ask only with screenshots)**: the fixes are made
   in the order above without asking, because each one either does not
   change the picture or is checked by shots first.

## Risks

- The L8 split may show the sea or capture larger than read; the ranking
  then changes (decision 1).
- A per-column top must be kept up to date with edits (a dug hole, a placed
  block); it is derived from the same column records the walk reads, so it
  is rebuilt where they are.
