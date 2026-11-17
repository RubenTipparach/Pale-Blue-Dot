# Kelp: before and after (`ocean-environments` B20), 2026-11-16

Machine: Apple M4 Max (Metal), the release build, windowed, no vsync. View:
`--walk --at 71.308 -8.196`, the walker on the bed 9 m down in the B16 kelp
forest, held (the stroll waits for a town that never comes near). Frames from
the game's READY plus 10 s to 70 s after launch. Three variants interleaved
B A N N A B B A N, two sittings (`held_view.py`, `results.json`):

- **before**: `claude/env-v2` at `af754356` with the instruments only;
- **after**: this branch;
- **before-nokelp**: before with `PBD_PASS_OFF=kelp` (no kelp stand laid).

## Findings

What does not depend on the machine being quiet, read off the game's own
`SEA_PLACES` line in every run (identical in every repeat):

| | before | after | no kelp |
| --- | ---: | ---: | ---: |
| kelp triangles drawn round the eye | **625,011** | **8,974** | 0 |
| all the sea's places' triangles | 686,211 | 38,734 | 61,200 |
| vertices rewritten for the sway, a frame | **1,875,033** | **26,922** | 0 |
| bytes re-uploaded for them, a frame (48 B a vertex) | about 90 MB | about 1.3 MB | 0 |
| main thread in the sea places' system, median ms a frame (second sitting) | **6.9 to 7.2** | **0.15 to 0.18** | 0.03 |
| triangles a stand in 12 m of water | 575 | 14 | |
| stands a square metre | 0.175 | 0.083 | |

The approved mockup (M25, `docs/mockups/ocean.html`, `sceneKelp`): 14
triangles a stand in 12 m of water (two crossed frond cards a 4 m, one flat
on the surface), 0.083 stands a square metre. The game now matches it.

The system time leaves out Bevy's render-world copy and upload of the
rewritten meshes, the larger share of the before's cost (90 MB a frame).

## Frame time: not a result

Every run of both sittings had other work on the same Mac beside it: up to
three other sessions' game captures and `cargo` builds and tests (listed in
`results.json` under `contaminated`). The perf rules make these numbers
invalid, and they read so: one variant's p50 ranges from 19 to 104 ms across
repeats. They are kept for the record and not claimed as a speed-up.

| variant | runs | p50 ms (each run) | p95 ms (each run) |
| --- | ---: | --- | --- |
| before | 6 | 49.9, 54.4, 38.5, 62.0, 54.8, 46.4 | 87.9, 108.2, 91.7, 110.6, 131.7, 97.7 |
| after | 6 | 35.4, 46.6, 33.0, 64.7, 104.4, 18.7 | 87.3, 101.4, 72.2, 100.5, 145.5, 76.2 |
| before-nokelp | 6 | 37.2, 30.5, 23.5, 25.9, 93.1, 19.6 | 91.2, 84.0, 72.3, 110.6, 134.8, 68.2 |

## Limits

- The frame-time sitting has to be repeated on a quiet machine:
  `python3 docs/benchmarks/2026-11-16-kelp/held_view.py . output/perf/kelp`
  with `pbd-app-before` and `pbd-app-after` in `output/perf/kelp-exe/`.
- GPU time per pass is not read on Metal (no timestamps inside a pass).
- No `LATE_PIPELINE` in any run of either exe.
