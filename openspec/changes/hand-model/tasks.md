# Tasks

- [x] 1.1 Mockup: the Hand tab in Tools in Hand (M36), the game's hand against the proposed hand,
  with diagnostic views and the textures
- [x] 1.2 The owner picks (section 2 of the design): the proposed hand, approved with Tools in Hand version 8 (owner, 2026-11-17: "tools in hand approved, implment")
- [x] 2.1 (after approval) The generator writes the new hand and its textures: `tools/gen_held_tools.py` ports the mockup's `proposedHandGeometry` (the blocks per grip, the two-segment thumb, the nails, the hem, the wrist joint) into `assets/models/held_tools.ron` and its `handTextures` into `assets/textures/hand/*.png` (identical to the mockup's, pixel for pixel); `pbd_core::hand` builds the chamfered blocks
- [x] 2.2 (after approval) `held.rs`: four textured, nearest-sampled materials, and the wrapped light in eye space, re-baked into the vertex colours when the hand turns (`shade_hands`); still multiplied by the field's light at the eye (`light_held`)
- [x] 2.3 (after approval) Tests and captures, before and after (`docs/screenshots/hand-model/`): closed blocks facing out, the texel size, the sleeve's growth and the wrapped light (`pbd_core::hand::tests`); the thumb's two segments, one skin with nails, cuff and sleeve, the fist on the handle, the textures' size and palette, the lit side staying toward the light, the hand re-lit as it swings (`held::tests`)
- [ ] 2.4 The owner's look in the game
- [x] 3.1 Mockup: palm and back (design 5) on the Hand tab of Tools in Hand, with a switch against one tone
- [x] 3.2 The owner's look at the mockup: version 10's hand ("You know what that hand looks good", 2026-11-17)
- [x] 3.3 (after approval; owner, 2026-11-18: "cool now implement all these") The generator and the game: versions 9 to 13 (design 5 to 5.3, as built in design 6): the traced palm, the slimmer tapering fingers, the thumb's round tip and sunk root, the faces split by side with the palm's and folds' islands, the palm (32 px) and fold (16 px) textures, the back's tone, the baked AO, the fingertips' flush, the 45 degree grip turn; `pbd_core::hand::build` from the bones in `held_tools.ron`
- [x] 3.4 Tests: the palm closed and facing out, the palm and back sides and the island, the thumb's two segments rooted in the palm, the AO between the fingers, the grip's turn, the textures' sizes and palettes, the flush (`held::tests`, `pbd_core::hand::tests`)
- [x] 3.5 Captures before and after (`docs/screenshots/hand-model/`), on lavapipe; frame cost not measured
- [ ] 3.6 The owner's look in the game
