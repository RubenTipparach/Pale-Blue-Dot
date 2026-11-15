# Flora v2: captures

For `openspec/changes/flora-v2`. Captured 2026-11-15 on the M4 Max Mac (its
GPU, Metal), fixed-step `--capture` runs of the fast build, each on a new
world deleted afterwards, at 2880x1800, saved as JPEG (before: full size;
after: 1920 wide). They show what things look like, not how smoothly they
run; frame cost was not measured.

Common flags: `target/fast/pbd-app --walk --weather-at 1800 --time 10
--world <new> --spawn <biome>`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `before-fields.jpg`, `before-jungle.jpg`, `before-swamp.jpg` | `--yaw 270 --pitch -12` | The build this branch started from (`claude/phase-1`, 6632239). |
| `after-fields-blades.jpg` | `--yaw 270 --pitch -12` | Pixel grass off (the default): the blades as before, now with flower colours, mushrooms and ferns. |
| `after-fields-tufts.jpg` | `... --grass tufts` | Pixel grass on: the Play demo's tufts in the fields' ramp (design 1). |
| `after-jungle-tufts.jpg`, `after-swamp-tufts.jpg`, `after-beach-tufts.jpg`, `after-tundra-tufts.jpg` | `--spawn <biome> --grass tufts` | The tufts in each biome's ramp; ferns under the canopy; no grass on the tundra's snow. |
| `after-tree-near.jpg` | `--yaw 300 --pitch 20 --grass tufts` | The hex-prism trees as today: the leaf cards were taken off at the owner's word (below). |
| `after-fields-day27.jpg`, `-day52.jpg`, `-day77.jpg` | `--pitch 5 --grass tufts --day N` | The fields' calendar (design 5): autumn colour on the prism leaves, winter (the leaves dulled toward twigs, straw grass), spring green. |
| `after-menu.jpg` | `--menu settings --settings-tab picture --grass tufts` | The Pixel grass check on the GRAPHICS (PICTURE) page. |

Checked in the logs: no `ERROR` in any capture. A normal (warming) launch,
`PBD_SETTINGS=<scratch> target/fast/pbd-app --world flora-warm-a1 --walk
--spawn fields --grass tufts`, logged `READY playable 5.75 s after launch
(97 pipelines)` and no `LATE_PIPELINE` in the 20 s after it: the change adds
no pipeline.

**Leaf cards removed (owner, 2026-11-15):** "Hmm trees have weird leave
structures on them I don't really like". Every after shot was retaken without
them on the same day; a second warming launch logged `READY playable 5.76 s
(97 pipelines)`, no `LATE_PIPELINE`, no `ERROR`.

For the owner to judge: whether Pixel grass should become the default; the
strength of the autumn orange; the winter crown (the prism's leaves stay, so
winter is a dull olive rather than bare branches).
