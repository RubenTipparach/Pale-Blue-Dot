# The goals on the HUD (`starter-missions` J.1)

Captured 2026-11-17 in a cloud container on lavapipe at 1440x900, with no
GPU, from the fast build on a memory-only new world:
`--walk --spawn fields --time 12 --pitch -8 --rain 0 --weather-at 7200`.
`--missions <n>` (a capture instrument) raises the first n steps of the
mission script, as the game's systems raise them.

| file | flags | what it shows |
| --- | --- | --- |
| `progress.jpg` | `--missions 11 --frames 400` | "Gather wood" partway: the met goal (Fell trees 2/2) green with its check on, the one to do next (Gather logs 0/10) in white. |
| `complete.jpg` | `--missions 8 --frames 150` | "Gather by hand" just finished: MISSION COMPLETE over its title, every goal met, for the notice's four seconds (the jingle plays as it opens). |

The jingle is `assets/audio/sfx/mission_done.ogg`, with a share copy in
`docs/mockups/sounds/mission_done.mp3`.
