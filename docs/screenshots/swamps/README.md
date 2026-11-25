# Swamps: screenshots

For `openspec/changes/swamps` task 4.1: the world map's biome layer on
generator version 10 beside version 11, and a version 11 swamp from inside
it. Captured 2026-11-18 in a cloud container on lavapipe (no GPU, software
rendering) at 1920 x 1200, each a fresh run of the fast build (`--profile
fast`) under `xvfb-run -a -s "-screen 0 1920x1200x24"` with
`PBD_WINDOW=1920x1200`, a new memory-only world. They show what things look
like, not how smoothly they run; frame cost was not measured (CLAUDE.md).

## Version 10 beside version 11

| file | flags | what it shows |
| --- | --- | --- |
| `map-v10.png` | `--generator 10 --walk --menu map --map-layer biomes --map-mpp 12 --time 12` | The whole planet's biome layer on version 10. The key: swamp 1%, jungle 26%, fields 27%. The swamp (dark grey-green) is a thin rim on a few wet coasts, hard to find anywhere. |
| `map-v11.png` | `--walk --menu map --map-layer biomes --map-mpp 12 --time 12` | The same on version 11, the generator a new world is made on. The key: swamp 7%, jungle 22%, fields 26%; desert, beach, mountains and tundra unchanged. The swamps run along the wet coasts and up the rivers' floodplains inland, mostly inside the jungle. Every height is version 10's, so the coasts, the rivers and the 55 settlements are where they were. |

The two maps were taken with the build at `b0737900`. The commit after it
(`b3ce7cff`) changes only which swamp cells hold a pool, never which cells
are swamp, so the biome layer is the same on the final build.

## A version 11 swamp from inside

A river swamp 203 m from the default spawn, found by
`planet_gen::tests::swamp_places_near_the_spawn` (14 pools among 36 points
round it), at 73 m over the sea. The final build (`b3ce7cff`).

| file | flags | what it shows |
| --- | --- | --- |
| `swamp-pools.png` | `--walk --at 27.9300 2.6279 --time 10 --pitch -25 --frames 120` | Looking down at the swamp moss, in rain: willows' hanging fronds and a birch, a held pool on the left and another behind, rain rings on both, mushrooms. |
| `swamp-eye.png` | `--walk --at 27.9300 2.6279 --time 10 --pitch -5 --yaw 90 --frames 120` | Eye level toward the river, in rain: a willow's fronds close in front, the river's open water on the right, moss and a mushroom under the trees. |
| `swamp-clear.png` | `--walk --at 27.9300 2.6279 --time 10 --rain 0 --pitch -10 --yaw 140 --frames 120` | Toward the north-east: the river through the swamp, willows along its bank with their fronds hanging, moss and tufts in front. |

The world's weather at that hour was rain, and `--rain 0` did not clear the
drops on the lens or the rings on the water in the last shot. The reeds
(decision 5) are not picked out in these frames: at this spot the moss's
tufts stand in front of them. A closer reed shot, and the same place in
clear weather, are left for the owner's machine.
