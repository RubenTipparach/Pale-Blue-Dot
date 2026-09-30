# cities-in-the-world

Screenshots for `openspec/changes/cities-in-the-world`: slice 1 (Holbrook
stands in the game) and slice 2a (walls and doorways), each game shot beside
the towns mockup's village from the same spot.

The game shots are in-game captures, retaken on 2026-09-30 by
`tools/capture_holbrook.sh`:
- in a cloud container on lavapipe with no GPU;
- at 1440 x 900, from the fast build (`--profile fast`), under `xvfb-run`;
- each in a new memory-only world, 150 frames in;
- at `--time 11 --rain 0 --weather-at 7200`.

They show what the town looks like, not how smoothly it runs. Frame cost
was not measured.

What changed since the shots of 2026-09-29, side by side with the mockup, is
in `retakes-2026-09-30.jpg`:
- **The town is built from its stored record** (slice 3a), not from the
  template.
  - Every shot's log reads the same town: 12 buildings, 31,068 triangles,
    a terrace at 74 m over 799 cells, eased over 2,051.
  - `settlement::tests::a_town_built_from_its_record_is_the_town_its_template_lays`
    holds the pieces and the ground to what the template laid.
- **The sky is dry.** `--rain 0` forces no storm, but the weather still
  rained over Holbrook at 11:00. `--weather-at 7200` runs the weather two
  hours on, to a dry sky, and leaves the clock at 11:00
  (`towns::tests::print_the_rain_over_holbrook`).
- **`--at` stands exactly on its spot.** It had stood 4 m west of it.
  - The outside shot now looks at the Fieldstone house's door.
  - The inside shot now looks out through that door, as the mockup does.
  - The 2026-09-29 inside shot faced a wall.

`towns::tests::print_where_to_stand_for_the_mockup_shots` printed the spots.
It lays Holbrook out and turns a layout cell or a door into `--at` and
`--yaw`.

The mockup shots are `docs/mockups/towns.html#village` in headless Chromium
at the same size, at 11:00 with its doors opened. The game draws its doors
open until they open and shut as world state.

| game | mockup | flags | what it shows |
| --- | --- | --- | --- |
| `game-lane.png` | `mockup-lane.png` | `--walk --at 29.69673 1.19287 --yaw 179.4` | The lane at layout cell (20, 17), looking along it: the houses in pairs two empty columns apart, on the terrace. Nothing is in hand: the tool slot shows bare hands. |
| `game-outside.png` | `mockup-outside.png` | `--walk --at 29.71981 0.82332 --yaw 59.2` | The Fieldstone house from 3.5 m outside its door. |
| `game-inside.png` | `mockup-inside.png` | `--walk --at 29.66752 0.78743 --yaw -120.8` | Slice 2a: inside the Fieldstone house, 1.6 m in from its door, looking out through it: the leaf swung in on the left, the lane, and the house opposite. The mockup's house opposite shows its open door there, and the game's shows a window. |
| `game-overview.png` | `mockup-overview.png` | `--view column --at 29.70 2.18 --height 60 --pitch -42` | The village from above and to the east. |
| `retakes-2026-09-30.jpg` | | | The lane, outside and inside shots of 2026-09-29, of 2026-09-30, and the mockup's, in three columns. |
| `holbrook-lane.png` | | `--walk --at 29.70 1.36` | Slice 1's first shot, before the fixes: each frame was turned up to 60 degrees off the rows, so the roofs sat crossways, and houses one column apart had eaves that met. Kept to show what changed. |

What differs from the mockup, and why, is in the design (decision 9, "How
Holbrook differs from the mockup's village").
