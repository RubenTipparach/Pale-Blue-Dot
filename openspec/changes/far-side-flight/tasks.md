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
- [x] Planned at lift-off from the weather and the terrain generator: 36
      headings scored on the longest stretch of solid cloud (cover >= 0.55) in
      the first 60% of the path (weighted to win), then relief, coast and biome
      variety less a penalty for open sea; daylight all the way (sun >= 0.2);
      5 km reach (9 km was a quarter of this world and flew into the night);
      landing on the flattest dry ground in the last third. Logged.
- [x] A cloud leg at the cloud's own middle height, with lead-in and out; then
      terrain-following at 90 m (highest ground 100 m behind to 600 m ahead,
      smoothed), with the ground's slope fed forward; a 30 m vertical lift-off
      and 45 degree climb.
- [ ] A camera that turns to each feature as it passes: not built; the route
      rig's pitch-from-dip camera is used.
- [x] `--verify-route scenic` (no weather headless): lands 22.5 m from the site
      in 43 s, never under 61 m airborne.
- [ ] Record it at full size. 2026-09-25, second machine (RTX 3060 Laptop,
      1440x900 window): planned 1.8 km of cloud from 1,050 m, touchdown 25.2 m
      from the site 48.9 s after the start; recorded at 720x450, 31 of 50
      seconds at 59+ (the ground before lift-off and seconds 27-31 around the
      cloud exit). Uploaded as
      `dungeon-crawler-2026/pale-blue-dot-scenic-cloud-flythrough-450p60.mp4`.
      The owner wants it at the window's full size, re-recorded once the haze
      work is on `main`.

## 4. Record
- [x] The cadence check in the `obs-record` skill (a repeat differs by under
      0.03 of a level after encoding, real motion by more than 0.1).
- [x] Recorded at 60 fps and 720x450: cadence median 60/60, 91 of 98 seconds at
      59+ (the rest are standing still before lift-off). Uploaded as
      `dungeon-crawler-2026/pale-blue-dot-far-side-flight.mp4`.
- [x] Real-time pacing on the route: 11-19 frames over 16.7 ms came from the
      atmosphere's step and a fine-set rebuild running at once (2 with the air
      frozen); they now never overlap: 3 per flight, p99.9 11.4 ms.
- [x] `obs-record` fixes found recording on a second machine:
  - [x] A relative `--launch` program path failed (`WinError 2`): the child
        runs in the program's own folder, where the relative path no longer
        points at it. Resolve the program against the caller's folder (or
        `PATH`) before launching, and before OBS is started, so a bad path
        costs nothing.
  - [x] That failure then reported "OBS NOT fully restored: temporary input"
        for an input that was never created. Remove only what was created.
  - [x] The contact sheet drew 3 of its 6 tiles: each tile selected a window
        half a frame wide by time, which misses the frame about half the time.
        Select each tile by frame number instead.
  - [x] The owner wants recordings at the window's full size: `--scale`
        defaults to 1.0, and the skill's recipes say so.
  - [x] The recipes say to pause any heavy build on the machine (another
        session's cargo build) before recording, because it skews the cadence.
  - [x] Record Pale Blue Dot with `--no-vsync`. Measured on the RTX 3060
        Laptop, cloud-hop on the merged build: with vsync the game averages
        60 frames a second but 23% of its frames run over 17.5 ms (p95 30 ms,
        GPU 5.5 ms), and the video carried a median of 56 new frames a second
        (12 of 43 seconds at 59+); without vsync, 59 (28 of 43). The uneven
        vsync pacing with a 5.5 ms GPU is its own finding for frame pacing.
