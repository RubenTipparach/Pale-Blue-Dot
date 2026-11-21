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

## The loading screen

A normal launch (no `--capture`, lavapipe) beside Holbrook's smith,
`pbd-app --walk --at 29.884943 1.485554 --yaw 17.1 --time 11`, laid
Holbrook's 5, Huatzico's 3 and Linenleigh's 19 townspeople, showed 129
prewarm proxies behind the screen (the townsfolk's figure among them) and
reached `READY` with 125 pipelines 224 s after launch. It ran on for eleven
minutes with the smith drawn a metre off: no `LATE_PIPELINE` and no error.
