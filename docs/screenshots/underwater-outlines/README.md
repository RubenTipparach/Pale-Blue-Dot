# underwater-outlines: bright rims round everything seen under water (`ocean-environments` B24)

The owner, 2026-11-19: "there seems to be outlining issues when under water",
with a close-up of the held fishing rod edged in a jagged pale line.

Fixed-step captures in a Claude Code cloud container: lavapipe (software
Vulkan, no GPU) under `xvfb-run`, 1440 x 900, `--profile fast`. Looks only:
frame time was NOT measured.

Shot: `--walk --at -4.125 -11.125 --tool rod --pitch -10 --time 11 --frames 360`
(feet about 9 m down, the rod held). "Before" is the same build with
`water.wgsl` at `bcc3254f` (before the fix) swapped in for the run.

| File | What it shows |
| --- | --- |
| `deep-rod-before.jpg`, `deep-rod-after.jpg` | The whole frame, before and after. |
| `rod-edge-before-after.png` | The rod's top right, 2x nearest: the cyan, green and blue rim before; a plain edge after. |
| `rod-edge-zoom-before-steps.png` | The rod's edge at 8x: before, the third step, and the final fix. |

Bright blue pixels within the rod's area (blue over green by 25, blue over
120), counted on the same crop:

| Step | Bright blue | Dark-red specks |
| --- | ---: | ---: |
| Before | 116 | 0 |
| 1. Wobble samples only its own surface | 78 | 0 |
| 2. Edge fogged by the samples' mean transmittance | 60 | 0 |
| 3. Edge split into near and far, each fogged by its own distance | 33 | 28 |
| 4. Step 3 with the unmix guarded | 33 | 20 |

The 33 left are mostly the edge's own blend of rod and water, which is the
antialiased edge, now at the water's colour and not the unfogged far one. A
handful of dark specks remain where the unmix still overshoots. Not shot:
the owner's darker scene (at dusk or night), and the hex steps against far
water at night.
