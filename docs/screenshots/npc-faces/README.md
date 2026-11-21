# M5 Faces tab (townsfolk design 13 to 18)

Screenshots of the Faces tab of `docs/mockups/npcs.html#faces` (M5 version 8, 2026-11-21, drafted),
taken in a Claude Code cloud container with no GPU: Chromium headless through Playwright with
SwiftShader (software rendering). They show the look, not the speed.

How they were made: the page served by `python3 -m http.server` from `docs/mockups/`, opened at
`npcs.html#faces` (1280 x 1000, the phone shots at 390 x 844), driven through the page's own buttons and
its `window.__faces` hook (`setExpr`, `setView`, `setHourF`, `setLayer`), and the stage element shot as JPEG.
three.js r128 was served from a local copy of the same file the page loads from cdnjs.

| File | Shows |
| --- | --- |
| `today-vs-proposed-day.jpg` | Today's hex figure (left, 1.9 m) beside the proposed friendly townsperson (1.47 m, 634 triangles with the face), happy, at 12:30, the seed 7 crowd behind |
| `expressions-contact-sheet.jpg` | The ten expressions, close on the face, with blink, gaze and head follow off so each is held: neutral, happy, delighted, surprised, worried, sad, cross, sleepy, thinking, talking |
| `generator-crowd-seed7.jpg`, `generator-crowd-seed8.jpg` | The generated crowd of eight for seed 7 and, after New person, seed 8: builds, heights, hair (bun, crop, braid, bald, hat), beards, noses, skin and clothes from the fields people's palette |
| `night-square-compare.jpg`, `night-square-crowd.jpg` | 22:30: the patch lit only by two lamp posts, the lit window and a brazier with the towns falloff, the moon at 0.035 |
| `night-face-close.jpg` | The proposed face at night by the brazier's light |
| `walk.jpg` | Play walk: the bouncy walk round the patch (bob, squash and arm swing) |
| `wave.jpg` | Wave, played from its button (some of the crowd wave too) while the walker heads home |
| `phone.jpg`, `phone-modal.jpg` | The tab at phone width, and its info modal |

Checked on these: no console errors in the Faces tab or after switching Square, Character sheets, Faces
and back to Square (the square's own lamps are restored when the Faces tab closes).
