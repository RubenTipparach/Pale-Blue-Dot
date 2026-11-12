# Tasks

The mockups come first and are the whole of this change until the owner
approves them. Game code waits for a request (CLAUDE.md, "Write it up before
touching code"). Groups 4 to 6 are the build, listed so the mockups are
judged against what they would cost.

## 1. The mockups

- [x] 1.1 M22 Cranes and Printers, `docs/mockups/construction.html` (https://claude.ai/artifact/2LjxDc7LuNFX6yjPbMLMwc), with `tools/art/construction_art.py` and the game's art in `construction/game/`. Verify: day and night shots in `docs/screenshots/mockup-construction/`; the print slider runs plot to roof; an overload is refused with its number.
- [x] 1.2 M23 Drydock and Hex Ship Builder, `docs/mockups/drydock.html` (https://claude.ai/artifact/DLRzGBKMJzhhFdayrbCVdx), with `tools/art/drydock_art.py` and the game's art in `drydock/game/`. Verify: day and night shots in `docs/screenshots/mockup-drydock/`; the water slider floats the ship off its blocks at its solved draft; the walk-the-deck view keeps the walker on the deck through the turn.
- [x] 1.3 M24 Hangar and Chibi Aircraft, `docs/mockups/hangar.html` (https://claude.ai/artifact/HrLdbcwrwbAD7PN1KojKKu), with `tools/art/hangar_art.py` and the game's art in `hangar/game/`. Verify: day and night shots and the five presets in `docs/screenshots/mockup-hangar/`; a helicopter with no tail rotor is refused.
- [x] 1.4 Publish the three, list them on the tracker (M22 to M24) and in `docs/mockups/mockup-work.md`, and record their links here. Verify: each page carries the tracker link under its title.
- [x] 1.5 Put the two questions judged by eye (chibi proportions; hex hull blockiness) in the survey, with screenshots. Verify: they are in `docs/design/survey.html` with the shots beside them (A1, A2; https://claude.ai/artifact/3ViY6udhFkkx5Wzx4jn3w5).

## 2. The owner's review

- [ ] 2.1 Read the tracker's `reviews` for M22 to M24 and the survey answers, and fold them into this design. Verify: the stages are in `mockup-work.md`, and the quotes are in `proposal.md`.

## 3. Videos (batch)

- [ ] 3.1 A walkthrough video for each mockup when the batch is recorded (CLAUDE.md, "For now, screenshots stand in"). Verify: on the gate page.

## 4. Construction (on request, after approval)

- [ ] 4.1 Crated items as stored records; the assembly bay's queue. Verify: format tests, and an app test that a crate survives a reload where it was set down.
- [ ] 4.2 The hex printer: one layer a pass as one edit authored by the printer. Verify: an app test prints a cottage blueprint, stops it at layer 4, reloads, and finds four layers.
- [ ] 4.3 The tower crane's seat, controls and load chart. Verify: core tests of the chart at the tip and close in.

## 5. Ships (on request, after approval)

- [ ] 5.1 A hex ship record in its own frame, with its cells floated as seven sub-cells. Verify: a core test that a solved draft is where the ship settles, within 0.1 m.
- [ ] 5.2 The builder's checks, including the gangway-to-helm walk. Verify: one core test per check, with the reason text.
- [ ] 5.3 The drydock: gate, pump, flooding and floating out. Verify: an app test that floods a dock and the ship lifts at its draft.
- [ ] 5.4 Walking a hex ship's decks through the cog's `CraftDecks` path. Verify: the cog's walker tests, run on a hex ship.

## 6. Aircraft (on request, after approval)

- [ ] 6.1 The aircraft record with livery; the hangar's workbench and door. Verify: format tests, and a span-too-wide refusal.
- [ ] 6.2 Parts to foils through the Kestrel's model. Verify: core tests that wing cells change the stall by `sqrt(S / (S + 2))`, and that a helicopter with no tail rotor is refused.
- [ ] 6.3 Frame cost of ships and aircraft with many cells. Verify: `tools/perf_suite.py` on the owner's hardware (not in a cloud session).
