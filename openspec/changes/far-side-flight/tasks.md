# Tasks

## 1. Measure
- [x] Tour frame times (p50 2.66, p95 6.23, p99 37.56 ms) and the recording's
      cadence (20-45 new frames of 60; freezes of 150-300 ms).
- [x] Fine-set rebuilds during the recorded flight: 35 in 47 s, every ~1.3 s.
- [x] `--frame-log`: attribute each over-budget frame to its cause. Baseline
      tour, 12,000 frames: 306 over 16.7 ms, all while a rebuild ran, none on a
      landing; p99 42.7, p99.9 79.2, max 147 ms.

## 2. Frame pacing
- [x] Bands by slant distance from the player's height (`live_bands_m`): a set
      lays only the live bands, and the partition table it publishes is built
      from them, so the GPU and the builder read one table. Tests: level 11 is
      empty from 300 m up, every band from 2.4 km, the ground unchanged.
      Alone: p99 15.7, p99.9 33.4 ms, 109 over budget.
- [x] The rebuild rule follows the live bands in every flight mode: a band
      turning live, a live band outrunning its laid margin (grown with height,
      `regen_m`), or high enough that none is live. Test pins the ground rule.
- [x] Background rebuilds on a quarter of the cores (forced ones keep all).
      The frame log showed it is THE fix: at 8/4/2 threads, 195/6/3 frames over
      budget, p99.9 47.5/8.8/5.6 ms.
- [x] Split a landing across frames: not needed. No over-budget frame fell on
      a landing.
- [x] The tour's p99.9 under 16.7 ms: 5.8-6.6 ms over three runs, 2-4 frames
      of 11,900 over budget (max 30-63 ms, isolated). Requirement 'Level is
      quantised from distance to the player' moved into the main spec.

## 3. The route
- [ ] `--route far-side` with its validated settings, and the phases per the design.
- [ ] `--verify-route far-side`: completes, lands at the destination, keeps its
      clearance, and has continuous acceleration. A test pins it.
- [ ] Choose the cruise height from a capture, and write it down.

- [ ] The camera rig per phase (the design's table): its settings as data, the
      reduced-motion setting, and its peak angular speed and acceleration in
      `--verify-route`.

## 4. Record
- [ ] The cadence check in the `obs-record` skill.
- [ ] Record the route at 60 fps and half resolution, check 60/60, and upload to
      Drive (`dungeon-crawler-2026`).
