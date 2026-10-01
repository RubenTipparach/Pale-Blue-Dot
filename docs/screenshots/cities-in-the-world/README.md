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

## Slice 2b: stairs, floors and doors (2026-09-30)

`tools/capture_holbrook_2b.sh` takes the game's shots at the same settings as
above. `tools/mockup_village_2b_shots.js` takes the mockup's same views, at
11:00. `slice-2b-2026-09-30.jpg` puts each game shot beside the mockup's.
- **Flags.** `--up 3.2` stands the walker on the upper floor.
  `--open-doors` opens the doors without saving it; doors start shut.
- **Where the spots come from.**
  `towns::tests::print_where_to_stand_for_the_mockup_shots` prints them.
  The mockup's stairs are found by the rule the game's cutter follows.

| game | mockup | flags | what it shows |
| --- | --- | --- | --- |
| `game-2b-newel-below.png` | `mockup-2b-newel-below.png` | `--walk --at 29.63663 0.80156 --yaw -0.4 --pitch 20 --open-doors` | The Fieldstone house's newel from the room beside it, through its doorway: the post and the winders climbing. |
| `game-2b-newel-above.png` | `mockup-2b-newel-above.png` | the same with `--up 3.2 --pitch -35` | The same doorway from the upper floor: the landing, its rail, and the winders turning down. |
| `game-2b-flight-foot.png` | `mockup-2b-flight-foot.png` | `--walk --at 29.63542 0.98162 --yaw 179.5 --pitch 20 --open-doors` | The half-timbered house's straight flight from before its foot, boxed between its walls. |
| `game-2b-flight-landing.png` | `mockup-2b-flight-landing.png` | `--walk --at 29.63465 1.08097 --yaw -0.5 --pitch -28 --up 3.2 --open-doors` | Down the same flight from its landing, the well railed on both sides. |
| `game-2b-door-shut.png` | `mockup-2b-door-shut.png` | `--walk --at 29.71981 0.82332 --yaw 59.2` | The Fieldstone house's door, shut: a new world's doors start shut, and the walker stops at them. |
| `game-2b-door-open.png` | `mockup-2b-door-open.png` | the same with `--open-doors` | The same door open, its leaf swung in against the wall. |

## Slice 4: built towns on the map (2026-10-01)

The owner: "highlight built cities on the map please". In-game captures,
2026-10-01: a cloud container on lavapipe at 1440 x 900 with no GPU, the
fast build under `xvfb-run`, a new memory-only world at `--time 12`.

| file | flags | what it shows |
| --- | --- | --- |
| `game-map-built-spawn.png` | `--walk --menu map --map-mpp 12` | The spawn's continent. A built town's marker is cream with a green ring: Holbrook, Linenleigh, Holingmouth, Holmouth, Ashenstead, Dunmouth and the rest. A site whose kind has no town yet is dimmed, its name too: Durnacrag, Huatzico, Barberg, Coringport. The legend reads "55 settlements on the map: 26 built, 29 still to come." |
| `game-map-built-world.png` | `--walk --menu map --map-mpp 40` | The whole planet: the 20 villages and 6 walled towns ringed, the harbours, desert, tundra, jungle, swamp, cliff and cave sites faint. |

## Slice 4a to 4c: the villages and walled towns in the world (2026-10-01)

Every village and walled-town site stands in the game: 20 villages and 6
walled towns on the shipped seed, each turned by its site and laid on its own
ground. The walled town stands on its levels, inside its curtain wall, with
its gates, two stair towers and the keep. In-game captures, 2026-09-30 and
2026-10-01: a cloud container on lavapipe at 1440 x 900 with no GPU, the
fast build under `xvfb-run`, a new memory-only world, `--view column`,
`--open-doors --rain 0 --frames 150`. Nothing here says how smoothly the
towns run; the frame cost was not measured (CLAUDE.md, cloud sessions).

`--time` is the home village's hour. A town far east or west of it is at its
own hour of the sun, so each shot is labelled with the town's local sun time,
from its longitude. `towns-4abc-2026-10-01.jpg` puts the nine side by side.

| file | town | flags | what it shows |
| --- | --- | --- | --- |
| `game-4c-linenleigh-street.jpg` | Linenleigh (walled) | `--at 29.41553 8.48*  --height 20 --pitch -18 --time 11` | Over the roofs toward the far wall: the houses in their kits, the wall and its merlons, the towers' cones. 11:30 there. |
| `game-4c-linenleigh-above.jpg` | Linenleigh | `--height 60 --pitch -42 --time 11` | The whole town from 60 m: the curtain wall round its terraces and its gate, the hamlet's huts outside it. |
| `game-4c-ashingstead-above-dusk.jpg` | Ashingstead (walled, 53° S) | `--at -53.42846 108.35* --height 60 --pitch -42 --time 11` | The same town at another site, turned and on other ground. It is 107° east of home, so 18:10 there: dusk, windows lit, cold haze. |
| `game-4a-holbrook-25m.jpg` | Holbrook (home) | `--at 29.69510 1.36* --height 25 --pitch -6 --time 11` | The home village as it was, unturned. |
| `game-4a-holmouth-low.jpg` | Holmouth (village) | `--at 1.16510 -39.77* --height 20 --pitch -18 --time 11` | The village turned by its site, on a slope at 52 m. 08:15 there. |
| `game-4a-dunmouth-low.jpg` | Dunmouth (village) | `--at 9.62570 56.57* --height 20 --pitch -18 --time 11` | Another turn, by the lane, at 81 m. 14:40 there. |
| `game-4a-holmouth-above-sunset.jpg` | Holmouth | `--height 60 --pitch -42 --time 22.5` | From 60 m as the sun sets there, 19:45. |
| `game-4c-linenleigh-above-night.jpg` | Linenleigh | `--height 60 --pitch -42 --time 22.5` | At night, 23:00: street lamps' pools along the lanes, lit windows, the wall's line. The pools in the fields outside are the glowing flowers. |
| `game-4a-dunmouth-above-night.jpg` | Dunmouth | `--height 60 --pitch -42 --time 22.5` | At night, 02:10: the lane's lamps and lit windows. |

`*` The camera stands 59.5 m east of the town's anchor (60 m at Holbrook),
added to the anchor's longitude, so the town is in front of it. The weather is a clear hour, `--weather-at 7200` by day
and `3600` at night.

**Not in these shots, and not built:** the smithy, well, windmill, crops and
fences, the market stalls, the quay's jetty and boat, furniture and people.
The lake under the walled town is dry ground, and the streets are dirt.

## Slice 4d: the harbour, beside the mockup (2026-10-01)

`harbour-4d-2026-10-01.jpg`: Holinghaven, the towns mockup's harbour
(`tools/mockup_coast_shots.js`) on the left and the game on the right, row by
row: the quay, the fish market, the shipyard, the cog at its pier, the harbour
from above, and the quay and market at 22:30. In-game captures on lavapipe at
1440 x 900 with no GPU, the fast build under `xvfb-run`, a new memory-only
world, `--open-doors --rain 0`; walking runs at the quay, market and shipyard,
`--view column` for the cog (14 m up) and from above (60 m). They show what
things look like, not how smoothly they run.

These were rendered on the build before the cog became a craft, so its ship
here is the town's piece; moored, the craft is the same cut. The quay run
moored 17 boats at Holinghaven (its log). They are in the frame: the flat
teal shapes along the piers on the left are the rowboats and canoes, moored
as the game's Loon (a teal canoe that sits low), and a Tern's sail stands
past the lighthouse. Beside the mockup's wooden rowboats and white sails they
hardly read as boats from the quay.

## Task 4.2b: the harbour's sailboats (2026-10-01)

`harbour-sailboats-2026-10-01.jpg`: Holinghaven's quay at 11:00, the mockup
above the game before and after. Before, every sailing berth held a canoe:
its 0.5 to 1.5 m of water was too shallow for the Tern's fixed 1.7 m keel.
Now the Tern's keel lifts to the water under it (design, task 4.2b), and the
berths along the piers hold Terns with their tall masts and sails. The cog's
stern lantern stands on its aftcastle. Same capture setup as the slice 4d
shots above (lavapipe, no GPU, `--walk --at 23.50609 -115.10515 --yaw -66.9
--pitch -4 --time 18.77`).
