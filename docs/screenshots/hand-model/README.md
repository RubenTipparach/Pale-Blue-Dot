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

## Versions 9 to 13 (2026-11-18)

Tools in Hand versions 9 to 13 in the game (`hand-model` design 5 to 5.3,
built as design 6): the traced palm, the slimmer tapering fingers, the
thumb's round tip and sunk root, the back and the palm by side with the
palm's lines and the folds, the baked AO, the fingertips' flush and the 45
degree grip turn.

| Shot | Before | After |
| --- | --- | --- |
| The pickaxe at rest, the whole view | `v13-rest-before.jpg` | `v13-rest-after.jpg` |
| The hand close (the right half, lower two thirds, of the same frame, 2x) | `v13-hand-close-before.jpg` | `v13-hand-close-after.jpg` |
| A stone axe chopping the ground, frame 200 (the same moment of the stroke in both) | `v13-chop-before.jpg` | `v13-chop-after.jpg` |
| The chop, a burst (every 3rd fixed-step frame from when the view is drawn, right two thirds) | | `v13-chop-after-burst.jpg` |

- Before is `8438582e`, the commit before this change: version 8's hand
  (chamfered blocks, one skin tone, the box palm).
- After: the grip is turned about the handle so the back of the hand and the
  thumb face the eye, as in the owner's photo; the back is browner, the
  fingers slimmer and tapering with redder tips, the paler palm side shows
  in the curl of the fingers, and the thumb's root sits in the hand's mass.
  The hand and textures were checked against the mockup's own code, run in
  node, not by eye: the same vertices and the same pixels.

How they were taken: on 2026-11-18 in a cloud container with no GPU, on
lavapipe (software Vulkan) under Xvfb, 1440 x 900, the `fast` build, a new
world per run. They show how things look, never how smoothly: frame cost was
not measured. Logs: no `ERROR` lines; 33 to 35 `LATE_PIPELINE` lines in
each run, before and after alike (a capture does not warm).

    pbd-app --world hm9-rest --walk --weather-at 1800 --time 10 --rain 0 --spawn fields --pitch -10 --tool pickaxe --capture rest.png --frames 200
    PBD_CAPTURE_RUN=burst,84,108,3 pbd-app --world hm9-chop --walk --weather-at 1800 --time 10 --rain 0 --spawn fields --pitch -25 --tool axe --break 10 --capture chop.png --frames 200
