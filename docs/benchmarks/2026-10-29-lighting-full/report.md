# lighting-model, every part: a clean sitting (2026-10-29)

Apple M4 Max (Metal), release build of `b85718f` (the tree's later edits were
docs, tooling and unbuilt code; the exe is that commit's), window 1440 x 900,
uncapped (`--no-vsync`), clock pinned at 10:00, first 10 s of each run thrown
away. Three scenarios: `clouds` (`--route scenic`), `storm` (`--route scenic
--rain 1`) and `walk` (`--walk-distance 1000`). Each variant ran twice,
interleaved, with the screen unlocked and **every GPU app closed** (Steam,
Chrome, Spotify, VS Code; owner: "close out any apps using the gpu before
doing your run"). All 28 runs were valid.

A first full sitting the same morning ran with Steam at 173% CPU in the
background. Its flights agree with this one to 0.3 ms at p50 (they are
GPU-bound); its walk does not (today's walk p99 31.9 ms there against
16.2 ms here), so this sitting replaces it.

## Findings

- **The new model costs about 4.1 ms at p50 in flight and 1.6 ms on foot.**
  Clouds 15.9 -> 20.0 ms, storm 15.5 -> 19.6 ms, walk 10.1 -> 11.7 ms. The
  spread between a variant's two runs is 0.2 ms or less on most flights,
  0.8 ms at most (today's clouds), and 0.7 / 1.1 ms on the walks.
- **L8 (reflections) is the one part that costs measurably: 1.3-1.9 ms** at
  p50 (clouds 20.0 against 18.1, storm 19.6 against 18.3), and most of the
  tail: p99 about 55 against 47 ms. It is the first place to tune.
- **L3 (soft shadows) and L4 + L7 (sun bounce, lamp bake) cost nothing
  measurable**: within 0.3 ms of new, inside the spread, on both flights.
- **Our bloom costs 1.5-2.2 ms** at p50 (21.6 / 21.8 against 20.0 / 19.6).
  It stays off by default on a Mac; Settings turns it on.
- **The storm costs no more than the clouds** with the new model: the wet
  film and the lit rain fog are inside the spread on these flights, which see
  wet ground only from far off.
- **On foot the new model's tail is 2-3 ms longer**: p95 / p99 13.5 / 16.2
  today against 16.2 / 18.0 new. The walk is CPU-bound enough that a busy
  machine moved it (see above).
- **About 6% of flight frames are spikes, and most are paired with a short
  frame** (a 30 ms frame then a 2 ms one, summing to about 2.2-2.6 frames):
  presentation pacing, the CPU waiting for the next drawable, not work. Today's
  model shows the same share; the new model makes each stall longer because its
  frame is longer on the GPU. The rest (10-30% of spikes) sit near terrain
  landings and the town bake.
- **Every scenario misses the owner's budgets** (p95 8.3 ms, p99 16.7 ms) with
  both models on this machine, as today's model already did.

## Limits

- One machine, one sitting. Separate sittings on this Mac differ by more than
  the in-sitting spread (today's walk: 9.0 ms in a run cut short earlier,
  10.1 here), which is why only interleaved variants are compared.
- On this Mac the "GPU clouds" column spans more than the cloud passes (it
  rises 4 ms with lighting changes that touch no cloud pass); read it as GPU
  time up to the composite. "GPU total" is not reported on Metal.
- The walk's stuck-handling tapped the dig button rather than holding it
  (owner: "gotta hold mouse down to break blocks"), so a stuck walker never
  dug out; it is fixed after this sitting, which makes later walks not
  comparable with this one.
- The CSVs and logs are local (`output/perf/2026-10-29-clean`).

## clouds

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 2/2 | 59.26 | 16.26 | 29.80 **over** | 42.53 **over** | 48.68 | 52.18 | - | 11.16 / 13.53 |
| new | 2/2 | 45.93 | 20.34 | 37.16 **over** | 55.36 **over** | 63.31 | 70.64 | - | 15.51 / 18.94 |
| no-L3 | 2/2 | 46.31 | 19.92 | 36.24 **over** | 54.47 **over** | 60.01 | 64.45 | - | 15.43 / 18.65 |
| no-L8 | 2/2 | 51.21 | 18.14 | 35.10 **over** | 47.87 **over** | 52.68 | 64.32 | - | 13.78 / 16.89 |
| no-L4-L7 | 2/2 | 46.49 | 19.77 | 36.24 **over** | 54.31 **over** | 59.97 | 65.48 | - | 15.42 / 18.59 |
| bloom-lite | 2/2 | 43.04 | 21.60 | 39.90 **over** | 57.07 **over** | 63.13 | 70.74 | - | 15.26 / 18.18 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 68.51 | 14.68 | 19.07 **over** | 29.30 **over** | 32.58 | - / - | 11.39 / 12.39 |
| new | 54.03 | 18.58 | 26.67 **over** | 37.49 **over** | 41.64 | - / - | 15.19 / 16.83 |
| no-L3 | 54.58 | 18.54 | 28.62 **over** | 36.66 **over** | 41.84 | - / - | 15.11 / 16.65 |
| no-L8 | 59.36 | 17.00 | 21.08 **over** | 34.36 **over** | 37.12 | - / - | 13.66 / 15.08 |
| no-L4-L7 | 54.58 | 18.49 | 27.94 **over** | 38.16 **over** | 41.84 | - / - | 15.11 / 16.54 |
| bloom-lite | 49.77 | 20.29 | 33.77 **over** | 40.13 **over** | 43.48 | - / - | 14.93 / 16.27 |


## storm

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 2/2 | 59.66 | 15.48 | 30.56 **over** | 42.07 **over** | 46.05 | 54.49 | - | 11.15 / 13.40 |
| new | 2/2 | 46.53 | 19.66 | 39.11 **over** | 56.05 **over** | 61.82 | 67.57 | - | 15.41 / 18.63 |
| no-L3 | 2/2 | 46.38 | 19.82 | 37.24 **over** | 54.27 **over** | 59.80 | 71.42 | - | 15.44 / 18.64 |
| no-L8 | 2/2 | 51.00 | 18.34 | 34.35 **over** | 47.14 **over** | 52.52 | 65.58 | - | 13.76 / 16.93 |
| no-L4-L7 | 2/2 | 46.29 | 19.94 | 39.23 **over** | 54.18 **over** | 65.20 | 70.27 | - | 15.44 / 18.75 |
| bloom-lite | 2/2 | 42.39 | 21.79 | 40.33 **over** | 57.63 **over** | 65.88 | 68.16 | - | 15.32 / 18.43 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 68.91 | 14.70 | 21.89 **over** | 29.67 **over** | 31.39 | - / - | 11.37 / 12.36 |
| new | 54.66 | 18.48 | 28.71 **over** | 37.72 **over** | 41.80 | - / - | 15.07 / 16.53 |
| no-L3 | 54.62 | 18.51 | 30.48 **over** | 37.70 **over** | 41.98 | - / - | 15.09 / 16.58 |
| no-L8 | 59.11 | 17.05 | 27.39 **over** | 35.29 **over** | 42.28 | - / - | 13.66 / 15.14 |
| no-L4-L7 | 54.21 | 18.62 | 30.37 **over** | 37.47 **over** | 41.70 | - / - | 15.07 / 16.74 |
| bloom-lite | 48.80 | 20.57 | 32.97 **over** | 41.42 **over** | 48.31 | - / - | 14.98 / 16.43 |

## walk

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 2/2 | 93.54 | 10.43 | 14.55 **over** | 17.03 **over** | 24.37 | 31.45 | - | 3.80 / 5.77 |
| new | 2/2 | 78.67 | 12.21 | 17.73 **over** | 19.82 **over** | 25.35 | 33.24 | - | 5.25 / 7.84 |
