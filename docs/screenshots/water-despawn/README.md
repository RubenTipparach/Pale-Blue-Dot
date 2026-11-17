# Water that dries eases out too (`water-flow` 5.4)

The owner, 2026-11-17: "water spawning does animate correctly ... The despawn
is still step based." The write-up is the `water-flow` design, "Water that
dries eases out too".

- **Renderer:** the Mac (M4 Max, Metal), on its GPU. `--profile fast`, a
  capture (the simulation stepped at a fixed 1/60 s: looks, not frame time),
  1920 x 1080 (`PBD_WINDOW=960x540` on the Retina display), the lens off
  (`PBD_PASS_OFF=lens`), `--time 13 --rain 0`.
- **The rig:** the capture rig digs a pit and pours one source 5.6 m short of
  it (`--pour 1 --pour-ahead 12 --pit 5` and `--pit 2`); the rule ticks live
  once a second. `PBD_POUR_STOP_S` (a measurement instrument) takes the
  source through the ordinary edit path after 12 s (pit) or 10 s (channel).
  `PBD_CAPTURE_RUN=<dir>,150,1700` saves every frame; the camera is held by
  `PBD_EYE_PATH` (`0` and `150` at the spawn pose `24.117 -2.964 65.6 90
  -25`, then `151` at `24.1167 -2.9703 70.0 90 -50`). Before is `origin/main`
  at `db8f17e8` with only the instrument added; after is this branch.
- **The measure:** the mean frame-to-frame change over the frame at 480 x 270,
  the HUD and the eye label masked. A step is a frame whose change is over
  1.6 times both its neighbours' and over 2.5.

| Run | Steps before | Steps after | Worst before | Worst after |
| --- | ---: | ---: | ---: | ---: |
| Pour into a 5-deep pit, then drained | 16 | 5 | 10.0 | 5.5 |
| Pour into a 2-deep dug channel, then emptied | 16 | 5 | 8.4 | 4.0 |

`pour-pit-sheet.jpg` and `dug-channel-sheet.jpg`: the change over the whole
run, before (orange) and after (blue), and the frames across the worst
"before" step, before over after. Before, the water stands still through each
tick and jumps at the next; after, each tick's movement is spread over the
tick (the arcs).

## What is left (stepped through frame by frame)

The mesh's own continuity was counted in the game at every remesh of both
runs: every triangle drawn the frame before a remesh is drawn the frame
after, in position, flow, normal and sky (one triangle in one lane missed,
once). What still steps is not the water's mesh:

1. **The ground's wet look** (pit 1249, 1609; channel 484, 1024, 1084): the
   terrain shades wet and submerged ground from the column records, which
   are repacked at each tick, so sand under a front turns wet in a frame and
   ground under a drained cell dries in one. Not eased here: it is the
   terrain's shading, not the water's mesh.
2. **The mist at a draining fall's foot** (pit 1249, 1309): the puff cloud at
   the pit's foot still jumps at those ticks. The splash's drop is eased and
   a leaving foot fades out, but this one was not traced to its cause.
3. **The level ease pauses at each tick** (not a step): levels are
   smoothstepped (M31's S1, approved), so a pool draining over several ticks
   slows to a stop at each tick and starts again: the dips to near zero
   between the blue arcs.
4. **Small steps during the pour** (pit 1009, 1609, 1669; channel 424,
   1504; change 2.6 to 3.4): the same wet-ground switch as item 1.
