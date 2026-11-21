# Tasks

Step 2e, after `cities-in-the-world`, whose settlements they live in, and
before `player-building`. It exists so that every feature of the approved
towns mockup is on the roadmap (the owner, 2026-09-27).

## 1. Who and where

- [ ] 1.1 Roles, posts and paths in each settlement template, ported from the mockup's `person()` calls. Verify: a test that the walled town has 16 people, with a smith and a gate guard among them.
- [ ] 1.2 `pbd_core::townsfolk` derives each settlement's people from its record and the seed. Verify: tests that the same record gives the same people twice, and that no post or path crosses a wall.

## 2. How they look and move

- [ ] 2.1 The hexagonal-prism figure in the terrain's pixel style (the owner, T2), instanced. Verify: a `--capture` a metre from a person showing six-sided parts, in `docs/screenshots/townsfolk/`.
- [ ] 2.2 The walk swing, the work loops (hammer, wipe) and the idle look, as the mockup animates them. Verify: a capture sequence of each.

## 3. Solid and lit

- [ ] 3.1 Each person's circle in the settlement's moving-solid list, checked by the walker. Verify: an app test that walking straight into a working smith slides round them with no overlapping tick.
- [ ] 3.2 Lighting through the field's sampler. Verify: a capture of the smith at the forge at 22:30.

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
