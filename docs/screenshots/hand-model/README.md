# hand-model

The proposed hand of the approved Tools in Hand mockup (M36 v8) in the game
(`openspec/changes/hand-model`, task 2.3): chamfered blocks on the same bones,
one skin tone, nail plates, a wrist into the cuff, the thumb in two segments,
16 px nearest-sampled textures, and the wrapped light in eye space.

| Shot | Before | After |
| --- | --- | --- |
| The pickaxe at rest, the whole view | `rest-before.jpg` | `rest-after.jpg` |
| The hand close (2x crop of the same frame) | `hand-close-before.jpg` | `hand-close-after.jpg` |

- Before is `54ec59cd`: hexagonal prisms, the middle finger segments a darker
  paint, a flat shade.
- After: one skin with its mottle, the chamfers and bevelled knuckles reading
  as form, paler nails on the tips, the lit side up and toward the eye.
- The swinging hand (re-lit as it turns) and the sleeve's cloth weave and the
  cuff are in the held-swing captures (`docs/screenshots/held-swing/`).

How they were taken: the same container, lavapipe, `fast` build and 1440 x
900 as the held-swing captures, the last frame (200) of
`pbd-app --world hs-hand-rest --walk --weather-at 1800 --time 10 --rain 0
--spawn fields --pitch -10 --tool pickaxe --capture rest.png --frames 200`.
Frame cost not measured (no GPU). Logs: no `ERROR` lines.
