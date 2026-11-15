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

- [x] H.1 The `eat`, `fell` and `night` objective kinds in `pbd_core::missions`, raised where eating, felling and the clock already act. Verify: core tests that each counts once per deed and that an old world's missions record reads unchanged.
  - `eat` is raised by `kitchen::eat` with the dish's code; `fell` by `take_log` on the axe's last log from a tree; `night` by `missions::night_watch` when the clock passes 20:00 with the player within 30 m of a light in the world's edits. All three are in `Kind::LIVE`, so `fell` is no longer pending. Tests: `eat_fell_and_night_count_once_per_deed`, `an_old_missions_record_reads_unchanged`.
- [x] H.2 The Homestead chain in `missions.ron`, opened by the starter chain. Verify: the load test names no unknown item; the chain is reachable (every `what` has a recipe or a source).
  - Six ids appended after `farm-meal`: `shore-lunch`, `bread`, `flock`, then `bronze` and `lamplight` both opened by `flock` (the bronze age waits on ore, below, and does not hold up the rest), and `feast` after `lamplight`. "Two species" in `shore-lunch` and "three different dishes" in `feast` are written as objectives per dish (a sea, a field and a coop dish) rather than a new "different" rule (recommendation taken, ask only with screenshots). `the_shipped_chain_names_only_what_the_build_knows` resolves all 25 with none pending, and the starter chain done opens `shore-lunch`.
- [ ] H.3 The bot's goals in `assets/showcase/phase-1-loop.ron`: the starter chain, then Homestead, played through the player's inputs (`step-videos` 5.1, 5.2). Verify: a run ends with every mission of both chains done in the save, and `SHOWCASE_END`.
  - **Blocked: no ore in the world.** `smelt` ("dig ore with the pickaxe") cannot be done in a new world: the generator never writes `Material::Ore` (`column::material_at_depth` gives sod, soil, then stone; only the test terrain in `terrain.rs` has ore), so nothing a player can dig yields ore or coal. `smelt`, and with it `metal-tool`, `collect`, `harvest` and `farm-meal`, and so the whole Homestead, are out of reach until ore is in the ground, which is a generation change (a new generator version, written up first). The bot's goal stops at `pen`, the fourteenth starter mission; its last shot says so. The bot plays the rest of the loop the brief asks for (harvest, egg, flatbread, eating) only where a later mission would count it.

