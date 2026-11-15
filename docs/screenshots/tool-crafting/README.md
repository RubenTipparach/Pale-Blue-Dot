# Tool crafting: captures

For `openspec/changes/tool-crafting`. Captured 2026-11-14 on the M4 Max Mac
(its GPU, Metal), fixed-step `--capture` runs of the fast build, each on a
new world of its own name, at 2880x1800, saved as JPEG at quality 88. They
show what things look like, not how smoothly they run; frame cost was not
measured. Videos wait for the owner's batch (CLAUDE.md, 2026-09-27).

A world made by this build owns no tools, so the capture flags give the
capture what it needs through the same paths a player's hands take:

- `--tool hand|rod|shovel|pickaxe|axe` puts it in hand (the rig gives the
  tool first, at `--tier stone|bronze|iron|steel`, stone when none is
  named); `--tool hand` is bare hands. With `--break SECONDS` the use button
  is held on what is under the reticle from frame 90.
- `--craft pack` stocks the slots, queues planks by hand 0.4 s into its
  1 s and opens the pack on the workbench recipe.
- `--craft bench [metres]` places a workbench that far ahead, stocks the
  slots, queues a stone pickaxe half made and opens its panel with the iron
  pickaxe picked; `--craft bench-view [metres]` only places the workbench.

Common flags: `target/fast/pbd-app --world <new name> --walk --spawn fields
--weather-at 1800 --time 10`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `punch-dirt-by-hand.jpg` | `--tool hand --break 10 --pitch -40` | Bare hands on the meadow (the hand in the tool slot): the grass block cracking about half way through its 2.0 s, four times the stone shovel's 0.5 s. |
| `dug-by-hand.jpg` | the same with `--frames 700` | Three blocks dug by hand, each logged at 2.00 s: grass, then the soil under it and its neighbour, floating out to be picked up. |
| `pack-craft-row.jpg` | `--pitch -10 --craft pack` | The pack at M29's 268 x 204 with its Craft row: the drop-down on WORKBENCH, the six planks it takes lit (carried), the workbench as the result, and the hands' plaque counting the planks job up (40%). The pack holds a log, fibre, planks and loose stone; the fences are husbandry's v4 kit grant. |
| `workbench-panel.jpg` | `--pitch -25 --craft bench 6` | A workbench (a block of the local trees' wood, a plank top and an iron vise) and its panel mid-craft: the eleven recipes as wells (stone, bronze and iron picks, shovels and axes, the rod, the forge), the iron pickaxe picked with its three iron ingots and two planks lit, and the stone pick half made in the bar. |
| `stone-pick-in-hand.jpg` | `--pitch -20 --craft bench-view 6 --tool pickaxe --tier stone` | A stone pickaxe in hand at the workbench: its head drawn in the stone tier's dark grey, the stone pick's icon in the tool slot. |
| `iron-pick-in-hand.jpg` | `--pitch -40 --tool pickaxe --tier iron --break 10 --frames 300` | An iron pickaxe (the head as drawn, bright) after digging: each block logged at 0.62 s, the stone pick's 1.5 s over iron's 2.4. |

Checked in the logs: every scripted queue and placement landed, and a normal
(warming) launch of the same build logged `READY` with no `LATE_PIPELINE`
line (`--world tc-warm-1 --walk --spawn fields`, 45 s). A capture does not
warm, so its logs list pipelines as late by design.

Not shown: a tree giving its branch (the capture rig cannot aim the reticle
at a trunk reliably); the core and app tests cover the rule and the save.
