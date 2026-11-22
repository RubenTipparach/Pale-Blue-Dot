# Tasks

Phase 4 (moved from 3c, owner, 2026-11-22), after `multiplayer` and `game-launcher` (3b). Nothing here starts
until the owner asks.

## 1. Mockups for the owner

- [ ] 1.1 The survival HUD (design 9): health, hunger and breath beside the hotbar, a tool's wear bar and a broken tool's icon (design 11), the hit edge and "You died", Explorer beside Survival, by day and lit at night. Published, on the tracker. Verify: the owner approves it.
- [ ] 1.2 A predator hunt (design 6): notice, stalk, warn, attack and flee, the player's swing as the hit, by the `animation` skill, by day and lit at night. Verify: the owner approves it.

## 2. The mode

- [ ] 2.1 The world's mode in its identity record, Explorer for every older save; the choice on the new-world screen and in the world list. Verify: a test that an old save opens as Explorer; a Survival world reopens as Survival.

## 3. Rules in the core

- [ ] 3.1 `pbd_core::survival`: health, hunger, breath and suit air as pure functions of state, play time and events; `assets/config/survival.ron` with units, validated. Verify: unit tests for refill and drain rates, and that time with the game closed changes nothing.
- [ ] 3.2 The fall rule (design 3) from impact speed along the body's up, with each body's gravity and water cancelling. Verify: tests for a two-layer step, a twenty-layer fall, the same fall on a half-gravity body, and a fall into deep water.
- [ ] 3.3 Meals fill hunger (`fills` per dish, scaled by stars). Verify: a test per star level.
- [ ] 3.4 Tool wear (design 11): uses by tier in `survival.ron`, one use per finished action and two for a harder block, broken at zero, repair costs. Verify: unit tests for each action, a missed swing costing nothing, a tool breaking and being repaired, and Explorer never wearing.

## 4. In the game

- [ ] 4.1 The server ticks survival for each player in Survival worlds; the player record gains health, hunger, breath and spawn. Verify: an app test that a long dive drains health and surfacing refills breath; a `--capture` shot of each bar.
- [ ] 4.2 Death, the dropped pack and waking up (design 8). Verify: an app test that a death drops every item in one bag, the bag survives a reload, and picking it up returns them.
- [ ] 4.3 Predators hunt in Survival (design 6), never in Explorer. Verify: a test per state change; a test that an Explorer fox never attacks.
- [ ] 4.4 The held tool as a weapon at the swing's hit moment. Verify: a test that an axe hit lowers a predator's health by its table value.
- [ ] 4.5 Townsfolk fight back and reputation drops (design 7). Verify: an app test that a struck villager brings the guards and lowers the town's standing; a swing in Explorer does nothing.
- [ ] 4.8 Tool wear in the game: uses on the owned-tool record (an old save reads full), spent in the same durable line as the action, the broken tool as bare hands, repair at the workbench and at a town's smith or trader for coins. Verify: app tests for a tool breaking and surviving a reload, and for both repairs; a `--capture` shot of a worn and a broken tool's slot.
- [ ] 4.6 The HUD from the approved mockup. Verify: `--capture` shots of each state in `docs/screenshots/survival-mode/`.
- [ ] 4.7 Multiplayer: two players and a fox show the same hit; a swing at another player does nothing. Verify: a two-client test.

## 5. The owner's gate

- [ ] 5.1 Screenshots of each survival state, then the gameplay demo (CLAUDE.md "Demos"): a bot with the goal "survive the first night in Survival", which eats, swims, fends off a fox and recovers its pack after a fall, and repairs a pickaxe that broke. Verify: the owner watches and approves.

## Workflow follow-up

- Sync `player/survival` into `openspec/specs/` requirement by requirement, with the test that proves each.
- Follow-ups: armour, ranged weapons, cold and heat (Warmth's mechanic), a larger suit tank with the rocketry tiers, craft crash damage, a mode switch if the owner wants one.
