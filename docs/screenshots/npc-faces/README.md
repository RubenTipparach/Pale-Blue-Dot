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

## Version 9 (2026-11-21, drafted): block heads, bigger faces, first person (townsfolk design 13a)

The `v9-` shots, made the same way (Chromium headless through Playwright, SwiftShader software
rendering, 1280 x 1000 and the phone at 390 x 844): they show the look, not the speed. First person
was set through `window.__faces.setFP(true, { x, z, yaw, pitch })`, the same state the First person
button, WASD and drag change.

| File | Shows |
| --- | --- |
| `v9-today-vs-proposed-day.jpg` | Today's hex figure (1.9 m) beside the proposed townsperson with the block head (0.5 x 0.6 x 0.45 m, 1.55 m in all, 678 triangles with the face), happy, 12:30 |
| `v9-expressions-contact-sheet.jpg` | The ten expressions on the bigger face, close, blink, gaze and head follow off so each is held |
| `v9-generator-crowd-seed7.jpg` | The seed 7 crowd with block heads, block hair caps, buns, hats and beards |
| `v9-night-square-compare.jpg`, `v9-night-square-crowd.jpg` | 22:30, lit only by the lamp posts, the window and the brazier |
| `v9-fp-day-meet-1.5m.jpg` | First person, 1.5 m in front of the proposed townsperson by day, looking level from the game's 1.6 m eye with its 60 degree vertical field of view: the head sits below eye level and looks up |
| `v9-fp-day-crowd-4m.jpg` | First person, meeting the crowd from about 4 m; today's 1.9 m figure on the right stands above the eye |
| `v9-fp-night-by-lamp.jpg` | First person at night beside a lamp post, the crowd turned to look at you |
| `v9-fp-walked.jpg` | After holding W (the game's 8 m/s walk), stopped short of the townsperson |
| `v9-back-to-orbit.jpg` | First person off: the orbit camera where it was |
| `v9-phone-first-person.jpg` | First person at phone width with the on-screen arrows and the crosshair |

Checked on these: no console errors in the Faces tab, in first person, or after switching Square,
Character sheets, Faces and back to Square (the only failed requests are the web fonts, blocked
in the container).
