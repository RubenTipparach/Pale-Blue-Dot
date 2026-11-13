# two-set-dissolve

Before-and-after evidence for `openspec/changes/two-set-dissolve`, 2026-11-13.
Fixed-step captures (`--capture`: 60 frames to a second of game time) of the
`fast` build (`cargo build --profile fast -p pbd-app`) on the M4 Max Mac, a
1440 x 900 window drawn at 2880 x 1800, a fresh world each run (deleted after).
The frames come from the landing-frames instrument (`PBD_LANDING_SHOTS`,
design section 5). They show what a landing looks like, not how smoothly it
runs: frame cost was not measured (no perf suite in this session).

Before is `claude/perf-tuning` at `40040db` with the instrument (`f992899`);
after is the change with the sea sheet's commit on top (`e89aa22`,
`ac98c01`). Both ran

```text
PBD_LANDING_SHOTS=3,4,5 target/fast/pbd-app --world <fresh> --route far-side \
  --time 10 --frames 7000 --capture <dir>/end.png
```

## Where

The far-side route from the spawn: `PLACE spawn at 28.646 0.000 (map 28.646S
0.000E) | 96.1 m elevation | 2.1 m above ground`; `sun from the spawn at
--time: --yaw -171.3 --pitch 37.9`; `Weather here: cover 1.00, rain 1.00` at
the spawn, `cover 0.72, rain 0.14` on the climb just before the landing. The
shots are of **fine set 4's landing, on frame 1,777**: the climb past 2,522 m
above the ground, where the set is replaced by one with no fine levels, so
every level-8 cell the old partition drew goes back to the base. The HUD at
frame 1,776 reads `at 44.543 -6.729 (map 44.5435 6.729W) | 2569.8 m elevation
| 2521.8 m above ground`, 1,223 m from the old set's anchor. (The run's own
`PLACE capture` line is for `end.png`, after touchdown on the far side: `at
-18.144 -176.690 (map 18.144N 176.690W)`.)

The crop is x 1300-1940, y 1040-1440 of the 2880 x 1800 frame, at full
resolution, so the dither is pixel for pixel: an island below and the sea
round it.

| file | what it shows |
| --- | --- |
| `far-side-climb-context.png` | The whole frame before the landing (frame 1,776, before the change), a quarter size, the crop outlined. |
| `far-side-climb-before-pair.png` | Before: the crop at frame 1,776, at 1,777, and white where they differ. The island's level-8 hexes are the base's larger ones in the next frame: the whole island switches at once (`LOD_FADE partial: level 8: the old band reaches 3547 m from the new anchor, the new records 0 m`). 13.1% of the crop changes on that frame, 9.0 to 9.3% on each frame after it. |
| `far-side-climb-before-steps.png` | Before: the crop 0, 18, 36 and 66 frames after the landing. Already the base from frame 0. |
| `far-side-climb-after-pair.png` | After: the same two frames. The landing frame is the frame before it, moved on: the difference is the motion and the dither's own shimmer, 9.4% of the crop, as on every frame round it (9.4 to 9.6%). |
| `far-side-climb-after-steps.png` | After: the crop 0, 18, 36 and 66 frames after the landing. The island and the sea round it dissolve from level 8 to the base over the second (60 frames): at 18 and 36 frames both are drawn, each in its share of the pixels; by 66 it is the base. |

## Landings

From the logs of the same build, `--route far-side` as above, `--route scenic
--time 10 --frames 4000`, and `--walk-distance 1000 --time 10 --frames 40000`:

| Run | Before: landings, `LOD_FADE partial` | After: landings, dissolved, `partial`, held |
| --- | --- | --- |
| far-side flight | 20, 2 (and the refill through 2.4 km landed undissolved) | 19, 19, 0, 10 (0.15 to 0.72 s) |
| scenic flight | 33, 6 (two runs alike) | 31, 31, 0, 8 (0.08 to 0.38 s) |
| 1 km walk | 38, 1 (the first set after the spawn) | 29, 29, 0, 0 |

Not shown: the voxel column tier, 90 m round the anchor, still switches at a
landing (the change's task 5.1).
