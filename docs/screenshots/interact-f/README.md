# Interact on F: captures

For `openspec/changes/stone-and-wood-ui` (design, "Interact on F, and the
prompt that names it"; tasks section 4). Captured 2026-11-16 on the M4 Max
Mac (its GPU, Metal), fixed-step `--capture` runs of the fast build, each on
a new memory-only world, at 2880x1800, saved as JPEG at quality 88. They show
what things look like, not how smoothly they run; frame cost was not
measured. The machine was shared with other builds during the sitting.

Two capture instruments were added for these shots (design, "Capture
instruments"): `--hold-pointer` makes the walker hold the pointer as a
player's does after a click, so the aim and the prompt show in a still
capture; `--farm-ripe` sows `--farm`'s field ripe, so the crop in front of
the eye is one F harvests. Neither changes anything a player runs.

Common flags: `target/fast/pbd-app --world <new name> --walk --weather-at
1800 --time 10 --hold-pointer`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `controls-hands.jpg` | `--spawn fields --menu settings --settings-tab input --input-tab hands` (no `--hold-pointer`) | The Input page's HANDS tab: the new INTERACT row, F, above the hands' own rows. E is on no on-foot row. |
| `prompt-harvest.jpg` | `--spawn fields --yaw 277 --pitch -24 --farm 2.5 --farm-ripe` | `F  HARVEST WHEAT` under the crosshair, which is on the ripe wheat's leaves. |
| `harvested.jpg` | the same with `--farm-harvest` | After the field's ripe cells are harvested (by the capture's scripted path, the farm's `harvest`, which F calls): bare tilled soil, no prompt, and the note, now below where the prompt stands. |
| `prompt-check.jpg` | `--spawn fields --yaw 277 --pitch -24 --farm 2.5` | `F  CHECK WHEAT` on wheat 80% grown (F says how long it has left). |
| `prompt-pick-wild.jpg` | `--at 23.7034 -0.7026 --yaw 275 --pitch -25` | `F  PICK WILD WHEAT`, the eye on the plant's leaves, not the cell under it. |
| `prompt-pick-fruit.jpg` | `--spawn fields --yaw 200 --pitch -40 --tool shovel` | `F  PICK PLUMS`: a fruiting tree beside the aim outranks tilling (rank 7 above 13). |
| `prompt-till.jpg` | `--spawn fields --yaw 100 --pitch -40 --tool shovel` | `F  TILL GRASS` with the shovel in hand. |
| `prompt-open-forge.jpg` | `--spawn fields --yaw 270 --pitch -30 --forge 2.5` | `F  OPEN FORGE`. |
| `prompt-open-campfire.jpg` | `--spawn fields --yaw 270 --pitch -32 --campfire 2.5` | `F  OPEN CAMPFIRE`, with the kitchen's note below it. |
| `prompt-open-workbench.jpg` | `--spawn fields --yaw 270 --pitch -30 --craft bench-view 2.5` | `F  OPEN WORKBENCH`. |
| `prompt-feed.jpg` | `--husbandry tame --at -37.5 -84 --time 16 --pitch -20 --frames 100` | `F  FEED WOOLBACK`: the tamed woolback following, turf in hand. |
| `prompt-rebound-k.jpg` | as `prompt-pick-wild.jpg`, with `--rebind interact:KeyF=KeyK` | The prompt shows the bound key: `K  PICK WILD WHEAT`. |

Not captured: `F  BOARD ROWBOAT` (no capture scene stands the walker beside a
moored craft; the board priority is pinned by
`vehicles::tests::f_boards_a_craft_only_when_nothing_usable_is_aimed`), and
`F  COLLECT EGGS` (the collect scene's hens stand out of reach of the
camera; the act's offer is the same code as `use_on`'s collect branch).

Checked in the logs: two normal (warming) launches of the final build logged
`READY` and no `LATE_PIPELINE` line: `--world interactf-warm --walk --spawn
fields` for 120 s, and `--world interactf-warm2 --walk --at 23.7034 -0.7026
--yaw 275 --pitch -25 --hold-pointer` for 90 s, with the prompt standing on
screen after `READY`. A capture does not warm, so its logs list pipelines as
late by design.
