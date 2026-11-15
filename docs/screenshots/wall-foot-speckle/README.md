# wall-foot-speckle

Screenshots for `openspec/changes/wall-foot-speckle`: the dark speckle on a
floor along the foot of a terrain wall, before and after the lamp bake's and
the bounce's sample is snapped to a whole layer.

Taken 2026-11-14 on an M4 Max Mac (Metal, the GPU), the fast build, 2880 x
1800, fixed-step captures of 200 frames, a new world for each. "Before" is
the shader at `d43d018` (`claude/perf-tuning`), "after" is this change.

| file | flags | what it shows |
| --- | --- | --- |
| `crop-coast-night-before-after.png` | `--walk --at 30 177.5 --pitch -35 --time 10 --rain 0 --weather-at 7200 --torch` | night by the torch in hand: the sand at a 1 m sand wall's foot, x 1050-1500, y 800-1150, 2x. Before: dark dots densest at the foot and thinning over about a metre; after: none |
| `crop-coast-shade-before-after.png` | the same at `--time 22` (day at this longitude) | the shaded sand at the same wall's foot, x 1080-1330, y 1100-1300, 3x. Before: grey-blue dots; after: none |
| `before-coast-night.jpg`, `after-coast-night.jpg`, `before-coast-day.jpg`, `after-coast-day.jpg` | as above | the whole frames the crops come from |
| `bisect-sheet.png` | the day spot, before | the same crop with one switch each (design 2). Top: none, `PBD_BOUNCE=0`, `PBD_SOFT_SHADOWS=0`. Bottom: `PBD_MSAA=1`, `--no-shadows`, `PBD_LAMP_BAKE=0`. Only the last two clear it, and `--no-shadows` only because full sun swamps the lamp term |

Not z-fighting: the geometry is the same in every bisect run, one sample a
pixel (`PBD_MSAA=1`) leaves it unchanged, and switching the lamp bake alone
removes it.

Holbrook's lane (`--walk --at 29.69673 1.19287 --yaw 179.4 --time 23`, the
owner's second screenshot's setting) was captured too, but neither view
from that spot (yaw 179.4 and 90) had a terrace wall's foot in frame, so it
shows nothing either way and is not kept. The cause is the same at every
wall foot the bake covers.
