# river-fixes: under a river's water, and nothing grown on its bed

The owner, 2026-11-16: "problems with submerging in rivers, also tree and
grass is spawned there, river should be dirt"
(`../rain-artifacts/owner-river-43.webp`, `owner-river-44.webp`). Write-ups:
`openspec/changes/ocean-environments/design.md` B17 (submerging) and
`openspec/changes/flora-v2/design.md` 12.10 (the beds).

Fixed-step `--capture` shots on the Mac (M4 Max, Metal, `--profile fast`),
2026-11-16, branch `claude/river-fixes`. Looks only: frame time was NOT
measured. Each shot is a new version 10 world (`--world rf-<tag>-<shot>-<pid>`),
`--frames 200`, with the rain, the near shower and the lens drops off for
clarity (`PBD_PASS_OFF=lens,rain,shower`). Before is `origin/claude/env-v2`
at `afcf100`; after is this branch. The river is the version 10 spawn's
nearest, 60 m away: its bed is a meadow's `DryGrass` 62 to 63 m up, and
the column's water at the eye stands at 65.8 m.

| Files | Flags | What it shows |
| --- | --- | --- |
| `before-under.jpg`, `after-under.jpg` | `--fly --at 28.2500 0.7500 --height 0.8 --time 12 --pitch 5` | The eye 1.2 m under the river's surface, looking along the channel. Before: dry air, a green sward with tufts on the bed. After: the river's murk and tint, the bed dirt and bare, the banks above the surface seen through it. |
| `before-up.jpg`, `after-up.jpg` | the same, `--pitch 40` | Looking up at the river's sheet from below. Before and after alike, the sheet is drawn from below (its own murk and window); after, the water between the eye and the banks is fogged as well. |
| `before-walk.jpg`, `after-walk.jpg` | `--walk --at 28.7500 0.7500 --time 12 --pitch -15` | The walker spawned in the river, floating with its eye under the surface. Before: a tree and tufts on the bed beside it, red mushrooms, no water to be seen. After: the river's water, the bed dirt, no tree, tuft or mushroom. |
| `before-air.jpg`, `after-air.jpg` | `--fly --at 28.7500 0.7500 --height 12 --time 12 --pitch -50` | The river from 12 m up. Before: trees and tufts on the green bed under the water. After: the bed dirt under the water; the trees left stand on dry hexes whose tops are above the river's water. |

## Second pass: the waterline (owner, 2026-11-16)

"underwater ground tint is happening before the water line"; "the land
right above the water right there is also getting that subtle twisty
effect". Before is this branch's first pass (the composite measured from
the river's surface with a 0.8 m band above it); after is the waterline
fix (design B17, second pass).

| Files | Flags | What it shows |
| --- | --- | --- |
| `before-skim.jpg`, `after-skim.jpg` | `--fly --at 28.2500 0.7500 --height 2.65 --time 12 --pitch -4` | The eye 0.2 m under the river's surface, looking at its banks. Before: the teal fades up the dirt walls over the waterline. After: it stops at the water. |
| `after-bank.jpg` | `--fly --at 28.7500 0.7500 --height 7 --time 12 --pitch -22` | From over the bank: the banks dry and untinted to the water's edge; the bed under the water tinted by the terrain's own underwater term, which a river's bed never had. |
| `after-under.jpg`, `after-up.jpg`, `after-walk.jpg`, `after-air.jpg` | as above | Retaken where the shared GPU allowed; `after-under` and `after-up` are the first pass's (the waterline fix does not change a view from well under the surface). |

## Third look: "the blocks right before water line gets bright" (owner, 2026-11-16)

Not reproduced on this branch (merged with `claude/env-v2` at `dfb2416`).
Night captures in rain, with the lens and the rain volume off:

| Files | Flags | What it shows |
| --- | --- | --- |
| `night-river-bank.jpg` | `--fly --at 28.7500 0.7500 --height 7 --time 5 --pitch -22 --rain 1` | The river's banks at night: the cells at the waterline as dark as their dry neighbours. |
| `night-sea-shore-rain.jpg`, `night-sea-shore-dry.jpg` | `--walk --at 20.0 26.0 --time 5 --pitch -10`, with and without `--rain 1` | A sea shore at night. The far shore has a thin mustard strip at the water: the beach band's sand. Measured, it is no brighter than the near dry sand of the same material (mean (31, 29, 9) on the strip against (46-50, 39-43, 18-22) on the near sand), and it is the same wet and dry. |

So no change was made for it: nothing found lights a waterline cell
differently from its dry neighbours here. The owner's shot (its place and
time) is needed to find it.

Honest notes:

- The owner's own shot (a sandy channel near the sea, in rain) was not
  found on the map; the faults were read in the code and shown at this
  river. Several captures timed out on the GPU (other games were running
  on the machine) and were retaken.

- The fall pieces' white faces (top right of the aerial shot) and the rain
  rings on the water are the water's look, unchanged here; the white speckle
  is `claude/rain-artifacts`'s.
- The dry hexes in the valley (aerial shot) are the generator's: ground above
  the river's water, so grass and trees stand on them as on any bank.
- Digging a drowned bed still gives what the column holds (a meadow's grass):
  the bed is drawn as dirt, the material is unchanged (design 12.10).
- Every capture logs the `LATE_PIPELINE` lines that captures have always
  logged on this branch and on `claude/env-v2` alike (25 a run); the plain
  launch is checked separately.
- No night capture.
