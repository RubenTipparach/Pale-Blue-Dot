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
| `red-ghost.jpg` | The Roof slot over open ground, seen from architecture mode (retaken 2026-11-24, task 6.3): no walls close round the cell, so the roof ghost, wholly in frame, is red and the line under the cursor says WALLS MUST CLOSE ROUND THE ROOM. | `mode:4,house:0,at:150,view,dist:12,tilt:35`, `--no-readout --frames 300` |
| `house-1-floors.jpg`, `house-2-ground-storey.jpg`, `house-3-upper-storey.jpg`, `house-4-roofed.jpg` (the last retaken 2026-11-24 from architecture mode, `choice:2,house:999,ahead:18,at:200,view,quiet`, the roof whole in frame) | A two-storey timber house (the village's six-cell Timber house as a blueprint) built piece by piece: its six floors; its ground storey's walls with their door and windows; the upper storey; and with its gable roof. | `choice:2,house:6` / `22` / `48` / `999`, `ahead:18,at:200`, `--frames 340` |
| `two-storey-house-night.jpg` | The same house whole at 22:00: its lanterns light its rooms, warm through the windows; the field lights the walls as the ground. | `choice:2,house:999,ahead:20,at:100`, `--time 22 --frames 240` |
| `kit-picker.jpg` | K in build mode: every kit the game ships (22), each with its wall, floor and roof textures; Timber, the chosen one, lit. | `picker,kit:2,at:150`, `--pitch -10 --frames 260` |

### The owner's notes (design B7 to B10, 2026-11-23)

The same timber house (`choice:2,house:999,ahead:18,at:200`), with
`doors` (a door put in the pack and hung in its doorway through
`Builds::place_door`), and architecture mode (`view`) over it. Run with
`--walk --rain 0 --weather-at 7200 --frames 300`; lavapipe, so the look
only, not the frame time.

| Shot | What it shows | Extra plan, args |
| --- | --- | --- |
| `architecture-day.png` | Architecture mode (Z in build mode) over the house at 11:00: the camera up and back, the palette, the cursor's ghost on the build layer (red: a wall there has nothing to stand on). The footing's coursed stone shows under the house. | `doors,view`, `--time 11` |
| `architecture-night.png` | The same at 22:30: the house as dark as the ground round it, its lanterns faint at the windows. | `quiet,doors,view`, `--time 22.5` |
| `footing-close.png` | The footing close up: the new `footing` texture under the floor where the desert falls away. | `quiet,doors,view,dist:12,tilt:18` |
| `door-shut.png`, `door-open.png` | The door hung in the doorway, shut, then opened a quarter turn into the room: one bevelled leaf of planks with its ledges, brace, strap hinges and pull painted in, in a U frame. | `quiet,doors,view:door,dist:7,tilt:28` (and `open`) |

### Double-click to walk and the cutaway (design B11, 2026-11-24)

The same house (`choice:2,house:999,ahead:18,at:200,doors,view,quiet`)
with `cut`, the cutaway End turns on in architecture mode. Lavapipe, so the
look only. Double-click to walk is pinned by its app tests, not a shot.

| Shot | What it shows | Extra plan, args |
| --- | --- | --- |
| `architecture-cutaway.jpg` | The cutaway at the ground storey, 11:00: the roof and the upper storey gone, the near walls cut 1.2 m over the floor, the far walls standing to the ceiling, so the floor, the stair and the doorways are in sight. The house's shadow on the ground is the whole house's: what is cut still casts. The terrain is not cut. | `cut`, `--time 11` |
| `architecture-cutaway-night.jpg` | The same at 22:30: the room lit warm by its lantern, the ground as dark round it. | `cut`, `--time 22.5` |
| `architecture-cutaway-upper.jpg` | The build layer a storey up (Page Up): the cut through the upper storey, its floor and the stair's well. | `cut,layer:3`, `--time 11` |

What is wrong in them, and stays as a limitation for now:

- **The house stands on the highest ground under it.** On the desert's
  terraces its floor slab hangs a layer or two over the lower cells, with
  nothing under it. A blueprint's ground floor goes on the highest cell so
  no floor is under the ground; a foundation or posts down to the ground
  are not built yet.
- Doors were doorways with no leaf (design B1) until design B8.
- The house casts no shadow into the sun's cascades: player buildings are
  not on the towns' caster lists yet.

Captures skip the loading screen's warm-up, so each logs its pipelines as
`LATE_PIPELINE` by design. A normal launch with this build (no `--capture`,
`--walk`, lavapipe) reached `READY` with 126 pipelines and logged no
`LATE_PIPELINE` in the six minutes after; the ghost's meshes are spawned
hidden at startup so the warm-up shows their material. Build mode itself was
not opened in that run.
