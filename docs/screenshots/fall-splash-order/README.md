# The fall splash drawn after the water (`water-flow` 6.1)

The owner, 2026-11-16: "water fall splash fx still not drawn in front of
water" (`docs/screenshots/handoff/fall-splash-behind-water.webp`).

- **Renderer:** the Mac (M4 Max, Metal), `--profile fast`, `--capture` at
  frame 180 (fixed step: looks, not frame time). The lens is off
  (`PBD_PASS_OFF=lens`).
- **Before:** `origin/claude/env-v2` at `b9635f4`. **After:** this branch.
- **Place:** a fall of 9 layers into a pool at `11.334 4.839` (jungle). The
  camera is held by `PBD_EYE_PATH` (one key: `0 lat lon altitude yaw pitch`,
  altitude over the sea's level), with `--fly --time 13 --rain 0`.

| Shot | Eye (`lat lon alt yaw pitch`) | What it shows |
| --- | --- | --- |
| `down` | `11.33374 4.79 26 180 -65` | Looking down onto the fall and the pools under it. Before, the spray and mist over the pools were painted over by the water (and leaked through it as a cyan glow); after, they stand on the surface. The mist against the bank on the left is the same pixel for pixel in both. |
| `side` | `11.33374 4.775 8.6 180 -6` | Eye level, 3.8 m over the pool. The mist in front of the fall and over the pool now shows. |
| `under-pool` | `11.33374 4.80 4.5 180 15` | From under the pool, 2.2 m down. The same as before (mean difference 0.7 of 255): the splash is murked by the compose's own terms. |
| `under-river` | `11.36 4.80 8.6 135 -6` | From under the river above the fall, 5 m down. Close to before; a mist puff that pokes into this river's water shows as a faint band where it is in front of the water's face. |

The finding of a fall foot was a temporary instrument (a log of the tier's
tallest `FallFeet`), not committed.

## The coordinator's review: is the pools' teal the water's? (2026-11-16)

Each `*-4way.jpg` is the same frame four ways: before (main), before with
the splash off, after with the splash off (`PBD_PASS_OFF=splash`), after.
"Before with the splash off" is a temporary build of `b9635f4` that skips
every foot (`PBD_NO_SPLASH`); it was not committed.

| Sheet | Before off vs after off (mean difference, of 255) | What it shows |
| --- | --- | --- |
| `down-4way` | 0.003 | With no splash, both builds draw the pools a dark slate with rain rings. The teal in "before" is the mist, drawn into the scene under the water, showing through it. |
| `side-4way` | 0.002 | The same from eye level: no splash, no teal, in both builds. |
| `under-pool-4way` | 0.000 | From under the pool. |
| `control-4way` | 0.004 | A shallow sandy reach upstream: teal in all four, with or without the splash. The deep pool's slate is the water's depth colour (shallow to deep), not something the change hides. |

So the change draws no water differently: the water pass, its blend and
its depth are the same as before; the splash pass draws after them and only
adds its quads. What changes is the mist: over a pool it is now the same
quads at the same opacity as over the bank (the bank's mist is the same
pixels in all four frames), where before it showed only as a teal glow
through the water.
