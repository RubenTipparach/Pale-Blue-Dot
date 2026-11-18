# Tools in Hand: videos

The step's video for the Tools in Hand mockup (M36, `docs/mockups/tool-motion.html`,
https://claude.ai/artifact/CwwZtbNGjZSVtjivQAMuzL): a technical montage of the motion of the
tool in hand, today against proposed, and a clip per topic. The owner, 2026-11-17: "make some
videos of the animations. they look freaking awesome, though I think the casting fishing line the
hand should be raised above shoulder". The cast shown is the raised, overhead one that note asked
for (`rod-motion` decision 7).

`index.html` is the gate page: the montage first, then one player per clip, each clip's shot
list in its info modal, the tracker and the live mockup linked at the top. It references the
videos by relative path, so it is published with them as supporting files.

## How they were made

- Rendered from the mockup in headless Chromium on **SwiftShader** (software WebGL; the cloud
  container has no GPU), 1280 x 720 at device scale 1, the 3D view filling the top 1280 x 656 and
  the caption strip below it.
- The page's clock was stopped (`window.__toolMotion.views[tab].frozen`) and stepped exactly 1/30 s
  per frame (`.advance()`), so the motion and its timing are the mockup's own whatever the render
  speed. They show the look and the timing, **never the frame rate or how smoothly it runs**.
  Smoothness is judged on the owner's machine (`obs-record`).
- Every motion is the page's own: the Dig and Chop views' dig button held, the Fish view's bot
  playing the line through the same controls as the mouse, the Hand view's Play button. The only
  scripted camera move is the Hand view's turn, standing in for a drag. Waiting is not sped up;
  the x0.25 shots are the page's own Slow switch, and their captions say so.
- Captions are burned into the frame by ffmpeg `drawtext` (Inter): the shot's title with TODAY or
  PROPOSED, a line on what it shows, and "SwiftShader, fixed 30 fps step".
- Encoded with ffmpeg: H.264, yuv420p, CRF 23, `-movflags +faststart`, no audio.

## The files

| File | Shots | Length | Size |
| --- | ---: | ---: | ---: |
| `tools-in-hand-montage.mp4` | 19 | 110.8 s | 9.4 MB |
| `tools-in-hand-dig.mp4` | 5 | 33.0 s | 3.6 MB (CRF 25 to stay under 4 MB) |
| `tools-in-hand-chop.mp4` | 3 | 17.0 s | 2.2 MB |
| `tools-in-hand-fish.mp4` | 7 | 37.0 s | 1.9 MB |
| `tools-in-hand-hand.mp4` | 4 | 23.8 s | 0.8 MB |

## The shots, in order

| # | Caption | What it shows | Demonstrates |
| ---: | --- | --- | --- |
| 1 | Dig - pickaxe on stone - TODAY | The game's swing today: one sine wave pitched about the fist, run while the block breaks. | The before for `held-swing` |
| 2 | Dig - pickaxe on stone - PROPOSED | The four-stage stroke on the shoulder arc at its steady beat, the last hit on the break. | `held-swing` decisions 5, 6, 7 |
| 3 | Dig - pickaxe - PROPOSED at x0.25 | The same at a quarter speed, the stage name live at the top right. | `held-swing` decisions 7, 8 |
| 4 | Dig - shovel on grass - PROPOSED | A steep stab along the handle, angle up, lift the dirt. | `held-swing` decision 9 |
| 5 | Dig - shovel - PROPOSED at x0.25 | Stab (HIT), angle up about the blade in the ground, lift. | `held-swing` decision 9 |
| 6 | Chop - axe on a trunk - TODAY | Today's chop, the same sine wave. | The before for `held-swing` |
| 7 | Chop - axe on a trunk - PROPOSED | Raised over the shoulder, a held coil, a fast hit reaching out to the trunk, the kick back. | `held-swing` decisions 5 to 8 |
| 8 | Chop - PROPOSED at night | The same by the lantern on the bank, the moon dim. | CLAUDE.md, mockups lit at night |
| 9 | Fish - the cast - PROPOSED | The fist raised above the right shoulder, a held coil, the flick down to the water, the tip's wobble, the float landing. | `rod-motion` decision 7 |
| 10 | Fish - the cast - PROPOSED at x0.25 | The stages named: WINDUP (RAISE), ANTICIPATION (COIL), HIT (FLICK), RECOIL. | `rod-motion` decision 7; the `animation` skill |
| 11 | Fish - float, nibble, bite, strike - PROPOSED | The tip twitches on each nibble, dips on the bite; the bot clicks and the rod comes up with a fish on. | `rod-motion` design 2 |
| 12 | Fish - the fight - PROPOSED | Held up, pulled down and rolled while the fish runs, shaking with the tension, the meter moving, the fist cranking. | `rod-motion` design 2 |
| 13 | Fish - landed - PROPOSED | The same fight's end: landed, a small dip, the lift, the ease back. | `rod-motion` design 2 |
| 14 | Fish - a snap - PROPOSED | Reeling without easing off: the meter into the red, the snap, the ease back. | `rod-motion` design 2 |
| 15 | Fish - the cast - TODAY | For contrast: the still rod before `rod-motion`. | The before for `rod-motion` |
| 16 | Hand - the game's hand, axe grip | Turned by the script as a drag would turn it. | The before for `hand-model`, `rod-motion` design 1 |
| 17 | Hand - the proposed hand, axe grip | The proposed hand with its textures and shading, turned the same way. | `hand-model` |
| 18 | Hand - the axe swing on the hand alone | The proposed stroke on the hand, the camera stood back. | `held-swing` decisions 6, 7 |
| 19 | Hand - the rod's phases on the hand alone | Charge (raise above the shoulder, coil), cast, float, nibble, bite, fight, catch. | `rod-motion` design 2, decision 7 |

The per-topic clips are the same shots: dig 1-5, chop 6-8, fish 9-15, hand 16-19.
