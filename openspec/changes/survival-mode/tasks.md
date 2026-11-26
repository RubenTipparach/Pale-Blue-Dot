# Tasks

Phase 4 (moved from 3c, owner, 2026-11-22), after `multiplayer` and `game-launcher` (3b).
Started on the owner's word, 2026-11-26 ("start working on phase 4, we're
gonna work on survival mode"): the write-up and the mockups first; game code
waits on their approval and a request to build. Forest fires and torches are
`wildfire`'s tasks.

## 1. Mockups for the owner

- [ ] 1.1 The survival HUD (design 9, 12 to 14): health, stamina, hunger, water, body temperature and breath stacked at the left edge (owner, 2026-11-26, survey G7: "a vertical stack at the left edge"), a full bar hidden and hunger and water shown only at 50% or lower ("Don't need to show food or water till it's 50% or lower. You can check in pack these stats at any time..if stats are full hide them."), the pack's stats panel, winded, heat stroke's warning above the hotbar (G8: "Just show stroke warning above hot bar") and passing out, a tool's wear bar and a broken tool's icon (design 11), the hit edge and "You died", Explorer beside Survival, by day and lit at night. Version 1 drafted and published 2026-11-26; version 2 (G7 and G8) drafted 2026-11-26 in `docs/mockups/survival.html#hud`, screenshots `docs/screenshots/survival-mode/mockup/hud-*.png`. Verify: the owner approves it.
- [ ] 1.3 The stamina and food curve (design 12) on `survival.html#stamina`: sliders on every number of `survival.ron`'s `stamina` and `food`, a live graph of stamina, strain and food over the three worked examples, the same model as `tools/survival/stamina_model.py`. Drafted 2026-11-26. The owner (G6, 2026-11-26) kept the numbers and asked: "Some sort of js animation would be nice to see this in action. Add an expand button to lazy load": version 2 adds, behind Expand, a side view of a player sprinting, swimming, jumping, climbing and resting with the stamina and food bars moving live by the model, still until Play (screenshots `stamina-anim-*.png`). Verify: the owner approves the curve or moves its sliders; the numbers they settle go into design 12 and 20.
- [x] 1.4 The weapons (design 16) on `survival.html#weapons`: the stone spear, the bow and arrows, the bronze hunting axe and the bronze sword, textured and smooth-shaded at true scale beside a 1.8 m figure on the hex grid, by day and lit at night by a campfire. Drafted 2026-11-26. Approved as drawn by the owner on 2026-11-26 (survey G10), for the animation step. Next: the held models and swings through the `animation` skill (M36) before the game (task 1.6).
- [ ] 1.6 The held weapons and their swings (design 16) by the `animation` skill, mockup first beside today's motion. Verify: the owner approves it.
- [ ] 1.5 The Survival branch (design 18) as M33's Survival tab (`docs/mockups/mission-tree.html#survival`), with stand-in icons from `tools/mockup_survival_icons.py`. Drafted 2026-11-26. Verify: the owner approves it. Its hanging point approved by the owner on 2026-11-26 (G11: "Off Open the pack", "Go right into survival after pack is opened (if survival mode is active)"); the tab's bot run now goes to "Drink water" right after the pack.
- [ ] 1.2 A predator hunt (design 6): notice, stalk, warn, attack and flee, the player's swing as the hit, by the `animation` skill, by day and lit at night. Verify: the owner approves it.

## 2. The mode

- [ ] 2.1 The world's mode in its identity record, Explorer for every older save; the choice on the new-world screen and in the world list. Verify: a test that an old save opens as Explorer; a Survival world reopens as Survival.

## 3. Rules in the core

- [ ] 3.1 `pbd_core::survival`: health, hunger, breath and suit air as pure functions of state, play time and events; `assets/config/survival.ron` with units, validated. Verify: unit tests for refill and drain rates, and that time with the game closed changes nothing.
- [ ] 3.2 The fall rule (design 3) from impact speed along the body's up, with each body's gravity and water cancelling. Verify: tests for a two-layer step, a twenty-layer fall, the same fall on a half-gravity body, and a fall into deep water.
- [ ] 3.3 Meals fill hunger (`fills` per dish, scaled by stars). Verify: a test per star level.
- [ ] 3.5 Stamina (design 12): spend by action, the refill's food cost by bar height, strain at 1.5, food setting the rate, winded. Verify: unit tests reproducing the design's worked examples to 0.05 fp (a minute's sprint 6.28, a 200 m swim 4.77, a day 112.9), that the bottom half costs less than the top, that tool work spends nothing, and Explorer has no bar.
- [ ] 3.6 Water and body temperature (design 13 to 15): `feel` from an `atmosphere::Sample`, the shade march on the column tier, the body's approach, water's rates, the drink table, the cactus record. Verify: tests for the design's desert and tundra examples, shade cooler than sun, a cloak and a fire each warmer, sea water giving nothing, a cactus's sips and its regrowth across a reload.
- [ ] 3.7 Heat stroke and passing out (design 14). Verify: tests that an empty water bar in the desert sun reaches 40 C and passes out, a watered one never does, a hit wakes, a stroke in deep water floats.
- [ ] 3.8 Weapons and hunting (design 16, 17): the damage table, a thrown spear and an arrow's flight under the body's gravity in the floating origin (on `tests/origin_guard.rs`), kills, drops, dressing, the `hunted` record and the herd's return, protected and kept animals refused. Verify: tests for each, an arrow's drop matching `g` on two bodies, a woolback's drops, a herd member back after 3 days and not before a reload.
- [ ] 3.4 Tool wear (design 11): uses by tier in `survival.ron`, one use per finished action and two for a harder block, broken at zero, repair costs. Verify: unit tests for each action, a missed swing costing nothing, a tool breaking and being repaired, and Explorer never wearing.

## 4. In the game

- [ ] 4.1 The server ticks survival for each player in Survival worlds; the player record gains health, hunger, breath and spawn. Verify: an app test that a long dive drains health and surfacing refills breath; a `--capture` shot of each bar.
- [ ] 4.2 Death, the dropped pack and waking up (design 8). Verify: an app test that a death drops every item in one bag, the bag survives a reload, and picking it up returns them.
- [ ] 4.3 Predators hunt in Survival (design 6), never in Explorer. Verify: a test per state change; a test that an Explorer fox never attacks.
- [ ] 4.4 The held tool as a weapon at the swing's hit moment. Verify: a test that an axe hit lowers a predator's health by its table value.
- [ ] 4.5 Townsfolk fight back and reputation drops (design 7). Verify: an app test that a struck villager brings the guards and lowers the town's standing; a swing in Explorer does nothing.
- [ ] 4.8 Tool wear in the game: uses on the owned-tool record (an old save reads full), spent in the same durable line as the action, the broken tool as bare hands, repair at the workbench and at a town's smith or trader for coins. Verify: app tests for a tool breaking and surviving a reload, and for both repairs; a `--capture` shot of a worn and a broken tool's slot.
- [ ] 4.9 Stamina, water and temperature in the walker and the HUD: sprint, swim, tread, jump and climb spend through the core rule; winded limits; worn slots. Verify: app tests that a sprint drains and refills, a winded swimmer floats, a hat and a cloak change `feel`; `--capture` shots of hot, cold and winded.
- [ ] 4.10 The goods, recipes and stations: `raw-meat`, `hide`, `bone`, `feather`, `straw`, the arrows, spears, bow, hunting axes, swords, `straw-hat`, `wool-cloak`, `bedroll`, `meat-stew`, the tanning rack, append-only, names in the locale file, icons by `tools/gen_item_icons.py`. Verify: the recipe tests, the locale lint, an Explorer world offering none of them.
- [ ] 4.11 The game and predator species appended to the home roster (hare, ridge goat, rootling, red fox, dust hound), textured, smooth-shaded and lit by the field, with motion by the `animation` skill and M7's sprites. Verify: the husbandry roster tests (append-only), a `--capture` close-up of each by day and at night.
- [ ] 4.6 The HUD from the approved mockup. Verify: `--capture` shots of each state in `docs/screenshots/survival-mode/`.
- [ ] 4.7 Multiplayer: two players and a fox show the same hit; a swing at another player does nothing. Verify: a two-client test.

## 6. The Survival branch (design 18)

- [ ] 6.1 `mode: survival` on missions in `missions.ron`, the branch's 23 missions and their unlocks, append-only ids; the Missions screen hides them in Explorer. Verify: a test that an Explorer world shows and counts none, and a Survival world makes `drink` the current goal when `pack` is done (owner, G11), while an Explorer world goes on to `axe`.
- [ ] 6.2 The new objective kinds (`drink`, `kill`, `dawn`, `cool`, `warm`, `wear`, `dive`, `fall`, `drive-off`, `relight`, `douse`), each raised where the game already acts. Verify: a unit test per kind.
- [ ] 6.3 The bedroll as a spawn (design 18, decision 8). Verify: an app test that a death wakes the player at a laid bedroll.
- [ ] 6.4 The bot plays the branch: "survive the first night in Survival" from `drink` to `first-night`, then each side branch. Verify: the run's log names each mission done; screenshots of each step (CLAUDE.md, screenshots stand in for videos).

## 5. The owner's gate

- [ ] 5.1 Screenshots of each survival state, then the gameplay demo (CLAUDE.md "Demos"): a bot with the goal "survive the first night in Survival", which eats, swims, fends off a fox and recovers its pack after a fall, and repairs a pickaxe that broke. Verify: the owner watches and approves.

## Workflow follow-up

- Sync `player/survival` into `openspec/specs/` requirement by requirement, with the test that proves each.
- Follow-ups: armour, butchering kept animals, food spoiling, a larger suit tank with the rocketry tiers, craft crash damage, a mode switch if the owner wants one.
