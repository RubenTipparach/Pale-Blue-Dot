# underwater-shafts: light in the water, not in the scene

Fixed-step `--capture` stills from a Claude Code cloud container with no GPU
(lavapipe, `--profile fast`). They show looks, not performance: the frame
cost was not measured. `before-*` is `water.wgsl` at `66780be5`, and `after-*`
is this change. Both were rendered by the same binary, because only the
shader differs.

| File | Where | Command |
| --- | --- | --- |
| `before-shallow-sun.jpg`, `after-shallow-sun.jpg` | seabed 3.4 m under the sheet, facing the sun | `pbd-app --walk --at 3.5 -18.15 --yaw -150 --pitch 10 --time 11 --frames 560 --capture <png>` |
| `before-10m-sun.jpg`, `after-10m-sun.jpg` | seabed 9.4 m under, a terrace on the right | `pbd-app --walk --at -4.125 -11.125 --yaw -150 --pitch 15 --time 11 --frames 750 --capture <png>` |

- Before: the shafts' march was spread over each pixel's distance to the
  scene. The light stops in a band where the far depth changes (the right
  shaft at 10 m) and does not reach the seabed.
- After: the shafts are sampled at the same sixteen distances along every
  ray, over their 40 m reach, and the scene only ends the march. They are
  columns that lean with the sun down to the seabed and across the terrace.
- The frames' mean brightness is within 2.5% of the before frames, measured
  in rows (design 2).
