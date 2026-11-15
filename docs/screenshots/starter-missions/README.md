# Starter missions: captures

For `openspec/changes/starter-missions` (design 7-17, tasks 3.2 and 3.3).
Captured 2026-11-14 on the M4 Max Mac (its GPU, Metal), fixed-step
`--capture` runs of the fast build, each on a new world, at 2880x1800, saved
as JPEG at quality 88. They show what things look like, not how smoothly
they run; frame cost was not measured. Videos wait for the owner's batch
(CLAUDE.md, 2026-09-27).

`--missions <n>` is the capture instrument (design 17): once the world is
open it raises the first `n` steps of a scripted chain, one every two
frames, as the messages the game's systems raise (`Gained`, `Planted`,
`Deed`, `Cooked`, `missions::Did`). The counting, the record, the HUD line
and the tab are the game's own; the script plays no digging, fishing or
farming itself, so the hotbar and the world do not change.

Common flags: `target/fast/pbd-app --walk --spawn fields --weather-at 1800
--yaw 270 --pitch -10 --time 10`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `hud-catch-fish.jpg` | `--world missions-hud-a2 --missions 22 --frames 420` | The HUD line mid-mission, top right under the compass strip: `CATCH FISH 1/3` (one fish of three landed). |
| `done-go-fishing.jpg` | `--world missions-hud-a1 --missions 22` | A completion: `DONE: GO FISHING` in the rune colour, held four seconds before the next mission's line. (Captured at frame 180, while the notice for the mission closed at step 21 is still up.) |
| `tab-mid-chain.jpg` | `--world missions-tab-a1 --missions 23 --menu missions --frames 120` | The Missions tab (O), with the PACK / MISSIONS tabs over it: eight missions done, "Catch fish" current (NOW) with its count `FISH 2/3`, the ten after it locked, and the current mission's help at the foot. |
| `tab-chain-done.jpg` | `--world missions-chain-a1 --missions 99 --frames 160 --menu missions` | The whole chain walked from a fresh world's first dig to the farm meal: all nineteen done. |

Checked in the logs: the run to the end logged `mission done:` nineteen
times, in the chain's order (gather, planks-rope, crafting-table,
mining-tools, axe, rod, fire, go-fishing, catch-fish, cook-fish, find-seeds,
plant, tame, pen, smelt, metal-tool, collect, harvest, farm-meal), and no
mission is logged as pending in this build. A capture does not warm, so its
logs list pipelines as late by design; a warming launch was not repeated for
this change, which adds only UI nodes and kit text.

Known limits: the HUD line is the kit's font with its shadow, on no plaque;
over bright snow or sky it may want a backing strip, a later look pass.
There is no chime on completion: the game has no audio engine yet.
