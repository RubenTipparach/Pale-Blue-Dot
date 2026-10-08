# Town frame time, 2026-10-08

Two rounds on the town walk out of Ashenstead, each an interleaved A/B in one
sitting on the owner's machine (RTX 3070, Vulkan, 1440 x 900, no vsync, 10:00,
rain forced).

## Findings

- **Round 2 (this page's tables): 82 to 98 fps mean, p50 10.5 to 9.1 ms.**
  `before` is round 1's build, `after` adds decisions 5-7. Resting craft
  sleep: the craft stepped a tick fell from about 20 to under 2, about 2.9 ms
  a tick saved. A town's templates are no longer cloned every frame, and a
  town material is re-prepared about once a second, not every 45 ms. p99.9
  fell from 131 to 45 ms.
- **Round 1: main 63 fps at p50 to 94-98 fps.** Each town's lighting is no
  longer recomputed on every terrain landing (240-450 ms hitches), and the
  field corners stop recomputing a ground height that never changes (6-10 ms
  a frame). Its tables are in `report.md`'s round 1 section, checked in
  beside this page.
- **From main to now, at p50: 16.0 ms to about 9.1 ms.** GPU time is about
  6 ms throughout, so the frame is close to the GPU now.
- **Still open:** frames slower than 14 ms are 7% of the walk and are not
  periodic: they are the spread of an uncapped frame. A fresh world freezes
  for about 18 s at its start, in both builds, and is next to measure.

## Limits

- The owner was using the machine during round 2, so its spread is wider.
  Its two variants were interleaved, so both saw the same disturbance.
- Only the town walk. The flights, storm and open-country walk are not
  measured in either round.

# Performance suite, 2026-10-08 18:39

- revision: `b3c5428` (uncommitted changes)
- adapter: NVIDIA GeForce RTX 3070 (Vulkan); window 1440 x 900, uncapped (`--no-vsync`)
- clock pinned at 10.0 h; first 10.0 s of each run thrown away; 2 run(s) per variant, interleaved
- budgets: p95 within 8.3 ms (120 fps), p99 within 16.7 ms (60 fps); **over** marks a miss. Each cell is the median over the valid runs.

## town

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| before | 2/2 | 82.38 | 10.52 | 16.35 **over** | 18.79 **over** | 130.66 | 6792.44 | 7.21 | 0.51 / 0.91 |
| after | 2/2 | 98.01 | 9.18 | 15.20 **over** | 19.58 **over** | 44.58 | 6838.06 | 5.86 | 0.19 / 0.49 |



## Round 1 (main against decisions 1-4)



- **Walking through a town: 63 fps to 94-98 fps at the median.** Frame p50
  is 16.0 ms on main and 10.2-10.7 ms after. p95 is 18.4 against 16.1-16.5
  ms, and p99 21.1 against 17.7-18.4 ms (frames from 30 s on). The 60 fps
  floor holds at p95 after and does not hold on main.
- **The hitches near towns are gone.** On main every fine-set landing
  re-baked every standing town's light, 240-450 ms over two frames (the
  trial walks earlier in the day: 34 frames over 150 ms in 101 s). In this
  sitting main's valid run had 8 frames over 50 ms after 30 s, and the
  branch had 1 and 2.
- **GPU time did not move** (6.3-6.7 ms p50 in both builds). The win is all
  main-thread work: the generator's ground height, asked 4,616 times a frame
  by the field corners and up to 360,000 times a landing by the bake.
- **The frame is still CPU-bound.** About 10 ms of wall time against 6.3 ms
  of GPU. The next cost has not been found yet. The weather suspects in the
  design (two cube uploads and 8,192 atmosphere samples a second, and the
  held-then-burst stepping) are the next instruments.
- **A fresh world freezes for about 18 s at its start, in both builds**:
  frames of 8.7 s, 2.5 s and 6.8 s, 8-18 s after launch, while the world's
  sites are surveyed and its towns laid and saved. The last of the three
  falls after the suite's 10 s warm-up, so it is each row's `max`. The town
  change does not touch it. It is not yet known whether opening a saved world
  pays any of it.

### Limits

- One valid main run. The other stopped at 230 m (`no WALK_DONE`) when the
  sitting was interrupted, so main's row is that one run. The branch's two
  runs differ by 0.5 ms at p50, which is less than the difference measured.
- Only the town scenario. Whether the flights, storm and open-country walk
  moved is not measured. The change only skips work that reads to the same
  values, so they are not expected to be worse.
- RTX 3070, Vulkan, 1440 x 900, `--no-vsync`, clock 10:00, rain forced to 1,
  first 10 s thrown away.

# Performance suite, 2026-10-08 16:27

- revision: `7ba872e`
- adapter: NVIDIA GeForce RTX 3070 (Vulkan); window 1440 x 900, uncapped (`--no-vsync`)
- clock pinned at 10.0 h; first 10.0 s of each run thrown away; 2 run(s) per variant, interleaved
- budgets: p95 within 8.3 ms (120 fps), p99 within 16.7 ms (60 fps); **over** marks a miss. Each cell is the median over the valid runs.

### town

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| main | 1/2 | 57.47 | 15.94 | 18.37 **over** | 21.22 **over** | 84.93 | 6911.23 | 6.66 | 0.19 / 0.51 |
| after | 2/2 | 91.56 | 10.59 | 16.39 **over** | 18.46 **over** | 56.92 | 6771.51 | 6.34 | 0.20 / 0.57 |
- invalid: main run 1: no WALK_DONE line


