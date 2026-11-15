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

## Third pass (2026-11-15): rivers and marsh pools, generator version 8

New worlds only (old saves keep their generator). Version 8 is version 7
plus rivers and rimmed marsh pools; version 7's noise channels stay as sea
inlets.

| File | Flags | What it shows |
| --- | --- | --- |
| `river-valley-day.jpg` | `--walk --at -34.6830 101.4869 --yaw 14 --pitch -6 --time 5` | A river coming down a terraced valley: a short fall of white water into a still reach, the reach running on toward the sea (lower left). |
| `river-valley-night.jpg` | the same, `--time 17` | The same at night. |
| `river-valley-arrows.jpg` | the same, `--arrows` | The arrows: down the fall's faces and along the sheet below it. |
| `marsh-pool-day.jpg` | `--walk --at 57.0007 -89.9243 --yaw 0 --pitch -25 --time 17` | A marsh pool held by its rim, under the trees. |

Honest notes:

- No frame shows a river from its spring to its mouth: each picture shows a
  stretch. The spring is a river's lattice head in the hills (design 13).
- Most of a river's steps are one or two layers: the ground is gentle at a
  cell's scale. M25's plunge pools are not generated.
- Beyond the column tier (about 90 m) a river's water is not drawn; its
  valley shows.
- The marsh frame is dark under the trees; the pool reads, the rim is the
  bank beside it.

## Fourth pass (2026-11-15): the owner's fall notes, rivers settled by the rule

| File | What it shows |
| --- | --- |
| `owner-fall-froth-rain-before-after.jpg` | A 9-layer fall into a pit (`--pour 1 --pour-ahead 12 --pit 9`): top row clear weather before and after, bottom row `--rain 1` before and after. After: white froth streaking down the faces; no rain rings on the vertical faces (before shows them). The light blue at the fall's foot is the splash. |
| `fall-splash-2-and-12-layers-day-night.jpg` | A 2-layer step and a 12-layer fall, by day (top) and at 22:00 (bottom). |
| `river-valley-from-60m.jpg` | `--fly --at -34.6830 101.4869 --height 60`: the valley from the air. |

Honest notes:

- The splash is hard to see in these frames: the 12-layer fall's foot is at
  the bottom of a pit, hidden by its near wall, and the 2-layer step's
  splash is a few small drops. No frame shows a tall fall's mist plume.
- The froth reads as blotches more than streaks close up.
- The lead's captures for rivers were not achieved: no frame from the air
  shows a river from high ground to the sea (from 60 m over the valley the
  river is down in its cut and not in view), no gorge fall (the terrain
  moved with the wider valleys and the fall found before is gone), no wide
  lower reach. The far rivers' sheet is built and tested only by the build,
  not seen in a picture.
- `river-valley-*` and `marsh-pool-day.jpg` above were taken before this
  pass's valley changes.

## Fifth pass (2026-11-15): falling froth, the lip, a river held, the aerial rig

| File | What it shows |
| --- | --- |
| `froth-lip-before-after.jpg` | The fourth pass's fall (`--walk --pitch -40 --yaw 90 --pour 1 --pour-ahead 12 --pit 9 --water-ticks 10 --time 11`), top row; the same pour at `--pitch -15`, bottom row. Before (left) is the committed shader and mesher on today's ground; after (right) this pass. After: thin vertical ribbons that leave the lip clear and grow longer and denser lower down; the top's last metre before the lip whitens. The white at the foot is the splash's mist. |
| `river-from-air-200m.jpg` | `--fly --at 24.0797 1.7719 --yaw 160 --pitch -35 --height 200 --time 12`: a river running down from a lake. |
| `river-reach-from-air-150m.jpg` | `--fly --at 24.2400 3.8543 --yaw 149 --pitch -35 --height 150 --time 12`: a lower reach, its valley and banks. |
| `far-rivers-from-250m.jpg` | `--yaw -20 --pitch -30 --height 250`: rivers far past the column tier (3,100 columns round the camera) drawn as the far sheet. |
| `river-from-air-night.jpg` | The 200 m view at 22:00. |

Honest notes:

- The spill the lead saw was underground. On four tiers over falls, the
  flowing water "outside the banks" was in caves the worms open under and
  beside a river, tens of metres down. Rock under a river and 15 m past its
  banks now keeps no cave above four metres under the sea's level, and a 3 m
  levee, falling away at one in one, stands a layer over the reach's water.
  `a_freshly_built_tier_s_rivers_are_what_the_rule_keeps` asserts no flowing
  water over the sea's layer outside a river's banks on all four. Water at
  the sea's layer in a sea cave is the sea's and is not counted. A plunge
  pool's edge now slopes up the bank rather than stepping two metres. The
  version 8 digests are re-pinned; the seam report holds at none of 4,000.
- The acceleration is in the shader: a fall's ribbons are read at the time
  the water left the lip, free fall from 1 m/s, so they speed up and stretch
  with sqrt(2 g d). A still frame shows the stretch, not the motion; the
  motion is for the recording.
- The top's whitening is drawn per triangle (corners at a lip are one, the
  middle nought), so up close it reads in facets.
- No frame shows a river's mouth at the sea or a tall river fall with its
  plume from the air: the tall fall here is the poured pit's nine layers.
- The start's cloud cover stays inside `new_world_start`'s 0.03 of 0.69
  (an earlier try at this pass, a deeper water clamp, moved it to 0.66 and
  was dropped).
- Captures in the `--capture` fixed-step mode on the Mac (M4 Max); frame
  cost not measured.
