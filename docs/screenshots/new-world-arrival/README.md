# New world arrival: before and after

For `openspec/changes/new-world-arrival`. Captured 2026-11-14 on the M4 Max
Mac (Metal, the GPU), the fast build (`--profile fast`), 1440 x 900 shown at
1280. They show what a switch looks like, not how smoothly it runs; frame
cost was not measured.

Each run makes a world at 28.3 S 1.5 E, away from the default spawn, and digs
three blocks there (`--world nw-old-<tag> --at 28.3 1.5 --walk --dig 3`),
then reopens it and makes a new world from inside it a third of the way to
the shot, as the saves page's NEW WORLD does (`--new-world nw-new-<tag>`, a
capture instrument). `WORLD_SWITCH before/after` log the world, its seed,
generator and edit count, its saved pose, the walker, its spawn direction and
the clock. The script is `run.sh` below; `after-rt-*` is the same switch with
`--real-time`, so the loading screen warms as it does for a player.

| file | what it shows |
| --- | --- |
| `before-old-make.jpg` | The old world: the player in the three-block hole at 28.3 S 1.5 E. |
| `before-after-switch.jpg` | Before the fix, 600 frames after NEW WORLD: the player is still at 28.300 1.500 (the HUD's second line), on the old world's ground. No loading screen was shown, no name. |
| `before.log` | `WORLD_SWITCH`: walker `lat 28.3000 lon 1.5000` before and after, spawn direction unchanged, clock 1440 s carried. |
| `after-after-switch-leaving.jpg`, `after-rt-leaving.jpg` | After the fix: the old world, ten frames before NEW WORLD. |
| `after-after-switch-loading.jpg`, `after-rt-loading.jpg` | The loading screen during the switch, building the new world's planet. |
| `after-after-switch-arrival.jpg`, `after-rt-arrival.jpg` | Arrival: the world's name and "A NEW WORLD", at 24.117 S 2.964 W, the new-world spawn, with the starting kit. In real time the fine set stands round the player (`0 m from anchor`); the capture lifts on the first open frame, before the set round the level-ground start lands (`485 m from anchor`), as a launched capture does. |
| `after-after-switch.jpg`, `after-rt.jpg` | The new world at the end of the run, banner gone. |
| `after.log` | `WORLD_SWITCH`: walker at `lat 24.1171 lon -2.9638` after; a fresh launch of a new world (`--world nw-fresh-a2`) spawns at the same 24.117 -2.964. In real time: `READY` after the switch, and no `LATE_PIPELINE` besides the instrument's own `screenshot-to-screen`. |

```bash
# run.sh <out> <tag>
pbd-app --world "nw-old-$TAG" --at 28.3 1.5 --walk --dig 3 --time 12 --capture "$OUT/old-make.png" --frames 600
pbd-app --world "nw-old-$TAG" --walk --time 12 --new-world "nw-new-$TAG" --capture "$OUT/after-switch.png" --frames 900
# rt: the same second run with --real-time
```
