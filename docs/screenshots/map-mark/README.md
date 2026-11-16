# The map's mark (`starter-missions` G3)

Captured 2026-11-16 in a cloud container on lavapipe at 1440x900, with no
GPU, from the fast build on a memory-only new world. It shows what the map
looks like, not how smoothly it runs; frame cost was not measured.

| file | flags | what it shows |
| --- | --- | --- |
| `mark.jpg` | `--walk --time 12 --menu map --map-mpp 2 --map-mark-east 230 --rain 0 --weather-at 7200` | The player's mark: the red pin, its point on the place 230 m east of the player (the white arrow), stored through the save's `map-mark` record by the `--map-mark-east` capture instrument, as a right click stores it. The key line reads "right-click mark". |
| `waypoints.jpg` | the same, with `--map-mark-east 120 230 180 260 150` (five waypoints, each turned 72 degrees on from east) | Up to four waypoints (owner, 2026-11-16): five were set and four stand; the first, 120 m due east, was dropped by the fifth. |
