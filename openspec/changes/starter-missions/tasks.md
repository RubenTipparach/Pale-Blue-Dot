# Tasks

Built last in phase 1, once tool crafting, farming, herding and cooking are in.

## 1. Rules

- [ ] 1.1 `pbd_core::missions`: the chain from data, objective matching, progress, opening by `needs`. Verify: core tests of a chain walked by a list of events, and of a state-met mission completing on open.
- [ ] 1.2 The `missions` record in the save; old worlds start the chain with state-met missions done. Verify: format round-trip; an old save opens with "make tools" done.

## 2. The chain

- [ ] 2.1 `assets/config/missions.ron` with the nineteen missions above, the counts tuned to 1-3 minutes each. Verify: a data test that every `what` names a known good, recipe, block or species.

## 3. In the game

- [ ] 3.0 The HUD line's setting (design 14) and the pending rows' load log (design 8). Verify: a settings round-trip test; a core test that an awaited name makes a mission pending and its dependents open.

- [ ] 3.1 The events fed from crafting, dig and place, fishing, farming, husbandry, cooking and smelting. Verify: an app test per kind.
- [ ] 3.2 The Missions tab and the HUD line, stone-and-wood kit, with a chime on completion. Verify: captures of the tab and the HUD.
- [ ] 3.3 A scripted run (a capture instrument, `--missions`) through the chain from "Go fishing" on a fresh world (the missions before it are pending until `tool-crafting` merges, design 8). Verify: the log shows each completion; screenshots in `docs/screenshots/starter-missions/`.
