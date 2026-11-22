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
