# Puff frustum cull and rain detail near the eye (2026-10-29)

Two changes, measured together against the commit before them:

- `puff-frustum-cull` (owner: "Cloud puff frustum culling then. Make sure not
  to cull stuff whose bounding box intersects with frustum").
- `wet-near-detail` (owner: "The wet surface can also be simplified 20m from
  the camera").

Apple M4 Max (Metal), release builds, window 1440 x 900, uncapped, clock at
10:00, first 10 s thrown away, 2 runs of each variant interleaved, GPU apps
closed (Steam, Chrome, Spotify, VS Code). `before` is `a23842b` with its own
assets; `after` is the working tree with both changes (and the walk script's
dig fix, which no scenario here exercises). All 12 runs were valid.

`rain-stroll` is new: up and down Holbrook's lane in a forced rain
(`--walk --at 29.69673 1.19287 --yaw 179.4 --rain 1 --stroll 45
--stroll-leg 7`), wet ground near and far on a script that never digs.

## Findings

**Corrected the same day.** The first reading of this sitting credited the
2.5 ms to the puff cull. A second sitting split the two changes, on the
after build with the wet cut switched off by its own setting
(`rain_detail_reach_m: 100000`, laid over the checkout for those runs only):

| Scenario | cull + cut | cull only | the cut's share |
| --- | ---: | ---: | ---: |
| clouds p50 | 17.17 | 19.27 | **2.10 ms** |
| storm p50 | 17.14 | 19.18 | **2.04 ms** |
| clouds p95 | 32.3 | 36.3 | 4.0 ms |

Repeats agree within 0.05 ms (`split/suite.json`).

- **The flights are 2.5 ms faster at p50**: clouds 19.74 -> 17.22 ms, storm
  19.70 -> 17.12 ms, p95 4-5 ms lower.
- **About 2.1 ms of it is the wet cut.** From the air nearly every wet pixel
  is past 20 m, and each one used to compute the rippled sheet and the ring
  layers and then fade them to nothing by its footprint. The GPU time up to
  the composite falls by the same 2.0 ms (15.0 -> 13.0).
- **About 0.45 ms is the puff cull** (19.74 before against 19.27 cull-only,
  across two sittings, so read it as roughly half a millisecond). That fits
  what the puffs cost at all: about 1.9 ms in `2026-10-28-cloud-puffs`.
  The cull keeps 250-400 of the 450-900 puffs over the horizon, from 2,080
  built; the build costs the CPU 0.15-0.18 ms.
- **On the rain stroll the cut does not show at p50** (19.27 -> 19.15, inside
  the 0.3 ms spread): in a lane much of the wet ground is inside 20 m, and the
  stroll's frame is dominated by other work.
- **Neither change is visible where it should not be.** The same cloudy
  frame before and after (`docs/screenshots/puff-frustum-cull/`) matches
  across the sky; 0.6% of pixels differ by more than 8/255, all in far wet
  grass, which is the wet cut. The street stills
  (`docs/screenshots/wet-near-detail/before-after.jpg`) show rings ending by
  20 m and the ground past it wet, puddled and mirroring.

## Limits

- One machine, one sitting.
- The stills are two `--capture` frames 900 steps in; the rain's own
  animation differs between them, so the difference image
  (`difference-x6.jpg`) also lights the near rings. The far grass past 20 m
  shows the real change: it loses the faint grass rings.
- "GPU clouds" on this Mac spans the GPU work up to the composite, not only
  the clouds (see `2026-10-29-lighting-full`).
- The CSVs and logs are local (`output/perf/2026-10-29-cull-wet`).

## clouds

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| before | 2/2 | 45.91 | 19.77 | 37.98 **over** | 57.03 **over** | 61.78 | 62.82 | - | 15.48 / 18.79 |
| after | 2/2 | 52.17 | 17.23 | 32.83 **over** | 50.16 **over** | 54.96 | 59.92 | - | 13.03 / 15.36 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| before | 54.61 | 18.54 | 28.73 **over** | 38.08 **over** | 42.80 | - / - | 15.08 / 16.61 |
| after | 61.82 | 16.30 | 19.74 **over** | 32.02 **over** | 34.75 | - / - | 12.97 / 13.98 |


## storm

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| before | 2/2 | 46.54 | 19.72 | 37.20 **over** | 55.29 **over** | 64.78 | 69.10 | - | 15.40 / 18.56 |
| after | 2/2 | 52.62 | 17.14 | 33.06 **over** | 51.07 **over** | 57.03 | 60.00 | - | 13.00 / 15.40 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| before | 54.64 | 18.48 | 29.93 **over** | 37.97 **over** | 40.88 | - / - | 15.09 / 16.62 |
| after | 62.03 | 16.20 | 25.11 **over** | 33.83 **over** | 35.73 | - / - | 12.92 / 13.95 |


## rain-stroll

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| before | 2/2 | 45.98 | 19.39 | 43.36 **over** | 57.38 **over** | 60.77 | 61.88 | - | 7.14 / 18.26 |
| after | 2/2 | 46.75 | 19.30 | 47.37 **over** | 55.49 **over** | 58.76 | 59.57 | - | 6.97 / 16.89 |
