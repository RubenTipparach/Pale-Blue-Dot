# Flora v2, roadmap step 2.1: captures

For `openspec/changes/flora-v2` (tasks 7.8, 9.5, 9.6, 12.13; design 12.4,
12.5, 12.11, 12.13). Taken 2026-11-18 in a Claude Code cloud container: headless
`--capture` runs of the fast build on lavapipe (software Vulkan, no GPU),
`xvfb-run -a -s "-screen 0 1920x1200x24"` with `PBD_WINDOW=1920x1200`, each
on a new world. They show what things look like, not how smoothly they run:
the frame cost was not measured. The debug readout (the top 120 px) is
cropped off every shot. No `ERROR` in any capture's log.

Common flags: `target/fast/pbd-app --walk --world <new> --time 10
--weather-at 1800`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `before-jungle.jpg`, `after-jungle.jpg` | `--at 37.995 15.633 --yaw 270 --pitch 12 --frames 120` | Vines from the ground in the jungle (task 9.5, design 12.4), the same view and time. Before (the branch's start, `6de31719`): a few thin strands under the canopy, right of centre, in the crown's own colour. After: crossed 0.5 m strands with a stem and broad leaves, a little yellower and lighter than the kapoks' crowns, hanging from the limbs down past the crowns to about head height. No outline; lit like the leaves. |
| `jungle-compare.jpg` | | The two above, before over after, at half size. |
| `beach-acacia-spot-now.jpg` | `--at 44.333 -15.791 --frames 120` | The spot where `flora-v2/v10-beach-acacias.jpg` caught version 10's acacia on beach sand on 2026-11-15. Today the sand there is bare (task 7.8). |
| `beach-acacia-then-now.jpg` | | Then (2026-11-15, the old shot) over now. The acacia stood on sand 1.0 m up, in the tide; 12.11 ("nothing grows where the sea or the tide stands") floods every beach cell, because every beach top is at 0 or 1 m and the high water is at 1.5 m. No beach acacia stands anywhere (the instrument below), so none can be captured. Bringing it back is a look change for the owner (design 12.11, "Measured 2026-11-18"). |
| `fruit-before-pick.jpg`, `fruit-after-pick.jpg` | `--spawn fields --day 20 --yaw 270 --pitch 25 --frames 150`; the after adds `--pick-trees` | Fruit picking (task 9.6, design 12.5): an autumn fields grove looking up into the nearest tree's crown. Before: plums hang in the crown, centre and upper left. After: the instrument marks the three nearest fruiting trees ahead (3, 6 and 11 m) picked today, as `fruit` records, and that crown's fruit is gone. A pixel difference of the two frames finds nothing else changed. |
| `fruit-pick-zoom.jpg` | | The crown round that plum at 2x, before (left) and after (right). |
| `fruit-pick-played-before.jpg`, `fruit-pick-played-after.jpg` | `--at 37.995 15.633 --day 20 --pick-fruit 20 --frames 260`; the before with a press frame past the run's end | A pick played with F (task 9.6). `--pick-fruit` waits for the fine set under the walker, chooses the nearest hanging fruit (a papaya 6.1 m off), turns the look onto it and walks with W until it is in reach (4.0 m), then presses F on the 20th frame there, as `break_script` presses the mouse. `fruit::pick` does the rest as for a player: the log reads `picked 4 papaya from the tree at cell 1325536530 on day 20.42`. Before: the papaya under the crosshair and the `F PICK PAPAYAS` prompt. After: the fruit gone, no prompt, `PAPAYA X4` in the hotbar's first slot and the item toast bottom right. |
| `fruit-pick-played-zoom.jpg` | | The crosshair at 2x, before (left) and after (right). |
| `v12-beach-acacias.jpg` | `--at -13.948 87.081 --yaw 78.2 --pitch 5 --frames 150`, no `--world` (a memory-only world on the newest generator, version 12) | Acacias behind a beach by day (task 12.13, design 12.13, survey F1). The walker stands on the beach's sand at eye level, looking inland: the acacias stand on the dry grass just behind the beach (right), above the high water; the sand under the high water is bare. The spot was found by a throwaway instrument over the version 12 records (warm beach spots, the most acacias on back cells within 40 m, a beach cell about 18 m off to stand on). |

`fruit-{before,after}-pick.jpg` mark the picked trees with the
`--pick-trees` instrument (records as a player's hands would write them);
`fruit-pick-played-*.jpg` play the pick with the F key.

## The instruments

- `tree_kit::tests::report_the_vines` (`cargo test -p pbd-core --lib
  report_the_vines -- --nocapture`): strands a tree, card width, longest
  strand and lowest vine leaf, per species. Before: kapok 4.25 strands,
  0.16 m cards, the lowest leaf 4.9 m up; after: 6.75 strands, 0.5 m crossed
  cards, 2.4 m.
- `planet::lod::flooded_ground::where_a_beach_acacia_stands` (ignored; `cargo
  test -p pbd-app --lib where_a_beach_acacia_stands -- --ignored
  --nocapture`): swept 1,527 beach spots; no dry beach cell, the highest beach
  cell 1.00 m against the high water's 1.50 m.
- `--take-trees` / `--pick-trees` log `TAKE_TREES` with the distances it
  chose at and how far the nearest record lies from the eye. Before the fix
  that was 23.4 m (a set laid before the walker came to rest) and it chose
  nothing nearer than 26 m; now 2.3 m, and it picks at 3, 6 and 11 m.
