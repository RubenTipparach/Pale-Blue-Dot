# Tasks

The mockup comes first and is the whole of this change until the owner
approves it. Game code waits for a request (CLAUDE.md, "Write it up before
touching code"). Groups 4 to 8 are the build in slices, listed so the mockup is
judged against what it would cost.

## 1. The mockup

- [x] 1.1 `docs/mockups/husbandry.html`, "Pens, Nests and Pedigrees": three 3D tabs (the farmstead by day and night, taming a wild herd, the hatchery), and the Kingdom, Breeding, Economy, Engine and Decisions tabs; `tools/art/husbandry_art.py`; the game's art in `docs/mockups/husbandry/game/`, M7's, M25's, M9's, M1's and M8's in `husbandry/m7/`, `m25/`, `m9/`, `m1/`, `m8/`, with a README naming every source. Verify: headless Chromium (SwiftShader, no GPU) shows no console errors; every tab opens and draws; one WebGL context for the whole page; the camera stays put when idle; no horizontal scroll at 390 px.
- [x] 1.2 Screenshots of the farmstead by day and at night, the fox at night, taming, eggs hatching, the life stages, the breeding screen, the Kingdom and the economy, in `docs/screenshots/mockup-husbandry/`. Verify: they are there, and say they were rendered on SwiftShader.
- [ ] 1.3 Publish the page, list it on the tracker and in `docs/mockups/mockup-work.md`, and record its link here. Verify: the page carries the tracker link under its title (it does), and the tracker lists it.
- [ ] 1.4 Put Q1 to Q3 in the survey with their screenshots. Verify: they are in `docs/design/survey.html`.

## 2. The owner's review

- [ ] 2.1 Read the tracker's `reviews` and the survey answers, and fold them into this design. Verify: the stage is in `mockup-work.md`, and the quotes are in `proposal.md`.

## 3. Videos (batch)

- [ ] 3.1 A walkthrough video of the page when the batch is recorded (CLAUDE.md, "For now, screenshots stand in"). Verify: on the gate page.

## 4. The kingdom in data (on request, after approval)

- [ ] 4.1 `class` and `status` on every species in `fauna.ron`; the land species appended to `home`, Sequoia's, Wick's and Fallow's rosters. Verify: the append-only test still passes; a test that every species has a class, a status and a field-guide entry, and that an airless body's roster is empty.

## 5. Taming and the animal record (on request, after approval)

- [ ] 5.1 `pbd_core::husbandry` genome (seven genes, two alleles) rolled from a wild animal's seed. Verify: a core test that the same seed gives the same genome, and the morph shows in 1 of 64 over 64,000 rolls within 10%.
- [ ] 5.2 Taming by feeding: trust, fear radius, decay; the `animal` record written through the journal `@p` on taming. Verify: a test that a tamed animal survives quit and reload with its genome; a test that a wild-only or protected species refuses with its reason.
- [ ] 5.3 The record's schema version in `identity.ron`. Verify: a world saved before it opens and gains no animals; a v1 record opens under a v2 build with the new gene at its default.

## 6. Pens, needs and products (on request, after approval)

- [ ] 6.1 Pens derived from fence and gate pieces. Verify: a core test that a closed loop with a gate is a pen and a broken loop is not.
- [ ] 6.2 Needs, mood, health and straying; the `husbandry` world process run by the steward. Verify: the catch-up test (a closed form for N hours equals N stepwise hours); a test that a process proposal changing a player-written field is refused.
- [ ] 6.3 Products by schedule and yield; compost from manure; the butcher's block. Verify: a test of a day's output for a known herd against the Economy tab's numbers.
- [ ] 6.4 Animals near the player as entities, far pens as numbers. Verify: captures by day and night beside the mockup's; `tools/perf_suite.py` before and after on the owner's hardware.

## 7. Eggs and young (on request, after approval)

- [ ] 7.1 Fertile eggs, broody hens, the incubator; life stages from the birth day. Verify: a test that an egg set on day d hatches on day d + 3 and lays from day d + 13.

## 8. Breeding (on request, after approval)

- [ ] 8.1 Pairing, gestation, cooldown; inheritance with mutation; Wright's F from the pedigree. Verify: a test that the four predicted values for a gene are exactly the ones births produce over many seeds; siblings give F = 0.25.
- [ ] 8.2 The breeding screen and the pedigree. Verify: a capture beside the mockup's Breeding tab.
