# water-flow, volumetric-water phase 2 and tides: first captures

Fixed-step `--capture` shots on the Mac (M4 Max, Metal, `--profile fast`),
2026-11-15, branch `claude/env-water`. Looks only: frame time was NOT
measured. The rain, the near shower and the lens drops were turned off for
clarity (`PBD_PASS_OFF=lens,rain,shower`); the wet ground still shows.

The rig (`--pour N`, `--pour-ahead M`, `--pit DEPTH`, `--water-ticks N`,
`--arrows`, `--tide high|low`) pours sources, digs a pit and runs N ticks of
the rule at once, so each picture shows water that has had N seconds to move.

| File | What it shows | Requirement |
| --- | --- | --- |
| `before-tile-slope-pour.jpg` (was `slope-pour.jpg`) | One source poured on the terraced desert hill 58 m ahead (spawn, `--yaw 90`), 30 ticks: the sheet runs down the 1 m terraces and spreads on the flat below. | Water flows by Luanti's rule; meshed per cell |
| `slope-pour-arrows.jpg` | The same with the arrows on: down arrows on every terrace's falling face, flat arrows along the sheet. | A flow-arrow overlay |
| `waterfall-t2.jpg`, `-t4`, `before-tile-waterfall-t8.jpg` | A source poured beyond a 9-deep pit (7 cells), after 2, 4 and 8 ticks: the water falls into the pit as whole blocks with their own sides. | Flowing water is meshed per cell |
| `waterfall-arrows.jpg` | The pit at 8 ticks with the arrows: down arrows on each falling block's open faces. | A flow-arrow overlay |
| `waterfall-sheet.jpg` | The four above on one sheet. | |
| `pit-by-the-sea.jpg` | A pit dug 5 deep beside the sea at 20.0, 26.0: the sea floods it to sea level on the edit; its rock reads as under water. | volumetric-water phase 2 |
| `tide-high-beach.jpg`, `tide-low-beach.jpg` | One beach (20.0, 26.0) with the tide's moon overhead and a quarter turn round: high water covers the sand bar, low water bares it, with no lip at the shore. | ocean-environments 5.1 |

What looks wrong, honestly:

- The water tile is big and busy close up (12.5 cm pixels, 2 m a tile, as
  the mockups have it); from the walker's height a flat sheet reads as
  confetti. Its contrast was eased once; the owner should judge it.
- The waterfall stages are hard to tell apart from this angle: by tick 2
  the far wall already carries a curtain, and the pit's floor shows only at
  t2. A view from the side of a natural cliff would read better; none
  steeper than 3 m was found near the spawn.
- `pit-by-the-sea.jpg`: the pit's rock is tinted as under water, but the
  sea sheet over it is faint at this angle; the picture does not prove the
  surface is drawn there.
- The `--view seacave` capture panics before its frame on this branch and
  on `claude/env-v2` alike (a missing resource in that view, not this
  change), so no capture inside a flooded sea cave was taken; the tier tests
  (`water_tier_tests`) prove the flood instead.
- No night capture of the flowing water was taken.

## Second pass (2026-11-15): one water, the walker, the bucket, the sea cave

The poured and flowing water is now drawn by the sea's own fragment
(`water-flow` design E9), not the unlit tile.

| File | What it shows |
| --- | --- |
| `look-before-after-sheet.jpg` | Top: the slope, the tile cut (pitch 4) then the sea's fragment by day and at 22:00 (pitch -8). Bottom: the pit's waterfall the same way. |
| `after-slope-11.jpg`, `after-slope-22.jpg` | The hill's sheet by day and night: the sea's green-blue over the sand, the bed seen through it, darkening with the sky at night. |
| `after-fall-11.jpg`, `after-fall-22.jpg` | The pit's falling blocks as white water streaming down, by day and night. |
| `seacave-flooded.jpg`, `-brightened.jpg` | `--view seacave` (its panic fixed): the chamber the capture stands in is joined to the sea, so it is now flooded (`volumetric-water` phase 2) and the eye is under water in an unlit cave; the second is the same frame times 4. |
| `pan-two-frames-diff.jpg` | Frames 200 and 260 (one second apart) of the slope shot, differenced: 30.9 % of the water band's pixels move by more than 24 levels, 0 % of the sky, 7.9 % of the wet sand (its rain rings). |
| `bucket-icons.png` | The empty and full bucket icons (16 px, x8) beside the milk pail they are drawn after. |

Honest notes:

- The 3.5 check as written ("flowing differs, still does not") cannot pass
  with the sea's fragment: its waves move everywhere, still water and sea
  included, as the sea always has. Flowing tops stream their waves along the
  flow and falls down the face; still water only ripples in place.
- The sheet's edge steps on the terraced hill read as faint pale triangles
  (Luanti's sloped edge cells and short falls, lit by the sky at a grazing
  angle).
- At night the water is as dark as the sea is at that hour; no lamp lights
  the poured water (the sea's fragment takes no lamps).
- No capture shows poured water meeting the sea in one frame: the beach tried
  put the eye inside the poured water. Both are the same fragment, so no
  seam in colour is possible, but it is not shown.
- The bucket is not shown in use: there is no capture rig for the right
  button; its rule is tested (`a_bucket_takes_a_source_and_pours_into_air`).
