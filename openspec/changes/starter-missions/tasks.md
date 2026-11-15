# Tasks

Built last in phase 1, once tool crafting, farming, herding and cooking are in.

Done on `claude/starter-missions` (2026-11-14): tests in `pbd_core::missions` and `pbd_app::missions`, captures in `docs/screenshots/starter-missions/`. The chime in 3.2 waits for an audio engine (design 12); the owner's video waits for the batch.

## 1. Rules

- [x] 1.1 `pbd_core::missions`: the chain from data, objective matching, progress, opening by `needs`. Verify: core tests of a chain walked by a list of events, and of a state-met mission completing on open.
- [x] 1.2 The `missions` record in the save; old worlds start the chain with state-met missions done. Verify: format round-trip; an old save opens with "make tools" done.

## 2. The chain

- [x] 2.1 `assets/config/missions.ron` with the nineteen missions above, the counts tuned to 1-3 minutes each. Verify: a data test that every `what` names a known good, recipe, block or species.

## 3. In the game

- [x] 3.0 The HUD line's setting (design 14) and the pending rows' load log (design 8). Verify: a settings round-trip test; a core test that an awaited name makes a mission pending and its dependents open.

- [x] 3.1 The events fed from crafting, dig and place, fishing, farming, husbandry, cooking and smelting. Verify: an app test per kind.
- [x] 3.2 The Missions tab and the HUD line, stone-and-wood kit, with a chime on completion. Verify: captures of the tab and the HUD.
- [x] 3.3 A scripted run (a capture instrument, `--missions`) through the whole chain on a fresh world, from the first dig to the farm meal (tool crafting merged, design 18). Verify: the log shows each completion; screenshots in `docs/screenshots/starter-missions/`.

## Homestead missions and the bot (design "Missions that show off the game")

- [ ] H.1 The `eat`, `fell` and `night` objective kinds in `pbd_core::missions`, raised where eating, felling and the clock already act. Verify: core tests that each counts once per deed and that an old world's missions record reads unchanged.
- [ ] H.2 The Homestead chain in `missions.ron`, opened by the starter chain. Verify: the load test names no unknown item; the chain is reachable (every `what` has a recipe or a source).
- [ ] H.3 The bot's goals in `assets/showcase/phase-1-loop.ron`: the starter chain, then Homestead, played through the player's inputs (`step-videos` 5.1, 5.2). Verify: a run ends with every mission of both chains done in the save, and `SHOWCASE_END`.

