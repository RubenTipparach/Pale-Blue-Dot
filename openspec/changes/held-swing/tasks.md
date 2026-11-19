# Tasks

- [x] 1.1 Mockup: the STRIKE stroke in the Dig and Chop tabs of `tool-motion.html` (M36) beside today's sine, with the curve's values as sliders; on the tracker; the owner judges.
- [x] 1.2 Approved for the game with Tools in Hand version 8 (owner, 2026-11-17: "tools in hand approved, implment").
- [x] 2.1 (after approval) The stroke in `held.ron` (`stroke`, `shovel`, the mockup's Copy RON schema and values); `held/stroke.rs` gives the stage and the impacts from the dig's beat (decision 5), and `hold` draws from it: the pickaxe and the axe on the shoulder arc in four stages (decisions 6-8), the shovel's tip, slide, lever and pull (decision 9); tests on the schedule (`held::stroke::tests`).
- [x] 2.2 (after approval) Dig sounds on the impacts: each impact but a block's last plays the dig sound, the last is the break's (`StrokeImpact`, `desktop/sounds.rs`).
- [x] 2.3 (after approval) Eased return to rest: letting go eases to rest over `recover_s`; taking up again, a block started again, a tool or a break time changed eases from where the tool is drawn; a test that no state change pops.
- [x] 2.4 The shovel's clump of dirt: seven dirt-coloured hex chips on the blade, shown while it scoops (the tip, the pull up and the first part of the return).
- [x] 2.5 Captures before and after (`docs/screenshots/held-swing/`), on lavapipe in the cloud; frame cost not measured (no GPU).
- [ ] 2.6 The owner's look in the game.
- [x] 2.7 The shoulder stays low (decision 9, Tools in Hand version 12): `fist_drop_m` 0.16, `arm_follow` 0.7 in `held.ron` and the defaults; the keys and `shovelAt` were already the mockup's; a test on the keyed poses; a capture (`docs/screenshots/held-swing/shovel-low-*.jpg`).

Punch and a swing on every click (design 10 to 12; owner, 2026-11-19: "do tools always play animations when left clicking? can you mock up hand punching stuff"):
- [x] 3.1 Mockup: the Punch tab of Tools in Hand version 14 (`tool-motion.html#punch`): bare hands punching a tree (the game's shake on each punch), dirt, the air and a refused stone; Hands, Pickaxe and Axe at the air; Today (no hand; no swing but on a breaking block) against Proposed; the `punch` and `swing` blocks as sliders with their RON. Screenshots in `docs/screenshots/hand-punch/` (SwiftShader, no GPU).
- [ ] 3.2 The owner's look at the mockup (decision 11 is a recommendation taken, ask only with screenshots; the tab shows each case).
- [ ] 3.3 (after approval) `held.ron` `punch` and `swing` blocks, validated, one source of defaults; `DigBeat` carries what the click meets and the press; `StrokeClock` runs the free beat at the air and a refused block; bare hands draw the fist (the plug, `punch_pose`); tests on the schedule (design, "In the game, once approved").
- [ ] 3.4 (after approval) The tree shakes on the stroke's impacts (`StrokeImpact` on a tree) and `HOLD_HIT_S` goes; `tree_shake` tests follow.
- [ ] 3.5 (after approval) A whoosh at the air and a knock on a thud, from the stroke's schedule (`sound-effects`).
- [ ] 3.6 (after approval) Captures before and after in the game (`docs/screenshots/hand-punch/`).

Follow-ups, not built:
- A camera kick on the impact (`camera_kick_rad`, approved at 0): the walking camera's look is the player's raw mouse, so a kick needs its own decision; `held.ron` refuses any value but 0 until then.
- The mockup's chips flying off each impact and the hit marker (secondary action, decision 8): the game shows the block's cracks and plays the dig sound; chips would be a small particle system of their own.
