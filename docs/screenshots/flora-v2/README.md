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
| `after-fields-blades.jpg` | `--yaw 270 --pitch -12` | Pixel grass off (the default): the blades as before, now with flower colours, mushrooms, ferns and leaf cards on the trees. |
| `after-fields-tufts.jpg` | `... --grass tufts` | Pixel grass on: the Play demo's tufts in the fields' ramp (design 1). |
| `after-jungle-tufts.jpg`, `after-swamp-tufts.jpg`, `after-beach-tufts.jpg`, `after-tundra-tufts.jpg` | `--spawn <biome> --grass tufts` | The tufts in each biome's ramp; ferns under the canopy; the tundra's pines in their needle cards (no grass on snow). |
| `after-tree-near.jpg` | `--yaw 300 --pitch 20 --grass tufts` | Leaf cards round the hex-prism crowns, hanging from each layer's rim and standing on the top. |
| `after-fields-day27.jpg`, `-day52.jpg`, `-day77.jpg` | `--pitch 5 --grass tufts --day N` | The fields' calendar (design 5): autumn colour, winter (cards thinned to bare, the prism's leaves dulled, straw grass), spring green. |
| `after-menu.jpg` | `--menu settings --settings-tab picture --grass tufts` | The Pixel grass check on the GRAPHICS (PICTURE) page. |

Checked in the logs: no `ERROR` in any capture. A normal (warming) launch,
`PBD_SETTINGS=<scratch> target/fast/pbd-app --world flora-warm-a1 --walk
--spawn fields --grass tufts`, logged `READY playable 5.75 s after launch
(97 pipelines)` and no `LATE_PIPELINE` in the 20 s after it: the change adds
no pipeline.

For the owner to judge: whether Pixel grass should become the default; the
strength of the autumn orange; the winter crown (the prism's leaves stay, as
the cell is the shape you chop, so winter is a dull olive rather than bare
branches).
