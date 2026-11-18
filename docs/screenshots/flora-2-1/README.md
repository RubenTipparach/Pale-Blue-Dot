# Flora v2, roadmap step 2.1: captures

For `openspec/changes/flora-v2` (tasks 7.8, 9.5, 9.6; design 12.4, 12.5,
12.11). Taken 2026-11-18 in a Claude Code cloud container: headless
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

The picked trees are marked by the `--pick-trees` instrument (records as a
player's hands would write them), not by a pick played with F: a still is
what this shows, and a played pick belongs to the gameplay demo.

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
