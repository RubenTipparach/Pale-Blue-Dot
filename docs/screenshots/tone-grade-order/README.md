# tone-grade-order: the trails behind sky objects, before and after

Turning the view smeared everything seen against the sky into fading copies:
stars into dotted arcs, trees and clouds into stepped trails. Cause and bisect:
`openspec/changes/tone-grade-order/design.md`.

All four are fixed-step captures on the Mac (Apple M4 Max, Metal, window
1440 x 900 at 2x, `--profile fast`), frame 240 after `READY`, from a new world
each time, turning 45 degrees a second (0.75 degrees a frame).

- **before**: `claude/perf-tuning` with only the fix taken out (the tone
  grade's node re-added per camera, as at `b76c3a9`).
- **after**: `ToneGrade::sub_graph` = `Core3d`.

Both builds carry the `position-readout` line, so the HUD's second line says
where each was taken.

| Shot | Where | Time and weather | Launch |
| --- | --- | --- | --- |
| `before-night.jpg`, `after-night.jpg`, `before-after-night.jpg` | `--at` 20.000 40.000 (map 20.000S 40.000E), on foot: eye 48.6 m above sea level, 1.6 m above the ground | 02:00 on the planet clock, sun 30.5 degrees below the horizon; `Weather here: cover 0.00, rain 0.00` | `pbd-app --world <new> --walk --at 20 40 --time 2 --pitch 40 --turn 45 --frames 240 --capture <png>` |
| `before-dawn.jpg`, `after-dawn.jpg`, `before-after-dawn.jpg` | the same spot | 05:00 on the planet clock, sun 5.0 degrees above the horizon; `Weather here: cover 0.00, rain 0.00` (cloud on the horizon, none overhead) | `pbd-app --world <new> --walk --at 20 40 --time 5 --pitch 15 --turn 45 --frames 240 --capture <png>` |

The after frames are also graded differently: with the node out of order the
grade could run before the main pass, so the frame on screen could miss its
exposure and display transform; after the fix it gets them every frame.
