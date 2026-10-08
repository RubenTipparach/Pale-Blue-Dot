# Town frame time, 2026-10-08

The `town-frame-time` change against `main`, in one sitting, interleaved
(main, after, after, main) on the owner's machine. The sitting was stopped
after the town runs because the owner needed the computer. `clouds`, `storm`
and `walk` were not measured.

## Findings

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

## Limits

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

## town

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| main | 1/2 | 57.47 | 15.94 | 18.37 **over** | 21.22 **over** | 84.93 | 6911.23 | 6.66 | 0.19 / 0.51 |
| after | 2/2 | 91.56 | 10.59 | 16.39 **over** | 18.46 **over** | 56.92 | 6771.51 | 6.34 | 0.20 / 0.57 |
- invalid: main run 1: no WALK_DONE line


