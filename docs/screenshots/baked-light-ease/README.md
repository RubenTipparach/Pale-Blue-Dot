# baked-light-ease: baked light eases, it never steps

Fixed-step captures on the Mac (Apple M4 Max, Metal, `--profile fast`), each
in a world made fresh for the run and deleted after it. Every frame of a
stretch was saved (`PBD_CAPTURE_RUN`) in a 480 x 300 point window
(`PBD_WINDOW=480x300`, 960 x 600 pixels), and a fixed patch of the picture
was averaged frame by frame: the mean Rec. 709 luma (0..255) of the sRGB
pixels, by `tools/burst_luma.py <dir> <out> name=x0,y0,x1,y1 ...` (patches in
fractions of the frame). The frame numbers are the frame log's, which the
`BAKED_LIGHT` log lines carry.

Before = `ea27c90` (the instrument commit: `claude/perf-tuning` at `a8376bd`
plus the instruments, which change nothing drawn), run against its own
shaders. After = this change.

| Shot | Place (`PLACE ready`) | `--time` | Sun (`sun from the spawn at --time`) | `Weather here` |
| --- | --- | ---: | --- | --- |
| night landing | -1.860 -1.140 (map 1.860N 1.140W), near Tahal, 16.6 m elevation, 1.6 m above ground; the frames shown are about 60 m west, at -1.860 -1.723 (map 1.860N 1.723W), 16.2 m elevation, 3.2 m above ground (the walker mid-hop), by the HUD | 22.5 | `--yaw -27.7 --pitch -35.3` (35 deg under) | cover 0.00, rain 0.00 |
| day landing | the same spawn; frames at -1.860 -1.721 (map 1.860N 1.721W), 16.3 m elevation, 3.3 m above ground | 18.5 | `--yaw -25.7 --pitch 19.3` | cover 0.00, rain 0.00 |
| dusk | 29.697 1.193 (map 29.697S 1.193E), Holbrook, 75.6 m elevation, 1.6 m above ground, standing still; lamps lit at hour 20.51 | 20.45, the clock running (`PBD_CLOCK_RUNS=1`) | `--yaw -25.0 --pitch 3.7` | cover 0.00, rain 0.00 |

Commands (`<new>` a fresh world name; `before` with the old exe's burst
counted in frame-log frames, `after` in frames from the first open frame):

- Night landing:
  - before: `PBD_WINDOW=480x300 PBD_CAPTURE_RUN=<dir>,330,430 pbd-app --world <new> --walk --at -1.86 -1.14 --time 22.5 --yaw 0 --pitch -12 --lamps 60 --walk-distance 300 --frames 780 --capture <png>`
  - after: the same with `PBD_CAPTURE_RUN=<dir>,150,330` and `--frames 620`.
- Day landing: as the night one with `--time 18.5` and no `--lamps`
  (after: `PBD_CAPTURE_RUN=<dir>,150,420`).
- Dusk:
  - before: `PBD_WINDOW=480x300 PBD_CLOCK_RUNS=1 PBD_CAPTURE_RUN=<dir>,420,330 pbd-app --world <new> --walk --at 29.69673 1.19287 --time 20.45 --yaw 90 --pitch -15 --lamps 6 --frames 680 --capture <png>`
  - after: the same with `PBD_CAPTURE_RUN=<dir>,380,260` and `--frames 700`.

## The files

- `brightness-over-frames.png`: each swap's patch, before (orange) and after
  (blue), from 8 frames before it to 60 after, aligned on the swap.
- `night-landing-frames.jpg`: the frame before and the frame of a landing.
  The lit ground's lower half: **+24.9 luma before** (the lamp flood stood in
  for the bake for two frames, then faded back over 0.5 s), **+0.5 after**
  (the walk's own drift; the set brought its lamp bake).
- `day-landing-frames.jpg`: the same by day. The sand terrace walls in
  shade: **-11.0 luma before** (the bounce at zero for two frames), **-2.4
  after** (the walk's own drift, -1.9 to -2.1 a frame on either side).
- `dusk-town-frames.jpg`: two of the town's meshes swapping in their dusk
  bakes, 8 to 9 frames after the relight. Sandstone wall **+5.9** and town
  wall **+5.3 luma in one frame before**; at most **+0.2** and **+0.3** a
  frame after. In the alley facing west (`--yaw` left out, the first dusk
  run), a stone tower stepped **+12.4** before.
- `dusk-town-settled.jpg`: 176 frames after the relight, before and after
  are the same picture (mean difference 0.0001 levels): only the way there
  changed.

The grass beside the lantern post (the lamp bake's own cross-fade, already
eased before) rises at most 1.2 luma a frame after, against 2.2 before (1 s
smoothstep against 0.5 s linear). The bounce's 2-degree re-bakes, measured
at a sand pit at 18:30 with the clock at 4x (`--at -1.86 -1.14 --yaw 0
--pitch -12`), showed nothing to fix and were left (design 3).

Frame cost was not measured here (`perf_suite.py` is the owner's): the bakes
a landing now brings are made on its pool task (4-10 ms there), and the GPU
reads the light field's second half only during a dusk blend.
