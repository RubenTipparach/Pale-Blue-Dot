# Tasks

Step 2e, after `cities-in-the-world`, whose settlements they live in, and
before `player-building`. It exists so that every feature of the approved
towns mockup is on the roadmap (the owner, 2026-09-27).

## 1. Who and where

- [x] 1.1 Roles, posts and paths in each settlement template, ported from the mockup's `person()` calls. Verify: a test that the walled town has 16 people, with a smith and a gate guard among them. (It holds 19: the mockup's four stallholders came after the count; design 19.)
- [x] 1.2 `pbd_core::townsfolk` derives each settlement's people from its record and the seed. Verify: tests that the same record gives the same people twice, and that no post or path crosses a wall.

## 2. How they look and move

- [x] 2.1 The figure of M5 version 9's Faces tab (design 13a, 21; it replaces the hexagonal-prism figure of T2), one material for every person. Verify: a `--capture` a metre from a person showing the block head and the face, in `docs/screenshots/townsfolk/`.
- [ ] 2.2 The walk swing, the work loops (hammer, wipe) and the idle look, as the mockup animates them. Verify: a capture sequence of each. Built (`townsfolk::motion`, `townsfolk::tests::the_body_walks_and_works`); the capture sequences are still owed.

## 3. Solid and lit

- [x] 3.1 Each person's circle in the settlement's moving-solid list, checked by the walker. Verify: an app test that walking straight into a working smith slides round them with no overlapping tick. Done: `walking::tests::walking_into_a_working_smith_slides_round_him`.
- [ ] 3.2 Lighting through the field's sampler. Verify: a capture of the smith at the forge at 22:30. Lit through the field (`LitByField`, `LitLikeTerrain`) and shot at 22:30 (`smith-forge-2230.png`), but open: the game has no forge yet (design 26), so the smith is lit by the lamps and the field.

## 4. The owner's check

- [ ] 4.1 The gate video, with the prototype beside it (`step-videos`): each settlement's people shot by shot, a walk into a crowd, and the forge at night, each beside the same shot in the towns mockup. Verify: the gate page is linked from the PR, with a note that frame cost was not measured in the cloud session.
- [ ] 4.2 The owner watches and accepts. Verify: the quote is in `proposal.md`. Sync `world/townsfolk`, and archive.

## 5. Coins (added 2026-11-21: "doing missions and rtrading should get you money, so we need currency")

- [ ] 5.1 The purse record (`purse`, schema 1) and the one function that changes it, with its reason, refusing a spend it cannot cover (design 5). Verify: `pbd-core` tests that a spend past the purse is refused, that the count survives a save and load, and that every reason is logged.
- [ ] 5.2 The coin count beside the hotbar, its icon and its words from `locale_en.yaml`. Verify: a capture with coins in the purse.
- [ ] 5.3 Values for every good, block, fish and dish in `assets/config/values.ron`, append-only (design 6). Verify: a test that every item the game knows has a value, and that the file only grows.

## 6. Trade

- [ ] 6.1 The trade panel in M5 brought up to the game's stone-and-wood UI, by day and at night; the owner approves it for the game. Verify: on the tracker.
- [ ] 6.2 Each town's trader, its prices from value times need, the 40% spread, stock and the day's drift (design 6, 7), in `pbd-core`. Verify: tests that a harbour sells fish below a mountain town, that selling back what you bought loses coins, that ten sales move the price, and that a day brings it back.
- [ ] 6.3 The trade panel in the game, opened by talking to the trader. Verify: a bot buys a loaf and sells three fish, and the purse and the pack agree.

## 7. Town missions

- [ ] 7.1 M5's job board and talk panel up to date with the job kinds below; the owner approves them for the game. Verify: on the tracker.
- [ ] 7.2 The new objective kinds (`Deliver`, `Escort`, `Repair`, `Sell`, `Buy`, `Light`, `Clear`) counted by the starter missions' function (design 8). Verify: a test per kind from its `Happened` event.
- [ ] 7.3 A town's board made from its needs, its trades and the day's seed, three to five jobs, replaced after three days (design 8). Verify: tests that a harbour's board asks for fish, that the same day gives the same board, and that a taken job never expires.
- [ ] 7.4 The jobs: trade, gather or hunt for the town (herd goods until predators), deliver a parcel, cook a dish, fish, mine, tend a herd, bring in a harvest, light the lamps at dusk. Verify: a bot takes and finishes one of each and is paid.
- [ ] 7.5 The lost villager: a record out of town, a map marker, following at your pace, home within the wall (design 9). Verify: an app test that the villager follows, waits at deep water, and counts at the wall.
- [ ] 7.6 Repair: world-made broken pieces in town buildings, never on a player's piece, mended by placing the material (design 9). Verify: tests that a player's piece is never broken and that placing the block counts.
- [ ] 7.7 Goodwill per town: raised by jobs and trade, better prices and more jobs (design 12). Verify: tests of the price at each goodwill step.

## 8. Roads

- [ ] 8.1 A Roads tab in the towns mockup (M15): sloped road cells, dirt and gravel, a road climbing a hill, a ford and a timber bridge, by day and lit at night; the owner approves it. Verify: on the tracker.
- [ ] 8.2 The sloped road cell: its top, its collision and the walker's ground height on it (design 10). Verify: a test that walking up a road over 10 layers never steps, and a capture.
- [ ] 8.3 Dirt and gravel, built by the player with a shovel or gravel, gravel 15% faster. Verify: tests of the place rule and the speed.
- [ ] 8.4 The road network planned from the sites and the seed, laid once as world records, never on a player's edit, old saves gaining it in place (design 11). Verify: tests that every settlement reaches its two nearest within 2 km, that a road never crosses a player edit, and that a save from before roads opens with them and no restart.
- [ ] 8.5 Missions on roads: clear a rockfall, escort a trader. Verify: a bot does each.

## 9. The owner's check for the additions

- [ ] 9.1 Captures of trade, a board, each special job and a road between two towns, in `docs/screenshots/townsfolk/`; frame cost not measured in a cloud session. Verify: in the PR.
- [ ] 9.2 A gameplay demo: a bot with the goal "earn 100 coins in town", playing jobs and trade through the player's own inputs (CLAUDE.md, Demos). Verify: the video, for the batch.

## 10. Friendly faces (added 2026-11-21: "Design new and friendly looking NPCs ... Think of like kerbal space program")

- [x] 10.1 The research and the plan (design, "Friendly faces", decisions 13 to 18). Verify: in this change.
- [x] 10.2 M5's Faces tab: today's hex figure beside the proposed one; the expression set; blink, gaze and talk running; the generator; a night square lit by its lamps; text in modals; nothing turning on its own. Verify: screenshots in `docs/screenshots/npc-faces/`, the page republished to its link and the tracker updated. Built 2026-11-21 as M5 version 8 (`docs/mockups/npcs.html#faces`, drafted): today's hex figure beside a proposed friendly townsperson (round build, 1.47 m, a 0.45 m six-sided head, 634 triangles with the face), the face as layered pieces 2 mm proud of the head's flat front (eye whites, sliding pupils with catch-lights, sliding lids with a lash line, brow bars, a canvas pixel-art mouth atlas swapped by UV, blush), the ten expressions of design 15, blink, gaze (to the cursor when near), head follow and talk as separate layers with toggles, Play walk (off by default), wave, hop, shrug and tilt, a crowd of eight from a seed, and a time slider with the patch lit at night by two lamp posts, a window and a brazier; the explanation in an info modal. Screenshots in `docs/screenshots/npc-faces/` (SwiftShader). Still to do: the parent session republishes the page and the tracker.
  - Version 9 (2026-11-21, drafted) answers design 13a, the owner's notes on version 8: a bevelled block head 0.5 m wide, 0.6 m tall and 0.45 m deep on every figure (1.55 m in all, the body kept), hair, hats, ears and beards as blocks fitted to it, a face about 1.6 times bigger filling the block's front (the mouth twice as wide), 678 triangles, and a First person button at the game's eye (1.6 m, a 60 degree vertical field of view, 8 m/s, WASD and drag) where nearby townsfolk look up at you. Screenshots `docs/screenshots/npc-faces/v9-*`.
- [x] 10.3 (Approved 2026-11-21, in chat: "People approved. Build.phase 3. Do it in parallel if you can"; version 9.) The owner approves the Faces tab for the game. Verify: on the tracker, recorded in `mockup-work.md`.
- [x] 10.4 The figure, face layers and generator in the game (after 10.3, on a request). Verify: captures by day and at night; a test that the same town gives the same faces. Done 2026-11-21: `townsfolk::tests::the_same_record_gives_the_same_people` and `the_generator_is_the_mockups`; captures by day and at 22:30 in `docs/screenshots/townsfolk/`.
