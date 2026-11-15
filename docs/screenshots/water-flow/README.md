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
| `slope-pour.jpg` | One source poured on the terraced desert hill 58 m ahead (spawn, `--yaw 90`), 30 ticks: the sheet runs down the 1 m terraces and spreads on the flat below. | Water flows by Luanti's rule; meshed per cell |
| `slope-pour-arrows.jpg` | The same with the arrows on: down arrows on every terrace's falling face, flat arrows along the sheet. | A flow-arrow overlay |
| `waterfall-t2.jpg`, `-t4`, `-t8` | A source poured beyond a 9-deep pit (7 cells), after 2, 4 and 8 ticks: the water falls into the pit as whole blocks with their own sides. | Flowing water is meshed per cell |
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
