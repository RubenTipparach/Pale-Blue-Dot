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
