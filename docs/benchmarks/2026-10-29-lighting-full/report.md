# lighting-model, every part: the full sitting (2026-10-29)

Apple M4 Max (Metal), release build at `b85718f`, window 1440 x 900, uncapped
(`--no-vsync`), clock pinned at 10:00, first 10 s of each run thrown away.
Three scenarios: `clouds` (`--route scenic`), `storm` (`--route scenic --rain 1`)
and `walk` (`--walk-distance 1000`). Each variant ran twice, interleaved (A..F
then F..A), with nothing else running and the screen unlocked. All 28 runs
were valid. This sitting replaces the stopped first pass in
`2026-10-29-lighting-all`.

## Findings

- **The new model costs about 4.2 ms at p50 in flight and 1.4 ms on foot.**
  Today 15.6 / 15.4 ms (clouds / storm) against new 19.8 / 19.7 ms; on the
  walk 10.4 against 11.8 ms. The spread between repeats is at most 0.55 ms
  (today's clouds; 0.2 ms or less everywhere else), so every difference
  below past that is a result. The owner has accepted the cost and will tune
  it ("Adding more frame time is not a problem").
- **L8 (reflections) is the one part that costs measurably: 1.6 ms** at p50
  on both flights (new 19.8 against no-L8 18.2), and most of the tail: p99
  54.5 against 47.5 ms. It is the first place to tune.
- **L3 (soft shadows) and L4 + L7 (sun bounce, lamp bake) cost nothing
  measurable here**: within 0.1 ms of new on both flights. The first pass's
  0.5 ms each did not repeat; it was a single run with no spread.
- **Our bloom costs 2.0 ms** at p50 (21.8 against 19.8 ms on both flights).
  It stays off by default on a Mac, as built; Settings turns it on.
- **The storm costs no more than the clouds** with the new model (19.7
  against 19.8 ms): the wet film and the lit rain fog are inside the spread.
- **On foot the tail is no worse.** Walk p95 / p99: today 18.7 / 31.9 and
  13.2 / 15.6 ms over its two runs, new 14.5 / 15.9 and 17.2 / 19.3 ms.
  Today's first run carried the sitting's start-up hitches; the medians
  across runs are within each other's spread.
- **Every scenario misses the owner's budgets** (p95 8.3 ms, p99 16.7 ms) with
  both models; today's model already did on this machine. This sitting
  measures the change, not the budget.

## Limits

- One machine, one sitting. The whole sitting ran slower than 2026-10-28's
  (today's model 15.6 ms here against 10.5 ms then); compare within this
  report only.
- On this Mac the column "GPU clouds" spans more than the cloud passes: it
  rises 4.2 ms with lighting changes that touch no cloud pass, and falls
  1.7 ms with L8 off. Read it as GPU time up to the composite, not as cloud
  cost. "GPU total" is not reported on Metal.
- Bevy's own bloom was not in this sitting (the first pass had it at 2.4 ms
  against our 1.6 ms).
- The frame graphs are in the HTML page; the CSVs and logs are local
  (`output/perf/2026-10-29-lighting`).

## clouds

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 2/2 | 58.90 | 15.88 | 30.11 **over** | 42.56 **over** | 48.53 | 54.27 | - | 11.24 / 13.61 |
| new | 2/2 | 46.53 | 19.78 | 36.44 **over** | 54.48 **over** | 60.98 | 69.55 | - | 15.41 / 18.52 |
| no-L3 | 2/2 | 46.54 | 19.74 | 37.67 **over** | 55.01 **over** | 64.93 | 70.00 | - | 15.40 / 18.53 |
| no-L8 | 2/2 | 51.06 | 18.19 | 34.62 **over** | 47.76 **over** | 52.21 | 64.30 | - | 13.74 / 16.78 |
| no-L4-L7 | 2/2 | 46.69 | 19.78 | 38.10 **over** | 54.55 **over** | 59.23 | 68.46 | - | 15.40 / 18.54 |
| bloom-lite | 2/2 | 42.44 | 21.79 | 41.12 **over** | 57.85 **over** | 64.89 | 70.82 | - | 15.28 / 18.35 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 68.09 | 14.77 | 21.18 **over** | 30.47 **over** | 32.81 | - / - | 11.42 / 12.68 |
| new | 54.62 | 18.50 | 27.65 **over** | 38.01 **over** | 40.84 | - / - | 15.08 / 16.55 |
| no-L3 | 54.71 | 18.36 | 30.79 **over** | 38.23 **over** | 41.61 | - / - | 15.06 / 16.57 |
| no-L8 | 59.49 | 16.99 | 25.56 **over** | 34.47 **over** | 38.16 | - / - | 13.58 / 14.97 |
| no-L4-L7 | 54.65 | 18.44 | 30.72 **over** | 38.14 **over** | 41.41 | - / - | 15.11 / 16.54 |
| bloom-lite | 48.91 | 20.57 | 32.24 **over** | 40.63 **over** | 53.50 | - / - | 14.95 / 16.26 |


## storm

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 2/2 | 59.72 | 15.50 | 30.12 **over** | 41.55 **over** | 45.79 | 50.17 | - | 11.12 / 13.31 |
| new | 2/2 | 46.36 | 19.79 | 38.38 **over** | 54.49 **over** | 61.56 | 70.25 | - | 15.46 / 18.70 |
| no-L3 | 2/2 | 46.49 | 19.70 | 37.54 **over** | 54.86 **over** | 63.44 | 68.10 | - | 15.34 / 18.55 |
| no-L8 | 2/2 | 51.22 | 18.17 | 34.56 **over** | 47.27 **over** | 54.89 | 63.32 | - | 13.73 / 16.77 |
| no-L4-L7 | 2/2 | 46.46 | 19.83 | 36.81 **over** | 54.57 **over** | 64.55 | 68.75 | - | 15.39 / 18.75 |
| bloom-lite | 2/2 | 42.37 | 21.78 | 41.33 **over** | 57.18 **over** | 65.22 | 71.09 | - | 15.30 / 18.48 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 69.05 | 14.61 | 18.34 **over** | 29.07 **over** | 31.87 | - / - | 11.35 / 12.32 |
| new | 54.65 | 18.50 | 29.62 **over** | 38.42 **over** | 42.62 | - / - | 15.11 / 16.61 |
| no-L3 | 54.84 | 18.31 | 29.35 **over** | 38.27 **over** | 41.42 | - / - | 15.01 / 16.53 |
| no-L8 | 59.57 | 16.89 | 21.46 **over** | 34.56 **over** | 38.45 | - / - | 13.60 / 15.05 |
| no-L4-L7 | 54.63 | 18.46 | 28.67 **over** | 37.89 **over** | 50.64 | - / - | 15.07 / 16.61 |
| bloom-lite | 49.16 | 20.48 | 34.16 **over** | 41.39 **over** | 44.50 | - / - | 14.93 / 16.26 |

## walk

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 2/2 | 84.83 | 10.41 | 18.71 **over** | 31.91 **over** | 37.26 | 47.20 | - | 3.59 / 6.28 |
| new | 2/2 | 81.35 | 11.92 | 17.20 **over** | 19.32 **over** | 30.63 | 37.21 | - | 5.30 / 9.55 |
