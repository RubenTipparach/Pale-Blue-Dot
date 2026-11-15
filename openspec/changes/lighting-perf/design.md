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

## The instruments' first sitting (measured, 2026-11-12)

`57d94b5` (tasks 1.1-1.3), same machine, 1440 x 900, uncapped, release,
clock at 10:00, interleaved. Two sittings run back to back
(`docs/benchmarks/2026-11-12-frame-latency/` and
`output/perf/2026-11-12-l8-split`):

### Frame latency (from `crab-breeze-lessons`)

The swapchain was asked for a queue of one frame. With two, every scenario
loses 3.2-4.3 ms at p50 and more at the tail (`rain-stroll` 14.18 to 10.49,
`storm` 10.17 to 6.67, `walk` 9.05 to 5.81, `town` 11.16 to 6.91 ms p50; 3
repeats, p50 spread 0.07-0.76 ms), with the same GPU time: with one frame
queued, the render world waited for the GPU's last frame before encoding the
next. Two is the default from `71c827a` (`crab-breeze-lessons` 1.3). **This
moves the baseline**: at latency 2 the stroll is GPU-bound (10.5 ms wall
against 9.9 ms of GPU), so a GPU saving now shows in wall time nearly one
for one. Every A/B below that names latency 2 is measured at it.

### L8 by part (task 1.1)

`PBD_REFLECT` with each part off, 2 repeats, latency 1. Cost = all on minus
that part off; wall p50 / p95 / p99, then the terrain pass's GPU p50
(`main_transparent_pass_3d`, where the terrain is drawn and both of wet
ground's traces run), then the compose pass's (`water_compose`, the sea).

| part | rain-stroll wall | stroll terrain GPU | storm wall | storm terrain GPU | storm sea GPU |
| --- | ---: | ---: | ---: | ---: | ---: |
| all of L8 (`0`) | **2.77 / 4.05 / 9.54** | **2.06** | **1.36 / 2.62 / 5.30** | **1.19** | 0.14 |
| the voxel walk (`walk`) | **2.10 / 3.07 / 7.54** | **1.50** | -0.03 / 0.53 / 1.87 | 0.05 | 0.00 |
| wet ground's screen trace (`screen`) | 0.46 / 1.68 / 2.54 | 0.21 | **1.10 / 2.07 / 4.59** | **1.09** | 0.00 |
| the sea's screen trace (`sea`) | 0.13 / 1.77 / 0.38 | -0.05 | 0.16 / 0.07 / 0.38 | 0.07 | 0.14 |
| the capture (`capture`) | 0.55 / 0.11 / 2.11 | 0.17 | 1.36 / 2.35 / 4.54 | 1.14 | 0.14 |

- **On foot in the rain, the cost is the voxel walk**: 2.1 of L8's 2.8 ms
  at p50 and 7.5 of 9.5 ms at p99. Rank 1 stands.
- **In flight, the cost is wet ground's screen trace, not the sea's**: 1.1
  ms of 1.4, all of it in the terrain pass; the sea's trace is 0.14 ms of
  GPU. Far wet ground runs the trace as near wet ground does, and a far
  fragment's 16.75 m march projects to a few texels.
- **The capture itself costs 0.03-0.04 ms of GPU** (`ssr_capture`).
  Switching it off saves what its two readers then skip, not its own pass.
  Rank 3 is not worth a fix: the dry case it targets saves 0.03 ms.
- The p95 column of the sea and capture rows on the stroll is inside the
  stroll's p95 spread (1.0 ms between `all-on`'s two runs).

### GPU per pass (task 1.2)

At latency 2, p50 ms (`docs/benchmarks/2026-11-12-frame-latency/`):

| pass | rain-stroll | storm | walk | town |
| --- | ---: | ---: | ---: | ---: |
| terrain (`main_transparent_pass_3d`, with L8's traces) | 4.34 | 2.97 | 2.37* | 1.86 |
| sun's cascades (`planet_shadow`) | 2.03 | 0.96 | 1.97* | 2.09 |
| rain volume (`rain`, the lit rain fog) | 1.44 | 0.00 | 0.15* | 0.66 |
| opaque (`main_opaque_pass_3d`: towns, trees, props) | 0.57 | 0.00 | 0.01* | 0.74 |
| compose (`water_compose`: sea, distance fog) | 0.07 | 0.49 | 0.06* | 0.12 |
| lens (`water_lens`) | 0.29 | 0.09 | 0.00* | 0.00 |
| cloud march + puffs | 0.03 | 0.31 | 0.02* | 0.03 |
| bloom (`lite_bloom`) | 0.09 | 0.08 | 0.07* | 0.07 |
| tone (`tonemapping` + `upscaling`) | 0.05 | 0.05 | 0.04* | 0.04 |
| reflection capture (`ssr_capture`) | 0.03 | 0.04 | 0.02* | 0.02 |

(*) `walk` at latency 1; the passes' GPU time does not depend on the queue.
The tone pass of L6 (`tone_grade.wgsl`, a `FullscreenMaterial`) records no
span of its own.

- **Bloom is 0.07-0.09 ms of GPU**, not the 0.33-0.36 ms its toggle took
  off the wall at latency 1. The rest of that was its CPU half (ten
  uniform buffers and bind groups made a frame) on a frame where CPU and
  GPU time added up. Rank 7's GPU half has almost nothing to win; its CPU
  half (uniforms made once) is what is left.
- **The rain volume is the one full-screen fog-like pass with a real
  price**: 1.44 ms on the stroll, 0.66 ms in the town walk.
- **The sun's cascades cost 1-2 ms everywhere on foot.** They are not the
  lighting model's (L3's soft filter costs nothing measurable) and not in
  this change; recorded for the next reader.

## The worst case: what the frames over 16.7 ms are (measured)

The owner, 2026-11-12: "get worst case to 60", p99 within 16.7 ms in every
scenario at latency 2. At latency 2 (`2026-11-12-frame-latency`, before
2.1) p99 was 23.3 on `rain-stroll`, 16.9 in the storm, 18.7 in the town
walk, 10.5 on the walk. `tools/tail_classes.py` puts every frame over 16.7
ms after the warm-up in one class: a fine-set **landing** (the drawn set
changed this frame or the two before), a **logged** piece of work within
0.15 s (`SPENT ...`, the towns' ground), **gpu** (the GPU's own time over
the floor), **gpu-paced** (a long frame beside a short one whose mean is
within 1.5 ms of the GPU's time: the GPU set the pace and the queue of two
split it unevenly, e.g. 22 ms then 5 ms against 13 ms of GPU), else
**cpu** (nothing logged, GPU under the floor). Three runs each, latency 2:

| scenario | frames over 16.7 ms | gpu-paced | gpu | cpu | `SPENT town bake` | landing | other logged |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| rain-stroll | 601 of 12,626 (4.8%) | **412** | 4 | 107 | 60 | 6 | 12 |
| storm | 187 of 18,331 (1.0%) | 50 | 0 | **79** | 33 | 3 | 22 |
| town | 590 of 47,459 (1.2%) | **260** | 45 | 76 | **194** | 8 | 7 |
| walk | 192 of 82,791 (0.2%) | 2 | 13 | **152** | 0 | 17 | 8 |

- **On foot in the rain the tail is the GPU.** Two thirds of the slow
  frames are a GPU-bound frame (12-15 ms of GPU in the lane's slow
  direction) paced unevenly. Lowering the GPU time is the lever: 2.1, then
  the rain volume (1.44 ms) and L8's traces at half resolution, and the
  sun's cascades (2.0 ms; not the lighting model's).
- **In the storm and on the walk the tail is unlogged CPU spikes** of
  30-220 ms, about one frame in 125 in the storm. In the storm 34 of a
  run's 48 came while a fine-set rebuild was in flight (one is in flight
  on 97% of its frames, so that says little); on the walk 11 of 50. They
  are not named by anything the game logs. Next: a trace of the storm
  flight (`--features trace`, `tools/trace_rank.py`) to name them, then
  move or spread that work. The "flight tails are not the lighting
  model's" note under "Not ranked" is reopened by this goal.
- **In the town the tail is the town's own work**: `SPENT ... town bake`
  (2-3 ms each, many a second) and GPU-paced frames. Town batching and
  the town's shadow casters are `crab-breeze-lessons` ranks 2 and 5,
  taken by the other agent (not this change).

## The fixes, measured (sitting 2, 2026-11-12)

One sitting, five variants interleaved, 2 repeats, latency 2 for all
(`output/perf/2026-11-12-fixes`, `docs/benchmarks/2026-11-12-fixes/`):
`base` (`57d94b5` at `--frame-latency 2`), `fix21` (`e9ed506`), `loading`
(the loading screen cherry-picked, `e8016e0`), `fix22` (`2185478`, same exe
as `loading`, its shaders), `fix245` (`6ec3ff8`). Each fix is read against
the variant before it **on the same side of the loading screen**: the
builds before it count their warm-up from the first frame and then freeze
about 6 s for the towns' ground, those after from `READY`, so across that
line they measure different stretches of a scripted route. The `walk`
cannot be read across it at all: with the loading screen the world opens
in a full storm where it opened dry (cover 1.00, rain 1.00 against 0.69,
0.00; reported, being fixed in `loading-screen`). p50 / p95 / p99 ms:

| fix | scenario | before | after | GPU pass before -> after |
| --- | --- | ---: | ---: | --- |
| 2.1 walk ends early | rain-stroll | 10.89 / 17.48 / 24.42 | **10.34 / 16.80 / 21.70** | terrain 4.43 -> 3.99 |
| 2.1 | storm | 6.87 / 12.90 / 19.34 | 6.73 / 13.03 / 17.99 | terrain 3.10 -> 3.05 |
| 2.2 screen traces skip | storm | 6.51 / 11.26 / 14.85 | **6.25 / 10.79** / 15.40 | terrain 2.80 -> 2.59 |
| 2.2 | rain-stroll | 10.44 / 17.20 / 22.13 | 10.35 / 17.13 / 21.98 | terrain 4.61 -> 4.49 |
| 2.4 + 2.5 (CPU) | storm | 6.25 / 10.79 / 15.40 | 6.24 / 10.42 / 13.76 | puff build 0.41 -> 0.28 ms mean, 0.58 -> 0.40 max |
| 2.4 + 2.5 | rain-stroll | 10.35 / 17.13 / 21.98 | 10.39 / 17.54 / 21.57 | (none) |

p50 spread between the two repeats: 0.02-0.18 ms (stroll, storm).

- **2.1 took 0.55 ms off the stroll's p50 and 2.7 ms off its p99**, 0.44
  ms of GPU in the terrain pass. Less than the walk's whole 1.5 ms: rays
  along the lane stay below the reach top of the houses' terraced ground
  for most of their 48 m.
- **2.2 took 0.26 ms off the storm's p50 and 0.47 off its p95**, 0.21 ms
  of GPU; the far wet ground's trace that is left lands outside three
  texels.
- **2.4 + 2.5 cut the puffs' build by a third** (0.41 to 0.28 ms a frame);
  wall time does not show it at latency 2, where the main thread is not
  the storm's limit at p50. Its p99 13.8 against 15.4 is one sitting.
- **The storm is now inside the floor at p99** (13.8 ms); the stroll is
  not (21.6 ms), and its p50 sits at its GPU time (10.4 against 9.8 ms).

## The checkpoint (task 3.1, measured 2026-11-12)

`docs/benchmarks/2026-11-12-lighting-perf/`
(https://claude.ai/artifact/Up2NNA9VWbFYA8inrfTuqH): `f15a647`, the branch
before this change (`86361b1`) and the branch at `cdf0317` (latency 2, 2.1,
2.2, 2.4, 2.5, 2.10, the loading screen), every scenario, 2 repeats. p50 /
p95 / p99 ms, `f15a647` then now:

| scenario | `f15a647` | now | p99 within 16.7 ms |
| --- | ---: | ---: | --- |
| clouds | 9.89 / 15.85 / 24.05 | 6.27 / 10.55 / 13.62 | yes |
| storm | 9.93 / 16.20 / 24.74 | 6.17 / 10.77 / 13.74 | yes |
| far-side | 6.57 / 11.95 / 15.94 | 4.08 / 6.98 / 9.04 | yes |
| rain-stroll | 14.03 / 18.30 / 27.93 | 9.98 / 15.63 / 20.37 | **no** |
| rain-stroll, 22:00 | 14.13 / 21.65 / 29.42 | 9.84 / 16.20 / 21.12 | **no** |
| town (rain) | 17.33 / 24.87 / 29.99 | 9.90 / 16.54 / 22.01 | **no** |

- **What is left over the floor is on foot in the rain, and it is the GPU**:
  the stroll's GPU p50 9.5 ms against a wall p50 of 10.0. The model still
  costs 2.8 ms there (9.95 against 7.18 ms p50 with `PBD_LIGHTING=today`
  in the same exe); the spec's 1.0 ms is not met.
- **2.10 saved nothing measurable** (the rain pass 1.50 ms against 1.44):
  the snow kind's loop and the shafts' matrix products were not the cost.
  The cuts are exact and stay; the rain volume's cost is its shells' haze
  and streaks per pixel, which is the half-resolution task.
- **With vsync on** (the game as played), frames over 16.7 ms: 3.9% on the
  stroll, 1.0% in the town, 0.5% in the storm.
- **The spikes over 30 ms** (`perf_suite.py` now says which recur at the
  same place in the other runs): in the storm four a run recur at the same
  route metres in every build (3544 m, 110 m above the ground; 3808 m, 58
  m up, a fine-set landing with an 11.6 ms upload; 4122 m, 36 m up; and
  touchdown), 30-65 ms each; the rest, and nearly all the walk's, fall at
  different places in each repeat. **Closing the Solitaire game changed
  nothing** (the owner's question): 14-30 spikes a walk run with it
  closed against 23 open, same exe, p99 20.2-20.7 against 20.3. They are
  not where p99 is set (the walk's and the storm's p99 are inside the
  floor), so no fix effort goes to them; the four descent spikes are
  written up here and wait for a trace.
- **The walk** opens in a storm with the loading screen (a bug being fixed
  in `loading-screen`), so its numbers here are a walk in the rain.

## The rain volume at half size (measured 2026-11-12)

`docs/benchmarks/2026-11-12-rain-half/`. **The rain that is drawn by
default is the volume** (`rain_renderer: Volume`: sixteen jittered steps of
streak noise to the ground), not the shells; 2.10's cuts were to the
shells, which is why they saved nothing. The half-size pass gathers either
renderer's light and transmittance into a target at `rain_render_scale`
(0.5) and lays it over the scene at full size, weighed by depth. One exe,
the scale the only difference: `rain-stroll` p50 / p95 / p99 10.03 / 15.73
/ 20.74 to **8.69 / 14.02 / 18.56** ms, the rain 1.49 to 0.45 ms of GPU;
storm 6.03 / 10.40 / 13.88 to 5.73 / 9.57 / 12.90. The shots are hard to
tell apart side by side, with no halo at an edge; most moved pixels move by
2/255 or less, 17% by more at night, where the volume's grain is now per 2
x 2 pixels. Kept, as a recommendation taken (ask only with screenshots):
the shots are in `docs/screenshots/lighting-perf/2.9-rain-half/` for the
owner, and `rain_render_scale: 1.0` puts the full-size pass back.

## Half resolution (owner, 2026-11-12)

The owner: "consider like half resolutions for expensive stuff like bloom
and fog rendering". Priced against the columns above:

| pass | resolution now | GPU p50 | half resolution |
| --- | --- | ---: | --- |
| bloom | built from a quarter-size image | 0.07-0.09 ms | nothing to win (at most ~0.05 ms) |
| cloud march | 0.4 scale, history, depth-aware upsample | 0.17 ms (storm) | already reduced |
| reflection capture | 960 x 600 | 0.03 ms | already reduced |
| compose (distance fog, sea) | full | 0.07-0.49 ms | the sea's part is per sea pixel and is L8's sea trace; the fog is a few ALU per pixel |
| rain volume (lit rain fog) | full | **1.44 ms** stroll, 0.66 town | **candidate** (rank 3 below) |
| L8's traces (in the terrain pass) | full, per wet fragment | **2.06 ms** stroll, 1.19 storm | **candidate after ranks 1-2** (rank 5) |

- **The rain volume at half resolution** (task 2.9): march the shells into
  a half-size target (colour and transmittance), then composite at full
  size with a depth-aware (bilateral) upsample, the four nearest
  half-size texels weighted by how close their depth is to the full-size
  pixel's, so a wall's silhouette in front of rain does not take a halo of
  rain from the half-size texel that straddles it. The streaks are thin
  lines; at half size they may soften or alias, which the shots decide.
  Expected (**estimate**): 0.7-1.0 ms of the stroll's 1.44.
- **How the rain volume goes to half resolution** (the plan, 2026-11-12,
  after 2.10 showed its cost is the shells' per-pixel haze and streaks):
  - `water.wgsl`: `rain_shells` returns what it gathers, `light` (rgb) and
    `transmittance`, instead of `scene*transmittance + light`; the full-
    size `rain` entry keeps composing them as today (so `PBD_PASS_OFF` and
    the `today` model are unchanged).
  - A `rain_half` entry writes `vec4(light, transmittance)` into a half-
    size `Rgba16Float` target (`planet_water.rs`, made beside the clouds'
    history and resized with the view), reading the full-size depth at
    the texel's own pixel for `far`.
  - A `rain_compose` entry at full size reads the scene, the half-size
    target and the depth: the four half-size texels round the pixel, each
    weighted bilinearly and by `exp(-|d_texel - d_pixel| / (0.02 d_pixel
    + 0.1 m))`, the full-size depth loaded at each texel's own pixel, so a
    wall in front of rain takes no rain from the texel past its edge; a
    pixel whose four weights all fall under 1e-3 takes the nearest texel
    by depth. Then `scene*T + light` as today.
  - The jitter that today varies per full-size pixel varies per half-size
    texel; the streaks are 0.004 rad wide (about 3-4 px at 1440 x 900), so
    at half size they stay over a texel wide.
  - Shots, against a control: the Holbrook lane in rain by day both ways,
    at night (the lamps' glow in the rain), a wall's edge against rain, a
    window, and the shore at 300 m in rain; pixel-diffed and shown to the
    owner, since this one moves pixels.
  - Expected (**estimate**): 0.8-1.1 ms of the stroll's 1.50 ms, one
    quarter of the pixels plus a cheap compose.
- **L8's traces at half resolution** (task 2.8, Crab Breeze's read agrees):
  the traces run inside the terrain's fragment shader, so halving them
  means a pass of their own (a half-size target of reflected colour and
  hit weight, from the depth and normal the terrain already writes) and a
  depth-aware upsample where the terrain reads it. That is a new pass and
  a split of the terrain shader. It waits for ranks 1-2: if they leave
  L8 under ~0.5 ms, it is not worth the pass.

## L8's traces at half size: the pass layout (task 2.8, plan 2026-11-12)

What is left over the floor is on foot in the rain (stroll p99 18.56 ms by
day, ~21 at 22:00; the town ~21), GPU-bound, and the largest piece of GPU
left there is wet ground's two traces inside the terrain's fragment shader
(the walk ~1.5 ms and the screen trace ~0.2 ms of the terrain pass's ~4 ms
on the stroll, measured in 1.1 before 2.1 took ~0.4 ms off the walk).

**Why it needs a pass of its own (read).** The traces run per wet fragment
inside `fragment()` of `planet_surface.wgsl`. Tracing one pixel in four in
place saves nothing: a warp waits for its slowest lane, and neighbouring rays
in a puddle walk about as far as each other. The rays have to be packed
densely into a pass of their own. The trace needs a wet pixel's position,
its reflected ray (rippled), and its column slot. Only the terrain's fragment
shader knows the last two, and the composite needs the fresnel weight and
the sky/glint terms that the hit replaces. So the terrain writes them out
beside its colour. The alternative, a half-size prepass that draws the
terrain again, costs a second geometry pass (the sun's cascades show a
terrain draw is ~0.5 ms) and has to repeat the wet code in a second entry
point. It was not chosen.

**Writing them out needs the terrain in a render pass of its own.** It draws
today as a `Transparent3d` phase item (`queue_planet`, `distance: f32::MIN`,
so first) inside Bevy's `main_transparent_pass_3d`. That pass has one colour
attachment, so the terrain moves into a `ViewNode`, `planet_terrain`, which
runs between `Node3d::MainTransmissivePass` and `Node3d::MainTransparentPass`.
That keeps the order exactly: after the opaque towns, before every
transparent mesh. The node owns the four indirect draws (surface, foliage,
clutter, columns) that `DrawPlanetIndirect` issued, with the same pipeline
specialised in `queue_planet` (now kept on the view, not added to the phase).

**The passes, all in that node** (`planet_reflect.rs`; per view, sized with
the view; MSAA is 4x on the camera, so the G-targets are too):

1. **`planet_terrain`**: the terrain's pass. Colour (the view's MSAA target,
   loaded, resolved as Bevy's passes do) plus, while `reflect_render_scale`
   is under 1, two more targets, cleared each frame:
   - `reflect_req` `Rgba16Uint` (MSAA, read at sample 0): the slot (two
     halves, the top bit of the upper one says "a request") and the
     reflected ray, octahedral, two `unorm16`;
   - `reflect_delta` `Rgba16Float` (MSAA, resolved to a 1x copy):
     `k*B` (rgb) and `k*fr` (a), where `fr` is the film's weight, `B =
     fr*env + S + 0.8*L` (the sky the hit would replace, the sun glint `S`
     and lamp glints `L` that the hit hides), and `k = 1 - 0.55*fog` (the
     haze `finish()` lays over it).
   The fragment shader writes them through `var<private>`s that the wet
   path and `finish()` set; a request draws the miss (`hit = 0`), exactly
   today's colour where nothing is hit. `fragment` (one target) stays as
   today's entry point, and `fragment_split` wraps the same body.
2. **`reflect_half`**: a full-screen pass at `reflect_render_scale` (0.5:
   720 x 450 at 1440 x 900), new entry point `reflect_gather` in
   `planet_surface.wgsl`, so `reflect_trace` and `ssr_trace` are the same
   functions with the same bindings (groups 0-2 as the terrain's, group 3
   the G-targets, the depth and a small uniform). Each texel takes the first
   requesting pixel of its 2 x 2 (sample 0), rebuilds its position from the
   depth (`body_from_clip`, made in f64 and relative to the eye), biases it
   2 cm toward the eye (not along the normal, which is not written), reads
   the cover there, and runs the walk and then the screen trace exactly as
   the fragment did. Out: `reflect_hit` (`Rgba16Float`: colour, weight) and
   `reflect_aux` (`Rgba16Float`: the ray, the distance from the eye; 0 = no
   request).
3. **`reflect_compose`**: a full-screen pass over the view's MSAA colour,
   additive blend, new `reflect_compose.wgsl`. Per pixel with a request
   (sample 0), the four half-size texels round it, each weighed bilinearly,
   by depth (`exp(-|d_t - d| / (0.02 d + 0.1 m))`, as the rain's), and by
   direction (`smoothstep(0.6, 0.95, dot(r_t, r))`, so a wall's ray does
   not lend its hit to the floor at the wall's foot). With `hH` and `h` the
   weighed means of `hit.rgb*hit.a` and `hit.a`, it adds `k*fr*hH - h*k*B`.
   That is what the fragment's `color*(1-fr) + mix(env,H,h)*fr + S*(1-h) +
   L*(1-0.8h)`, through `finish()`, adds over the miss, exactly, because
   each term is linear in `h` and the MSAA resolve is linear. If no weight
   is over 1e-3, nothing is added (the miss).

**`reflect_render_scale`** (`lighting.ron`, 0.25..=1, default 0.5): at 1.0
the terrain keeps one target and traces inline as today, and passes 2-3 do
not run. That is the A/B in one exe, and the regression check for the move
into `planet_terrain`: at 1.0 the shots must match the build before it to the
pixel (`capture_repeat.py --against`).

**The sea's trace stays in `water_compose`.** It is 0.14 ms of GPU in the
storm and nothing on foot (1.1). A pass for it does not pay.

**Frame log**: new columns `planet_terrain`, `reflect_half` and
`reflect_compose`. `main_transparent_pass_3d` no longer holds the terrain.

**Expected (estimate)**: a quarter of the rays and none in overdrawn
fragments, against the extra targets' writes (MSAA-compressed, ~0.1 ms) and
the compose (~0.1 ms). That is 1.0-1.4 ms of the stroll's GPU. Memory: two
MSAA targets of 8 bytes a sample, ~83 MB at 1440 x 900, plus ~10 MB.

**Look risk**: the hit is now per 2 x 2. The rings still bend the sky and
the fresnel at full size, but what the reflection shows is softer. At an
edge, the depth and direction weights keep a hit from crossing a silhouette.
A pixel that only other surfaces' texels surround shows the sky (the miss).
Shots (owner's look, since this moves pixels): the Holbrook lane in rain at
10:00 and 22:00 both ways, a puddle under a wall, a harbour from 30 m and
300 m, the town in rain, full against half, with a run-to-run noise pair.

### Built and measured (2026-11-12)

`docs/benchmarks/2026-11-12-reflect-half/`: one exe, `PBD_REFLECT_SCALE=1`
against 0.5, 2 repeats interleaved, latency 2. p50 / p95 / p99 ms:

| scenario | full (inline) | half | terrain GPU, full -> half (+ trace + compose) |
| --- | ---: | ---: | --- |
| rain-stroll, 10:00 | 8.27 / 12.94 / 16.79 | **7.03 / 10.46 / 12.68** | 3.75 -> 2.13 (+0.35 +0.05) |
| rain-stroll, 22:00 | 8.52 / 14.34 / 17.28 | **7.14 / 11.04 / 13.26** | 3.77 -> 2.03 (+0.33 +0.05) |
| town (rain) | 8.19 / 14.04 / 17.23 | **7.29 / 10.65 / 12.73** | 3.98 -> 2.85 (+0.26 +0.03) |
| storm | 5.53 / 8.44 / 11.73 | **5.01 / 7.57 / 9.86** | 2.38 -> 1.44 (+0.09 +0.04) |
| walk | 5.18 / 6.94 / 8.25 | 5.50 / 8.05 / 9.69 | 2.25 -> 2.13 (+0.02 +0.01) |

- **The goal is met in this sitting**: p99 within 16.7 ms in every scenario.
  On foot in rain the frame gains ~1.3 ms of GPU net, as estimated.
- **The walk's difference is outside this change**: its slower half run's
  extra time is the sun's cascades and the transparent pass. Its terrain pass
  is faster, and the split's own passes cost 0.03 ms where little reflects.
  The town walk is bimodal in both arms (its wet mode gains 4.5 ms at p99).
- **One deviation from the plan**: the gather starts each ray 2 cm toward
  the eye **and 2 cm up**. Toward the eye alone is a millimetre up at a
  glancing view, and a floor rebuilt from depth a hair low started the walk
  inside its own layer: the first shots were darker by 1.3/255 in the
  middle distance, where the walk hit the ground beside it.
- **At 1.0 the move is exact**: the lane by day and night and the harbour
  match the build before it to the pixel.
- **The look**: 6-26% of a rain scene's pixels move, mostly at the ripples'
  scale (the reflection's content is per 2 x 2). No halo at the wall's foot
  or the houses' silhouettes. Kept as a recommendation taken (ask only with
  screenshots); survey T16; `reflect_render_scale: 1.0` puts it back.

### Worst case 60, met (measured 2026-11-12)

`docs/benchmarks/2026-11-12-worst-case-60/`
(https://claude.ai/artifact/DiyFFNobHpBLsuCpvB5ssk): `f15a647` (main, its
own latency 1) against the branch at `c05ae5f`, every scenario, 2 repeats.
p50 / p95 / p99 ms, main then the branch:

| scenario | `f15a647` | branch |
| --- | ---: | ---: |
| rain-stroll | 13.88 / 17.94 / 25.38 | 7.18 / 10.70 / 12.86 |
| rain-stroll, 22:00 | 13.66 / 18.20 / 27.07 | 6.79 / 10.92 / 13.10 |
| town (rain) | 16.37 / 20.17 / 23.94 | 7.32 / 11.30 / 14.22 |
| storm | 9.55 / 15.42 / 22.69 | 5.01 / 7.54 / 9.85 |
| clouds | 9.57 / 15.01 / 22.74 | 4.98 / 7.53 / 9.55 |
| far-side | 6.14 / 11.03 / 14.38 | 3.78 / 5.92 / 6.95 |
| walk | 9.14 / 11.43 / 14.45 | 5.65 / 8.38 / 9.93 |

Every scenario is inside 16.7 ms at p99, so the tail classes are not
re-run and no further lever is taken here. What is still over a budget is
p95 against 120 fps (8.3 ms) on foot in rain and in the town.

## Ranked optimizations

**Re-ranked on the split (decision 1)**, 2026-11-12. In order of work:

1. L8 voxel walk ends when nothing can be hit (2.1): 2.1 ms p50 on foot in
   rain.
2. Both screen traces skip a march that projects inside the texels it
   skips (2.2): 1.1 ms p50 in the storm, from wet ground's trace.
3. The rain volume at half resolution (2.9): 1.44 ms of GPU on the stroll.
4. Cloud-puff sites (2.4), env reads and `near_lamps` (2.5): main-thread
   time, which latency 2 now overlaps with the GPU.
5. L8's traces at half resolution (2.8), if 1-2 leave enough.
6. Bloom's uniforms made once (2.6's CPU half); its composite fold has
   ~0.05 ms of GPU to win.
7. Not pursued: the capture skip (2.3; 0.03 ms) and the lamp bake (2.7).

The sections below are the plan as first written (rank numbers as then).


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
