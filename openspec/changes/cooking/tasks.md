# Tasks

The mockup comes first and is the whole of this change until the owner
approves it. Game code waits for a request (CLAUDE.md, "Write it up before
touching code"). Groups 4 to 8 are the build, listed so the mockup is judged
against what it would cost.

## 1. The mockup

- [x] 1.1 `docs/mockups/cooking.html`, "Hearth and Kitchen": four kitchens as hex-cell scenes by day, dusk and night (yard, bakery, inn kitchen, galley); the cooking panel with heat, freshness, quality and the HUD; the recipe book; field to plate; new crops; meat, milk and eggs; fish and seafood; what food does; decisions. `tools/art/cooking_art.py`; the game's art in `docs/mockups/cooking/game/`, M3's, M7's, M8's and M25's in `cooking/m3/`, `m7/`, `m8/`, `m25/`. Verify: headless Chromium (SwiftShader, no GPU) shows no console errors; every tab opens and draws; one WebGL context for the whole page and the loop stops on a 2D tab; the camera stays put when idle; no horizontal scroll at 390 px.
- [x] 1.2 Screenshots of each kitchen by day and at night, the yard at dusk, a picked station, the cooking panel, the HUD with buffs and with a hunger bar, the pack two days later, the recipe book, field to plate, the crops, the ranch and the sea tables, and the page at 390 px, in `docs/screenshots/mockup-cooking/`. Verify: they are there, and say they were rendered on SwiftShader.
- [ ] 1.3 Publish the page, list it on the tracker and in `docs/mockups/mockup-work.md`, and record its link here. Verify: the page carries the tracker link under its title (it does), and the tracker lists it.
- [ ] 1.4 Put Q1 to Q3 in the survey with their screenshots. Verify: they are in `docs/design/survey.html`.
- [ ] 1.5 Screenshot the page on the owner's desktop GPU (ART-DIRECTION.md, "The gate before a page is published") and set it beside an in-game shot. Verify: the shots are in `docs/screenshots/mockup-cooking/`.

## 2. The owner's review

- [ ] 2.1 Read the tracker's `reviews` and the survey answers, and fold them into this design. Verify: the stage is in `mockup-work.md`, and the quotes are in `proposal.md`.
- [ ] 2.2 Tell the farming (M8) and seas (`ocean-environments`) owners' changes about the twelve crops and three catchables, and the husbandry draft (`animal-husbandry`) about the cuts and the venison gap. Verify: each change names this one.

## 3. Videos (batch)

- [ ] 3.1 A walkthrough video of the page when the batch is recorded (CLAUDE.md, "For now, screenshots stand in"). Verify: on the gate page.

## 4. Food in the inventory (on request, after approval)

- [ ] 4.1 `Item::Food(u16)` and an append-only food roster; `made_at` and `stars` on a stack; merging keeps the older time. Verify: inventory tests for merge and give order; a save with fish and no food round-trips unchanged.
- [ ] 4.2 Freshness bands and spoiling to compost from world time. Verify: a core test that two played days spoil a one-day food into the same count of compost, and that closed time ages nothing.

## 5. Stations and recipes (on request, after approval)

- [ ] 5.1 Kitchen stations as placed records with jobs that run only in play, fuel as an ingredient, and recipes in `recipes.ron`. Verify: a job-time test across a save and load; a recipe-data test that every ingredient and output exists.
- [ ] 5.2 The station panel with heat, freshness and stars. Verify: a capture beside `cook-panel.png`; the deterministic-stars test.

## 6. Eating and effects (on request, after approval)

- [ ] 6.1 Eat, three effects at once, stride, steady hands, keen and glow wired to walking, fishing, digging and lights; buff icons over the hotbar. Verify: tests per effect against the configured values; a capture beside `cook-hud-buffs.png`.

## 7. Crops (on request, after approval, with M8)

- [ ] 7.1 The twelve crops appended to the roster, their stage sprites, perennials. Verify: the append-only test; the ground digest unchanged.

## 8. Kitchens in the game (on request, after approval)

- [ ] 8.1 Station models and their lights; kitchens in towns (the bakery, the inn's kitchen). Verify: captures by day and at night; `tools/perf_suite.py` before and after on the owner's hardware, since each oven and fire is a lamp.
