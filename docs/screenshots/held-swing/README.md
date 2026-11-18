# held-swing

The dig and chop strokes of the approved Tools in Hand mockup (M36 v8) in the
game (`openspec/changes/held-swing`, tasks 2.1-2.5): the pickaxe and the axe
swing from the shoulder in four stages, the shovel tips down, slides into the
dirt, levers and pulls up with a clump of dirt on the blade, on one steady
beat timed by the dig.

Each file is a contact sheet of one burst: 18 frames, every 3rd frame of the
fixed-step capture (0.05 s apart, read left to right, top to bottom), cropped
to the right two thirds of the view, each labelled with its frame number.

| Shot | Before | After |
| --- | --- | --- |
| A stone pickaxe on stone, at the bottom of a hole dug 4 deep (each block logged at 1.20 s) | `pickaxe-stone-before.jpg` | `pickaxe-stone-after.jpg` |
| A stone shovel on grass and the soil under it (0.5 s a block) | `shovel-grass-before.jpg` | `shovel-grass-after.jpg` |
| A stone axe chopping the ground (the capture rig cannot aim at a trunk) | `axe-chop-before.jpg` | `axe-chop-after.jpg` |

- Before is `54ec59cd`, the commit before this change: one sine (`swing_rate`
  9 rad/s), the tool pitching about the fist.
- After: the pickaxe and the axe rise up and back with the arm, pause, sweep
  forward and down to full reach with the sleeve showing, and recoil; the
  shovel's blade carries a clump of dirt through the scoop.

How they were taken:
- In-game captures on 2026-11-17 in a cloud container with no GPU, on
  lavapipe (software Vulkan) under Xvfb, 1440 x 900, the `fast` build, a new
  world per run. They show how things look, never how smoothly: frame cost
  was not measured.
- Each run saved a burst with `PBD_CAPTURE_RUN=<dir>,84,108,3`; the break
  script holds the use button from frame 90.
- Logs: no `ERROR` lines. A capture does not warm, so its pipelines are all
  logged late by design: 33-34 `LATE_PIPELINE` lines before, one more after
  (the hand's textured layout: positions, normals, UVs, colours). A normal
  warming launch of the after build (`--world hs-warm --walk --spawn fields
  --tool pickaxe`, 150 s) reached `READY` with 90 prewarm proxies and logged
  no `LATE_PIPELINE` after it.

Launch lines (common: `--walk --weather-at 1800 --time 10 --rain 0 --frames 200`):

    pbd-app --world hs-pick-deep --spawn mountains --dig 4 --pitch -85 --tool pickaxe --break 10
    pbd-app --world hs-shovel-grass --spawn fields --pitch -40 --tool shovel --break 10
    pbd-app --world hs-axe-chop --spawn fields --pitch -25 --tool axe --break 10

## The shoulder stays low (2026-11-18)

Decision 9's version 12 change in the game: `fist_drop_m` 0.16 (was 0.04)
and `arm_follow` 0.7 (was 1). The hand is version 13's (`hand-model`).

| Shot | Before | After |
| --- | --- | --- |
| A stone shovel on grass and the soil under it, frame 200 (the same moment of the stroke in both) | `shovel-low-before.jpg` | `shovel-low-after.jpg` |
| The same, a burst (every 3rd fixed-step frame from when the view is drawn, right two thirds) | `shovel-low-before-burst.jpg` | `shovel-low-after-burst.jpg` |

- Before is `8438582e`: the fist and the sleeve climb the right of the view
  as the spade goes in.
- After: the fist sits lower and leaves the bottom of the view through the
  tip-down and the slide, and comes back with the lift (the owner: "it's ok
  if shovel animation moves hand off screen for a split second").

Taken as above (lavapipe, `fast`, 1440 x 900, 2026-11-18; frame cost not
measured; no `ERROR` lines, 34 `LATE_PIPELINE` lines in each run):

    PBD_CAPTURE_RUN=burst,84,108,3 pbd-app --world hm9-sh --walk --weather-at 1800 --time 10 --rain 0 --spawn fields --pitch -40 --tool shovel --break 10 --capture shovel.png --frames 200
