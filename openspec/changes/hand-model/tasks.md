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
- [ ] 3.3 (after approval) The generator and the game: a fifth texture (the palm), the faces split by side
