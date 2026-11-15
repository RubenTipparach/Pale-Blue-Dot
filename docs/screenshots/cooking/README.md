# Cooking: captures

For `openspec/changes/cooking` (design S1 to S8, tasks 9). Captured
2026-11-14 on the M4 Max Mac (its GPU, Metal), fixed-step `--capture` runs of
the fast build, each on a new memory-only world, at 2880x1800, saved as JPEG
at quality 88. They show what things look like, not how smoothly they run;
frame cost was not measured. Videos wait for the owner's batch (CLAUDE.md,
2026-09-27).

`--campfire [metres]` is the slice's capture instrument (task 9.5): it places
a campfire that far ahead through the same durable edit path a player's
right button takes, gives two fish of one species, one of another, six wheat
and two chili, puts flatbread on the fire 60 world seconds ago (done: it takes
30) and a fish 15 seconds ago (three-quarters of its 20), each through the
same queue the panel uses. With `--campfire-panel` the panel opens, and the
panel collects the flatbread; without it the flatbread is collected at once.
`--eat <dish,dish>` puts each dish in the selected slot and eats it through
the same path as the right button.

Common flags: `target/fast/pbd-app --world <new name> --walk --spawn fields
--weather-at 1800 --yaw 270 --pitch -28`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `campfire-day.jpg` | `--world cook-day-b1 --time 10 --campfire 4` | A campfire at 10:00: the ring of stones and the fire on it, drawn by the column pass as the seventh light. |
| `campfire-night.jpg` | `--world cook-night-b1 --time 22 --campfire 4` | The same at 22:00 in the rain: the fire lights the meadow round it (a light of level 13, a street lantern's reach), the moon the rest (lit at night). The note is the flatbread collected. |
| `campfire-night-crop.jpg` | | Part of `campfire-night.jpg`, enlarged two times (nearest). |
| `campfire-panel.jpg` | `--world cook-panel-b1 --time 10 --campfire 4 --campfire-panel` | A fish cooking: the campfire's panel with its six dishes (grill fish and chili fish take any fish, drawn as the fish held; fried egg takes husbandry's egg), the fish three-quarters grilled on the bar, and the line that says how a dish is eaten. Plaques whose inputs the slots lack are greyed. Roast meat is pending (no raw meat in this build) and not listed. |
| `flatbread-pack.jpg` | `--world cook-pack-b1 --time 10 --campfire 4 --menu pack` | Bread from wheat: the pack after the flatbread was taken off the fire, two flatbread from three wheat (six given, three left), one fish gone to the grill, and the kit's campfire (kit v6). |
| `eaten.jpg` | `--world cook-eat-b1 --time 10 --eat flatbread,grilled-fish,roast-turnip` | Eating with the effects shown: three dishes eaten, the effects row over the hotbar (steady hands 6:00, stride 5:00, keen 4:00, each with the dish's icon), and the note for the last one. |

Checked in the logs: every scripted queue, collect and meal landed; the
recipe `roast-meat` is logged as pending. A normal (warming) launch of the
same build logged `READY` with no `LATE_PIPELINE` line (`--world
warm-cook-c1 --walk --spawn fields`, 50 s). A capture does not warm, so its
logs list pipelines as late by design.

Known limits of the look: the campfire is the lamp draw's two boxes (a stone
ring and the fire), so the fire reads as a glowing block rather than flames
and logs; a model of its own (crossed flame cards, logs) is a later pass.
