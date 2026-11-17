# Waypoints on the compass and the map (`waypoints`)

Captured 2026-11-17 in a Claude Code cloud container at 1440x900. It has no
GPU, so these were rendered in software on lavapipe. They show how things
look, not how smoothly they run, and frame cost was not measured. Both are
from the `fast` build on a new world kept in memory only. The waypoints were
placed by the `--map-mark-east` capture instrument, which stores them through
the save's `map-mark` record the same way a right click does.

| file | flags | what it shows |
| --- | --- | --- |
| `compass.png`, `compass-crop.png` (the bar, at 2x) | `--walk --time 12 --yaw 180 --map-mark-east 230 --map-mark-east 1200 --rain 0 --weather-at 7200` | Facing east (`--yaw 180`) toward W1, set 230 m due east. Its red pin is under the oak pointer on the bar's foot row, and the line under the bar reads "W1 230 M". W2, 1.2 km away and 72 degrees north of east, is the pin to the left, toward NE. The brass studs are towns. |
| `map.png`, `map-crop.png` (around the pins, at 2x) | `--walk --time 12 --menu map --map-mpp 2 --map-mark-east 120 --map-mark-east 230 --map-mark-east 180 --map-mark-east 260 --map-mark-east 150 --rain 0 --weather-at 7200` | Five waypoints were set and four are kept, each with its label beside the pin. The first (120 m east, W1) was dropped by the fifth, which took its free number, W1. The others are W2 to W4. |

Launch line, from the worktree root:

```text
xvfb-run -a -s "-screen 0 1440x900x24" target/fast/pbd-app --capture <png> <flags above>
```
