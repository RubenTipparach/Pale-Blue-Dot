# Tasks

- [x] 1.1 Mockup: the STRIKE stroke in the Dig and Chop tabs of `tool-motion.html` (M36) beside today's sine, with the curve's values as sliders; on the tracker; the owner judges.
- [x] 1.2 Approved for the game with Tools in Hand version 8 (owner, 2026-11-17: "tools in hand approved, implment").
- [x] 2.1 (after approval) The stroke in `held.ron` (`stroke`, `shovel`, the mockup's Copy RON schema and values); `held/stroke.rs` gives the stage and the impacts from the dig's beat (decision 5), and `hold` draws from it: the pickaxe and the axe on the shoulder arc in four stages (decisions 6-8), the shovel's tip, slide, lever and pull (decision 9); tests on the schedule (`held::stroke::tests`).
- [x] 2.2 (after approval) Dig sounds on the impacts: each impact but a block's last plays the dig sound, the last is the break's (`StrokeImpact`, `desktop/sounds.rs`).
- [x] 2.3 (after approval) Eased return to rest: letting go eases to rest over `recover_s`; taking up again, a block started again, a tool or a break time changed eases from where the tool is drawn; a test that no state change pops.
- [x] 2.4 The shovel's clump of dirt: seven dirt-coloured hex chips on the blade, shown while it scoops (the tip, the pull up and the first part of the return).
- [x] 2.5 Captures before and after (`docs/screenshots/held-swing/`), on lavapipe in the cloud; frame cost not measured (no GPU).
- [ ] 2.6 The owner's look in the game.

Follow-ups, not built:
- A camera kick on the impact (`camera_kick_rad`, approved at 0): the walking camera's look is the player's raw mouse, so a kick needs its own decision; `held.ron` refuses any value but 0 until then.
- The mockup's chips flying off each impact and the hit marker (secondary action, decision 8): the game shows the block's cracks and plays the dig sound; chips would be a small particle system of their own.
