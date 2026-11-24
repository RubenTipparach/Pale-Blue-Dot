# Tasks

## 1. Playthrough

- [x] 1.1 Release build of `claude/river-downhill` with `origin/main` merged.
- [x] 1.2 Code audit of every objective (design 1).
- [x] 1.3 Bot: give up a plant F offers nothing for; the status line shows the F offer.
- [x] 1.4 Farming shots: six sown, six harvested; the full bucket picked before the pour.
- [ ] 1.5 Play the remaining branches (herding, cooking, mining, light, about) and record their stalls here.
- [ ] 1.6 (design 5: `phase-1-last`) Shot lists for the missions no showcase plays (bait, three kinds, the sea and fresh water, bronze and iron, a mixed herd, the feast, reef, town, boat).

## 2. Fixes

- [x] 2.1 Coal counts when dug (design 3.1); "Find ore" completed in play on dug coal. Its automated test is still owed.
- [ ] 2.2 The reef re-arms (design 3.2), with its test.
- [x] 2.3 Help lines for `three-crops` and `iron` (design 3.3), and bait's (4.3).

## 3. The recording

- [ ] 3.1 One long video of the bot playing down the tree, on a fresh world, release build, to Drive.

## 4. Found in play (design 4)

- [x] 4.1 One focused mission; the HUD tracker with icons; the tree's focus frame. Core test `the_focus_is_the_one_mission_shown_and_goes_on_down_its_branch`.
- [x] 4.2 No cast under water; sea-level flats are water to the line; "Go fishing" is catch fish only, old records migrate (test).
- [x] 4.3 Bait from any earth, one in five.
- [x] 4.4 The pack's craft bar, QUEUE plaque, four hand jobs.
- [x] 4.5 The bot's route, dig-through, cast aim, mine, free pointer.
- [ ] 4.6 Run the tests and clippy; openspec validate.

## 6. The reef by the map (owner's playtest, 2026-11-22; `docs/plans/2026-11-22-playtest-todo.md`)

- [x] 6.1 `MapGoal(layer, class)`: open the map, pick the layer, find, zoom, mark, close (5.8). Built (bot code, `map_goal`; the base's texels read through `MapRaster::base_texels`); `ToMark` walks in 120 m legs and stops at the coast for a mark at sea. Not yet played: 6.3.
- [x] 6.2 The swim to the rock: the coast, into the water, across, down; dig the reef rock (5.7). Built as `Swim("reef")` (`swim`); the dig is the shot's own `Dig`. Not yet played: 6.3.
- [ ] 6.3 The reef shot played to "mission done: reef"; then the ring search fallback removed.
- [ ] 6.4 The planner's lag: measure the bot's walk (PLAN_TIME, ROUTE_TIME), cut the search cap and the re-plan rate, measure again.
- [ ] 6.5 Measurement logs out before the commit.

## 7. Every starter mission played (owner, 2026-11-23: "covering all missions test in the starter tab")

The showcase files had a shot for 42 of the 44 missions; "Open your pack"
(`pack`, added 2026-11-22) and "A chest" (`storage`) had none.

- [ ] 7.1 `Pack(secs)`: Tab opens the pack, Tab shuts it. A trunk shot after "Gather by hand".
- [ ] 7.2 `Store`: to the nearest chest, F, one stack of the ground's own blocks into it (`make_room`'s store path, for one stack). A trunk shot after the fire: logs, planks, the chest at the bench, set down, stored.
- [ ] 7.3 One fresh world played down every branch in order (trunk, fishing, cooking, farming, herding, mining, light, about, rest, last), the F3 graph on; each stall fixed in the bot and the shot played again from its number.

Stalls on the full run (`tree-run-7`, 2026-11-23), each fixed in the bot and the shot played again:
- Fire: the campfire never set down; the shovel was still in hand and the right button tilled the grass. `place` now puts a held tool away first.
- Rod: two planks short; the new chest shot uses the trunk's spare planks. The rod shot chops a log and makes planks first.
- Go fishing: 16 minutes without a bite at one shore in one launch; the next launch at the same shore caught three minnows in 2 minutes. Not explained: the fish logs were off.
- Three kinds: eight schools spawned in the first minute and none after in 15 minutes. A school stays until the player is 90 m off (`despawn_m`), so a player who stays at one shore sees the same few kinds for good. The bot now moves 100 m along the shore after twelve casts without a bite, not 15 m. Whether a standing angler should see schools come and go is a game question for the owner, not changed here.
- Three kinds, again: the 100 m point along the shore had no route over land; the bot now walks 100 m off with `Away` and searches for the shore again.
- A lag found with the F3 graph on the same shot: a failed surface route cleared the route and its half-second wait with it, so the bot planned every frame (298 plans in 65 s at about 32 ms each; 222 of 978 frames over 50 ms in 20 s, the main thread up to 133 ms). A failed plan now waits a second before the next (`route_failed_at`). The bot's own cost, not the game's.
- The planner's 15 ms budget was checked every 64 expansions; in the jungle by the cove an expansion costs about 1 ms, so a plan ran 66 ms at the median and 119 ms at worst (145 plans in 6 minutes; 163 frames over 50 ms). Checked every 8 now.
- The main thread climbing from 17 to 22 ms over 10 minutes of the stuck shot is the bot's, not the game's: the same world opened with no bot and stood in for 240 s holds at 10.4 - 10.7 ms the whole time (`--world tree-run-7 --stand 240`). The extra 10 ms a frame is the bot stuck in a pit, trying "up and out" every frame.
- The three-kinds shot is left at 2 of 3 for now: the cove it starts at is walled in and full of the bot's bait pits. Farming, herding, cooking, mining, light and about are played first; three-kinds is played again from wherever the about branch leaves the player, before rest and last.
- Farming, "Break ground": the bot dug down for seeds into stone and ore and was held in its pit; "up and out" dug at the lowest diggable block over its head up to 12 layers up, past an open gap and out of reach, for minutes (owner, watching: "bot stuck digging blocks it cant reach"). It now digs over the head only within 4 layers of the feet, and builds up otherwise.
- The forge: made, never set down. No fresh level cell 2.6 to 4.2 m off by the bench, which stands in the bot's own pit; the shovel taken back up on the roam tilled instead. Now 2.2 to 6 m, a step up allowed, 90 s, and a tool put away before the right button.
- The game stalled 74 s by the forge (the owner killed it): a cell plan of 0 steps cleared the plan and its half-second wait, and the bot planned every frame. A failed or empty cell plan now waits a second, as the surface route does.
- The iron age: the forge carried but "the selected slot holds no block": climbing out had selected and spent a dirt stack. `place` checks the slot holds the item before the right button and selects it again.
- The torches: two of four set, then held in a pit by the base for 8 minutes. "Up and out" gave up every frame when the lowest diggable block over the head was out of reach and "never dug" (the reach rule of the farming stall left the never-dug test on an out-of-reach block); it now builds up instead. The torch shot also walks 80 m off first, away from the base's pits.
- In and out of water: swimming out after 20 s with wet feet or below the sea's level over sea floor (the sea is not in the columns), to the nearest shore, a quarter turn when 15 s bring it no nearer; ahead of "up and out", which held the walk at a wall.
- A full pack and hotbar: junk dropped off the panels (pick up, a `Clear` click letting go every press left standing, a click on the background), in `make_room` and `select`; and a `Declutter` act at the start of the last branch's digging shots.
- Where the run stands (2026-11-23, 16:20): 36 of 44 done on `tree-run-7`. Open: sea-fishing's fresh half, three-crops, farm-meal, feast, reef, town, boat. The bot is held 10 m down a flooded shaft by the shore, of its own digging over many retries: it swims up to the water's top, and the shaft's walls go on above it, where a block under the feet would lift it but none is set in water.
