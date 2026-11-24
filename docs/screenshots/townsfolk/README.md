# townsfolk: the townspeople in the game (tasks 2.1, 3.2, 10.4)

The owner, 2026-11-21: "People approved. Build.phase 3". These are the first
frames of M5 version 9's Faces-tab figures in the game
(https://claude.ai/artifact/E2U5akkQgd5cdzPAXjrEyx#faces): the bevelled block
head, the layered face, the generated looks, standing at their posts and
walking their paths in Holbrook and Linenleigh.

Fixed-step captures in the Claude Code cloud container: **lavapipe, no GPU**,
`--profile fast`, memory-only worlds, a 960 x 600 window, 150 frames in,
`--rain 0 --weather-at 7200`, saved as JPEG (quality 90) from the PNGs the script writes. They show the look only: **frame time was not
measured**. `tools/capture_townsfolk.sh` takes them all; the spots come from
`cargo test --profile fast -p pbd-app --lib -- --ignored
print_where_to_stand_for_the_townsfolk_shots --nocapture`.
`PBD_TOWNSFOLK_EXPRESSION=<id>` (a capture instrument) holds every face.

| Shot | Flags | What it shows |
| --- | --- | --- |
| `smith-face.jpg` | `--walk --at 29.884943 1.485554 --yaw 17.1 --pitch -14 --time 11` | Holbrook's smith from a metre in front of him: the block head, the crop and the beard as blocks, the button nose, the eyes with their catch-lights looking up at the player, the blush and the smile of a meeting (surprised for half a second, then happy). |
| `smith-forge-2230.jpg` | `--walk --at 29.860050 1.502882 --yaw -39.2 --pitch -8 --time 22.5` | The smith at 22:30, lit by the lamp post beside him and the field, as the ground under him is. **Not the shot asked for**: the game has no forge yet (design 26), so nothing lights his forge side. |
| `market-day.jpg` | `--walk --at 29.702875 9.150705 --yaw 24.2 --pitch -6 --time 11` | Linenleigh's market street by day: a stallholder at her post looking at the player, two walkers down the lane, and a townswoman in red with her braid, each with their own build, head, hair and clothes from the seed. |
| `face-<expression>.jpg` | `PBD_TOWNSFOLK_EXPRESSION=<expression>`, as `smith-face` | The smith held in surprised, sleepy, delighted, worried, cross and thinking. |

Ashenham, the first walled town by id, stands in water on today's ground
(its lanes are flooded to its people's waists); the market shot was moved to
Linenleigh. That is the towns' ground, not the people.

## Ears and soft shading (design 38 and 36a, 2026-11-22)

Retaken with the same flags (`tools/capture_townsfolk.sh`, `ONLY=ears` and
`ONLY=beards`; lavapipe, `--profile fast`, frame time not measured). Each ear
is one tapered, cupped piece grown from the head's side (it was a box), and
the beard and the ears are soft shaded (the owner: "soft shade the beard, not
flat shade").

| Shot | What it shows |
| --- | --- |
| `ears-smith-side.jpg`, `ears-smith-side-close.jpg` | Holbrook's smith three-quarter on (the `smith-forge-2230` spot at 11:00), and a 3x nearest-neighbour crop of his head: the right ear flares out of the side face and rolls round to its rim, no hard corner where it meets the head; his goatee shaded smoothly. |
| `ears-smith-side-2230.jpg`, `ears-smith-side-2230-close.jpg` | The same at 22:30, by the lamp post. |
| `ears-smith-front.jpg` | The smith face on from a metre: at that distance the ears stay inside the head's silhouette. |
| `beard-goatee.jpg`, `beard-goatee-talking.jpg`, `beard-full-front.jpg`, `beard-full-front-2230.jpg` | The beard shots retaken soft shaded: no crumpled facets; the innkeeper's ears show either side of his bald head. |

## The loading screen

A normal launch (no `--capture`, lavapipe) beside Holbrook's smith,
`pbd-app --walk --at 29.884943 1.485554 --yaw 17.1 --time 11`, laid
Holbrook's 5, Huatzico's 3 and Linenleigh's 19 townspeople, showed 129
prewarm proxies behind the screen (the townsfolk's figure among them) and
reached `READY` with 125 pipelines 224 s after launch. It ran on for eleven
minutes with the smith drawn a metre off: no `LATE_PIPELINE` and no error.

## The motion as capture sequences (task 2.2, 2026-11-24)

`ONLY=motion tools/capture_townsfolk.sh` (lavapipe; the look only): every
4th frame of 48 from frame 150 (`PBD_CAPTURE_RUN`), each cropped to the
person and laid out by `tools/crop_sequence.py`. The idle look (taken
2026-11-24) is shot at Linenleigh's merchant idling at his post, from the
talk captures' `STALL_FAR` spot: the gate guard's and the bearded
stallholder's old spots showed a door and a stallholder's back since the
towns were laid again.

| Sheet | What it shows |
| --- | --- |
| `motion-hammer-sheet.jpg` | Holbrook's smith at his post: the arm raised and brought down, then at rest. |
| `motion-wipe-sheet.jpg` | Linenleigh's innkeeper: the hand circling as it wipes. |
| `motion-walk-sheet.jpg` | Two of Linenleigh's people walking their paths: arms and legs swinging opposite. |
| `motion-idle-sheet.jpg` | Linenleigh's merchant idling at his post, the player 2.7 m off: his eyes move in saccades and settle on the player, the head sways a little, and he blinks (the last two frames). |

## The road planner (`townsfolk` 14.3 to 14.7, design 42 to 42l)

Fixed-step captures by `tools/capture_road_planner.sh` in the Claude Code
cloud container: **lavapipe, no GPU**, `--profile fast`, new memory-only
worlds at the `fields` spawn, 1440 x 900, 400 frames, JPEG. They show the
look only: **frame time was not measured**. `--paving 5 --road-plan <scene>`
is the capture instrument: the grinder and the mixer either side, then
roads put in through the paths the planner's clicks take
(`roads::paving::end_road`, `erase_road`), stakes in hand. There is no
BEFORE: the game had no road planner (design 39's stake search is retired).

| Shot | What it shows |
| --- | --- |
| `road-planner-two-1100.jpg` | A road by two clicks up the view, its cells under the lime band and a stake at its near end; an open start a few metres ahead, so the ghost (the gravel texture in the ghost tint, lit as the ground) runs from it to the aim; the HUD line with the bill. |
| `road-planner-stake-1100.jpg` | A second road started from the first road's middle (a new stake there; the far peg on the right is its end), and the idle ghost: the one cell a click would start on. The bill counts both roads (22 cells). |
| `road-planner-erased-1100.jpg` | The same after the eraser took the second road: its peg gone, its stakes back in hand (14), the bill down to the first road's 16 cells. |
| `road-planner-wide-1100.jpg` | A three-wide road open from 34 m up the view: its ghost back to the aim, lanes red where the ground is off the centre's height ("Too steep across"). |
| `mark-close-1100.jpg`, `mark-close-2230.jpg` | The default paving scene close up (task 13.4, design 39a): a road by two clicks, its six nearer cells paved in gravel, a peg of split timber with its dyed cloth at the start, the lime band on the unpaved cells beyond; by day, and at 22:30 as dark as the ground beside it. |

The bill line is red where the pack is short: each scene gives 12 gravel,
and its roads want more. The ghost at its shipped `ghost_opacity` (0.35)
reads faintly over the desert sand in these frames; it is a tunable in
`road_planner.ron`. No shot at 22:30 of the planner scenes was taken (a
shot is about half an hour on lavapipe here).

