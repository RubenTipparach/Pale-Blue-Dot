# player-building: build mode in the game (`player-building` 4.2 to 4.6)

Fixed-step captures in the Claude Code cloud container: **lavapipe, no
GPU**, `--profile fast`, memory-only worlds, 1200 x 750, at the default
spawn on the desert (24.117 -2.964). They show the look only: **frame time
was not measured**.

`PBD_BUILD_SHOT` (a capture instrument, `desktop/build_mode.rs`) opens build
mode on a palette slot and, with `house:<n>`, places the first `n` pieces
of a template blueprint 18 to 20 m along the look, each through the rules
(`building::rules`) and `Builds::place` as a click would. The ghosts are
build mode's own aim: nothing is posed for them.

```sh
PBD_BUILD_SHOT="<plan>" PBD_WINDOW=1200x750 xvfb-run -a -s "-screen 0 1200x750x24" \
  target/fast/pbd-app --capture <shot>.png --walk --rain 0 --weather-at 7200 --time 11 <args>
```

| Shot | What it shows | `<plan>`, `<args>` |
| --- | --- | --- |
| `green-ghost.jpg` | The Wall slot (palette in the hotbar's place, the wall's icon lit), the aim snapped to the nearest cell edge: a storey-high wall ghost, green, on the ground in front. | `mode:0,at:150`, `--pitch -30 --frames 260` |
| `red-ghost.jpg` | The Roof slot aimed at open ground: no walls close round the cell, so the ghost (a cone a storey up, at the top of the frame) is red and the line under the crosshair says WALLS MUST CLOSE ROUND THE ROOM. | `mode:4,at:100`, `--pitch -13 --frames 220` (an earlier binary) |
| `house-1-floors.jpg`, `house-2-ground-storey.jpg`, `house-3-upper-storey.jpg`, `house-4-roofed.jpg` | A two-storey timber house (the village's six-cell Timber house as a blueprint) built piece by piece: its six floors; its ground storey's walls with their door and windows; the upper storey; and with its gable roof. | `choice:2,house:6` / `22` / `48` / `999`, `ahead:18,at:200`, `--frames 340` |
| `two-storey-house-night.jpg` | The same house whole at 22:00: its lanterns light its rooms, warm through the windows; the field lights the walls as the ground. | `choice:2,house:999,ahead:20,at:100`, `--time 22 --frames 240` |
| `kit-picker.jpg` | K in build mode: every kit the game ships (22), each with its wall, floor and roof textures; Timber, the chosen one, lit. | `picker,kit:2,at:150`, `--pitch -10 --frames 260` |

What is wrong in them, and stays as a limitation for now:

- **The house stands on the highest ground under it.** On the desert's
  terraces its floor slab hangs a layer or two over the lower cells, with
  nothing under it. A blueprint's ground floor goes on the highest cell so
  no floor is under the ground; a foundation or posts down to the ground
  are not built yet.
- Doors are doorways with no leaf (design B1).
- The house casts no shadow into the sun's cascades: player buildings are
  not on the towns' caster lists yet.

Captures skip the loading screen's warm-up, so each logs its pipelines as
`LATE_PIPELINE` by design; the ghost's material is spawned at startup so a
normal launch warms it.
