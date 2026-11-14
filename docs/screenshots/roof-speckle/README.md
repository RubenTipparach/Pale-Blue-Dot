# roof-speckle: before and after

`openspec/changes/roof-speckle`. The owner, 2026-11-14: "under roof areas has
zfighting still", and "stair way too".

## How they were taken

- An M4 Max Mac, Metal, on the GPU, 2880 x 1800, the fast build
  (`cargo build --profile fast -p pbd-app`) of `fix/roof-speckle`.
- Fixed-step captures, `--capture <png> --frames 150`, each in a new world
  (`--world rs-<n>`), `--rain 0 --weather-at 7200 --open-doors`. A capture
  runs at 100 % render scale; the player's default is 85 %, which showed the
  same.
- Before and after are the same binary: the fix is in
  `assets/shaders/field_lit.wgsl`, read at launch, so "before" ran with the
  shader as it was at `3b44fb5` and "after" with `BAND_SLACK`.
- The house is Holbrook's Fieldstone house, the one with the newel stair.
  The walker's `--up 3.2` stood it on the ground floor in these runs, so the
  upper-floor shots use the free camera, `--view column --height 4.6`
  (the upper floor is 3 m up, plus eye height).
- Full frames are JPEG (quality 90); the two crops are PNG at full
  resolution.

## The shots

| Files (`before-` / `after-`) | Flags | What it shows |
| --- | --- | --- |
| `upper-floor-newel-ceiling-2200.jpg` | `--view column --at 29.63663 0.80156 --height 4.6 --yaw -0.4 --pitch 30 --time 22` | From the upper floor, through the doorway, the ceiling under the roof over the newel. Before: dark, covered in fine orange speckle in curved fringes. After: lit by the newel's sconce, clean. |
| `upper-floor-newel-ceiling-1830.jpg` | the same with `--time 18.5` | The same at dusk: the speckle shows on the sky-lit (greenish) ceiling. |
| `upper-floor-down-the-newel-2200.jpg` | the same with `--pitch -35 --time 22` | Down the newel from the upper floor. Before: a tread at 3.0 m speckled. After: clean. |
| `ground-floor-newel-2200.jpg` | `--walk --at 29.63663 0.80156 --yaw -0.4 --pitch 20 --time 22` | The slice 2b newel spot from the room beside it: the line where a sconce's band ends on the wall moves 5 cm; nothing else changes. |
| `crop-ceiling-before-top-after-bottom.png` | crops of the first two rows, 22:00 left, 18:30 right | The ceiling, before above and after below. |
| `crop-stair-before-left-after-right.png` | crops of the last two rows | The tread (top) and the wall band (bottom), before left and after right. |

## Not shown

- The owner's own frames were not available here; these are the same faces
  the instrument names (design 2), reproduced at the newel house.
- No video: screenshots stand in until the owner's batch (CLAUDE.md).
- The frame cost was not measured (two subtractions per room light per
  pixel).
