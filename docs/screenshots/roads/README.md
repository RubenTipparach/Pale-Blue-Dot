# roads: sloped road cells, dirt and gravel, a ford and a bridge (`townsfolk` 8.1 to 8.4)

## The mockup: M15's Roads scene (`docs/mockups/towns.html#roads`)

Rendered by `tools/mockup_roads_shots.js` in the Claude Code cloud
container: headless Chromium on **SwiftShader, no GPU**, 1440 x 900. They
show the look only; nothing here is a frame time.

    THREE_JS=<three.min.js r128> NODE_PATH=$(npm root -g) \
      node tools/mockup_roads_shots.js $PWD/docs/mockups/towns.html docs/screenshots/roads

The shots are JPEGs. Every spot by day (`-11h00`) and at night (`-22h30`), lit by the road's
own lanterns:

| Shot | What it shows |
| --- | --- |
| `mockup-roads-crossroads-*` | The dirt track meeting the gravel road at the foot of the hill road. |
| `mockup-roads-hillfoot-*` | The hill road from its foot: each cell a plane rising one layer, beside a hillside that steps a metre a row. |
| `mockup-roads-climb-*` | Half way up, after the bend. |
| `mockup-roads-hilltop-*` | From the turning place on the plateau, ten layers up, looking back down. |
| `mockup-roads-ford-*` | The dirt track going down a layer into the brook, over a gravel bed 0.3 m under the water, and up the far bank. |
| `mockup-roads-bridge-*` | The gravel road over the deep river on a railed timber bridge at the banks' height. |
| `mockup-roads-gravel-*` | The gravel road east. |
| `mockup-roads-overview-*` | The whole scene from above. |
| `mockup-roads-info-phone.jpg` | The info modal at a phone's width. |

The same script walks the mockup's walker up the hill road from its foot
to the turning place and counts the eye's jumps (a rise of more than
0.1 m in one 0.1 m piece of the sweep): it climbs 10 layers in 692 ticks
with **0 eye jumps**, the steepest rise 0.046 m a tick.

## The mockup: M15's Wide roads scene (`docs/mockups/towns.html#wideroads`, `townsfolk` design 10b)

Rendered by `tools/mockup_wide_roads_shots.js` the same way: headless
Chromium on **SwiftShader, no GPU**, 1440 x 900; the look only, no frame
time.

    THREE_JS=<three.min.js r128> NODE_PATH=$(npm root -g) \
      node tools/mockup_wide_roads_shots.js $PWD/docs/mockups/towns.html docs/screenshots/roads

| Shot | What it shows |
| --- | --- |
| `mockup-wide-parallel-*` | From the knoll: three roads side by side, one, two and three cells wide, dirt near, gravel beyond, climbing the hill together in the distance. |
| `mockup-wide-lanes-*` | Standing on the three-wide road looking east along it to the climb. |
| `mockup-wide-climbfoot-*` | The three-wide climb from its foot: one ramp across the whole width. |
| `mockup-wide-climbing-*` | Half way up the three-wide climb, across the lanes: no step between lanes; the cut bank on the uphill (north) side. |
| `mockup-wide-cutfill-*` | From below the road on the downhill (south) side: the road's bed above the stepped ground (fill), and the cut bank beyond it. |
| `mockup-wide-climb-above-*` | The climbs from above and the side: the one-, two- and three-wide ramps smooth beside ground that steps a metre a cell. |
| `mockup-wide-bend3-*`, `mockup-wide-bend2-*` | The three-wide road's bend (south-east) and the two-wide road's bend (north-east) on the plateau, from a lookout mound beside each. At eye height a bend is hard to read; the shot from above is the one to look at. |
| `mockup-wide-bends-above-*` | Both bends from above, level, with the one-wide road meeting the two-wide road's bend in a T. |
| `mockup-wide-junction-*` | A one-wide track leaving the three-wide road in a T. |
| `mockup-wide-bridge2-*`, `mockup-wide-bridge3-*` | Two- and three-wide timber bridges: a deck over the full width, rails only on the outer edges. |
| `mockup-wide-overview-*` | The whole scene from above. |

The same script walks the mockup's walker diagonally across every lane up
and down both wide climbs (1 to 11 m) and round both bends: **0 eye jumps**
on every walk, the steepest rise 0.027 m a tick at 120 Hz. It also samples
every shared edge of the 53 sloped wide cells from both sides: the largest
difference between two lanes' surfaces is 0.0007 m (the 1 mm sampling
offset across a 19-degree slope). Of those 53 cells, 23 are in step with the
plane (no cut, no fill) and 30 are half a cell along it (column cut a
layer, a bed 0.5 to 1.5 m).

Not right yet: the far ground past the map's north and south edges is cut in
slices a cell wide at each edge cell's height, and near the horizon thin
lines show between slices of different heights.

## The game (`townsfolk` 8.2 and 8.4)

Fixed-step `--capture` shots in the Claude Code cloud container:
**lavapipe, no GPU**, `--profile fast`, new memory-only worlds, 1440 x 900,
by `tools/capture_roads.sh`. Each run is a new world, so its loading thread
plans the roads as an old save's first open does: the log says "roads
planned: 22 of 49 links, 9269 cells, in 7.88 s" (the towns laid, cave towns
left out) and names each road's middle, first climb and first bridge as
`--at` and `--yaw`. They show the look only: **frame time was not
measured**. A capture skips the warm-up, so every capture logs its
pipelines as late by design.

| Shot | Command | What it shows |
| --- | --- | --- |
| `game-road-between-towns.jpg` | `--walk --at 29.60869 4.84535 --yaw 2.6 --time 11 --rain 0 --frames 150` | The dirt road from Holbrook to Linenleigh through the forest, its trees cleared, climbing a layer a cell in the distance. Grass tufts and a wild plant still stand on it (not yet cleared). |
| `game-road-into-holbrook.jpg` | `--walk --at 29.68193 2.25400 --yaw 1.2 ...` | The same road's end at Holbrook, running into the village's lane between two houses. |
| `game-road-climb.jpg` | `--walk --at 29.68193 2.25400 --yaw 181.2 --pitch 8 ...` | The road leaving Holbrook up the hill toward Linenleigh: each cell a plane rising a layer, between terraced ground that steps a metre a cell. No step on the road. |
| `game-road-climb-night.jpg` | the same, `--time 22.5` | The same at night: the road is as dark as the ground beside it, lit as the terrain is (no glow). The game's roads carry no lamps yet (the mockup's lanterns are not built). |
| `game-road-under-bridge.jpg` | `--walk --at 14.32551 59.25625 --yaw -31.4 ...` | **Not the shot asked for.** The spot is a bridge's first cell, so the walker stood on the river bed under it: the timber deck overhead and two of its posts, through the water. A shot from the deck is still owed. |

`--yaw` for these spots: the plan's log first gave the road's bearing,
which faced the other way (`game-road-into-holbrook.jpg` was taken
with it); the log now gives the turn `--yaw` takes.

The walker climbing a road is pinned by a test, not by these frames:
`walking::tests::a_walker_climbs_a_road_over_ten_layers_without_a_step`
(ten layers up and down again at the walking speed, no eye jump, never a
tick in the air).
