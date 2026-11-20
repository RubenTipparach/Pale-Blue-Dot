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
| 5. The composite's sample mask (no unmixing) | 34 | 42 |
| 6. The water cap covered by its sample share | 34 | 42 |
| 7. The wobble reads colour at the same whole pixel as depth | 33 | 42 |
| **8. No multisampling by default** | **0** | **0** |

Steps 5 to 7 moved nothing because the cause was upstream of every water
pass. A capture of the raw scene (the compose passing it straight through)
showed the bright edge pixels already in it: the 4x multisample resolve
had blended rod and sky into one colour before any water shading ran, and
no later depth test can split a blend. With multisampling off by default
(step 8; `PBD_MSAA=2|4` brings it back) every pixel is one surface and the
edge is clean: `rod-edge-zoom-final.png` (before, step 7, step 8) and
`rod-edge-before-after.png`. Not shot:
the owner's darker scene (at dusk or night), and the hex steps against far
water at night.
