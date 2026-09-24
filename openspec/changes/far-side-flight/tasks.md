# Tasks

## 1. Measure
- [x] Tour frame times (p50 2.66, p95 6.23, p99 37.56 ms) and the recording's
      cadence (20-45 new frames of 60; freezes of 150-300 ms).
- [x] Fine-set rebuilds during the recorded flight: 35 in 47 s, every ~1.3 s.
- [ ] `--frame-log`: attribute each over-budget frame to its cause (generation
      on every core, a landing, or something else).

## 2. Frame pacing
- [ ] Gate fine-set rebuilds on clearance, from the projection, in every flight
      mode (manual flight included).
- [ ] Leave a core free for the main thread, only if the frame log shows it helps.
- [ ] Split a landing across frames, only if the frame log shows a landing is the hitch.
- [ ] The tour's p99.9 under 16.7 ms, measured.

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
