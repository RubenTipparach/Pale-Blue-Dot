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
- [x] `--route far-side` with its validated settings (`FlightViewConfig`
      route_*). One height line, min(climb, cruise, descent) with rounded
      corners, instead of timed phases. Lands on the dry land nearest the
      antipode (169 degrees round), along the great circle through it.
- [x] `--verify-route far-side`: completed in 98.9 s, touchdown 5.7 m from the
      site, peak 3,096 m, camera at most 0.13 rad/s, one guard event at
      touchdown. It asserts each of these.
- [x] Cruise height 3,000 m: the planet's disc fills the 75 degree view.
      Camera pitch = the horizon's dip + 0.20 rad, which keeps the limb in the
      upper third (a level camera filmed black space through the climb).

- [x] The camera rig: pitch from the dip, a slow sway with bank, the landing
      site framed on the approach (aimed just past it), eased first-order with
      a rate cap; its settings as data with a reduced-motion switch; its peak
      rate reported by `--verify-route`.

## 3b. The scenic route
- [ ] Waypoints scored from the generator (relief, coast, biome rarity), a
      greedy spaced pick, the scores logged.
- [ ] A cloud leg at mid-slab where the weather map has cover; then
      terrain-following at about 90 m with a smoothed look-ahead.
- [ ] A camera that turns to each feature as the ship passes it.
- [ ] `--verify-route scenic`; record it and upload it like the far-side route.

## 4. Record
- [x] The cadence check in the `obs-record` skill (a repeat differs by under
      0.03 of a level after encoding, real motion by more than 0.1).
- [x] Recorded at 60 fps and 720x450: cadence median 60/60, 91 of 98 seconds at
      59+ (the rest are standing still before lift-off). Uploaded as
      `dungeon-crawler-2026/pale-blue-dot-far-side-flight.mp4`.
- [x] Real-time pacing on the route: 11-19 frames over 16.7 ms came from the
      atmosphere's step and a fine-set rebuild running at once (2 with the air
      frozen); they now never overlap: 3 per flight, p99.9 11.4 ms.
