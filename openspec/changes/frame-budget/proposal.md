# Proposal: frame-budget

## Why

The owner's budgets are p95 within 8.3 ms (120 fps) and p99 within 16.7 ms
(60 fps). The `cloud-puffs` benchmark on the Mac (Apple M4 Max, Metal, a
1440 x 900 window = 2880 x 1800 Retina pixels, `--no-vsync`, release build,
clock pinned at 10 h; `output/perf/2026-10-28-cloud-puffs*`) shows the frame
missing both with the clouds switched off (`PBD_NO_CLOUDS=1`):

| Scenario, clouds off | p50 | p95 | p99 | max |
| --- | ---: | ---: | ---: | ---: |
| `clouds` | 8.9 ms | 26.5 | 42.5 | 9,170 |
| `storm` | 16.8 | 337-417 | 677-714 | 9,673 |

So most of what misses the budget is not the clouds. This change finds where
the rest goes and ranks what to do about it. It is a write-up: no source,
shader, config or tool is changed here.

## What was found

Every number below was read off the existing frame logs and logs with
`tools/frame_graph.py` and short scripts over the CSVs; nothing was re-run.
Measured means read off a log. Inferred means argued from the code and
consistent with the logs, but not isolated by an A/B run. The design has the
detail.

1. **The storm numbers are not the storm. They are the window losing focus
   (measured), plus a throttle (inferred).** Every run from 16:12:53 on has its
   median at 60 Hz (p50 16.7-17.4 ms, 60-83% of frames between 15 and 18.5 ms).
   That covers `storm` and `cloud-hop` alike, with clouds off, puffs or the
   march. Every run before then has not (all six `clouds` runs: 13% of frames
   in that band). The march adds 10-15 ms of measured GPU cloud time to a storm
   run and the median does not move (17.4 against 16.9 ms), so the 60 Hz is a
   pace, not a cost. Bevy's default `WinitSettings::game()` drops an unfocused
   window to `reactive_low_power(1/60 s)`, and the game does not override it.
   The 600-1,100 ms frames start about 30 s into each of these runs, along with
   fine-set rebuilds that take 3.5-10 s instead of 0.4-0.6 s. That fits macOS
   throttling a background process (App Nap), but it is not proven. The
   `cloud-puffs` storm and cloud-hop rows therefore measure a backgrounded
   window, not the game.
2. **A fine-set landing re-bakes every standing town's lighting (measured
   correlation, mechanism from code).** The first frame after a landing costs
   73-103 ms, and often the next one 30-49 ms, while two or three towns stand.
   It costs 20-25 ms with one town and 2-8 ms with none, even for a set with 0
   columns, 2 km above any town. `bake_town_outsides` keys its bake on the
   set's `Arc` pointer, so every landing re-bakes every vertex of every
   standing town: up to 200,000 a frame, each one a terrain-noise height read
   and a field read. On the `clouds` flight that is 42 of the 91 frames over
   33 ms.
3. **Opening a world freezes for 9.2 s (measured), and the installed town
   ground makes every height read about 11x dearer (inferred).** The first
   planet takes 0.81 s. After the towns' ground is installed, the synchronous
   `rebuild_planet` takes 9.16 s on the main thread. Each height read then
   takes a process-wide `RwLock` and compares a whole `TerrainConfig`. It
   lands in every run 1.5-2 s after the 10 s warm-up, which is why every
   run's max is 9.2-9.7 s.
4. **Near the ground the frame doubles (measured), around the rain gate and
   the finest band (inferred).** On the focused `clouds` flight, with no
   rebuild and no landing, frames are p50 16 ms / p95 33 ms below 200 m, about
   12 ms at 240-320 m, and 8.4 / 15 ms above 320 m. The steps sit at
   `rain_lod_alt_m` (200 m), where the CPU shower mesh, the lens drops and the
   lens blur turn on, and at 300 m, where level 11 and the column tier go.
5. **Even the quiet frame is over budget at altitude (measured), and on
   Metal the rig cannot say where it goes (measured).** p50 is 8.4 ms and p95
   15 ms above 320 m with no event at all, on 5.2 Mpx, four times the RTX
   3070 bench's pixel count. The `gpu_total_ms` column reads 0 on Metal
   (Bevy's in-pass timestamps), so only the clouds' own pass-boundary timer
   reports.

## What changes

The ranked plan, measurement first (detail and risks in `design.md`):

1. **Measure a focused window only.** The frame log records focus and
   occlusion. Under `--frame-log` the game keeps updating while unfocused.
   The suite invalidates any run that lost focus and holds off App Nap. Then
   the `storm` and `cloud-hop` rows are run again.
2. **GPU time per pass on Metal.** The clouds' pass-boundary timer is
   extended to compose, rain, lens and the scene passes, and
   `PBD_PASS_OFF=shower` is added beside the existing `rain`, `lens` and
   `mist` switches.
3. **Bake a town only when its light can have changed.** The bake is keyed
   on the field around the town (light version, and whether the tier's
   columns reach the town), not on the set's pointer, and runs off the main
   thread.
4. **No freeze on opening a world.** The ground is read without a lock per
   height (one snapshot per build), and the town install's rebuild runs on
   the pool behind the old planet, as a fine set does.
5. **Rain near the ground on the GPU.** The shower's streaks become an
   instanced draw computed from index and clock, and the rain volume's
   per-step gradient noise moves to a baked texture or a reduced-resolution
   march. Which of these comes first is decided by the measurements from
   item 2.
6. **A 3D render scale for Retina.** This is a quality knob, chosen by the
   owner on screenshots.
7. **Per-frame work the pinned clock hides.** The field light per mesh and
   the material churn when the sun moves, measured in a running-clock
   scenario.

## Not in this change

- The clouds' own cost (`cloud-puffs`, `cloud-budget`).
- Anything that trades a fade for a cut (CLAUDE.md priority 1). Spreading the
  town bake over more frames is allowed only if the light eases. It may not
  step.
- Generation changes. Nothing here may change a height, a record or a digest.
  The lock change is a read-path change, held to the existing digest tests.
- The frame cost in a Claude Code cloud session (no GPU). Every measurement
  here is the Mac's or the owner's desktop.
