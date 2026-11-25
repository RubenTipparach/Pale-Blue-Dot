# Design

## 1. The rule

In `digging.rs`'s place path, where `occupies` refuses a block in the
player's capsule:
- the feet's layer `F` is the layer under the head's (`occupies` reads the
  head from the eye), and the target is `(cell, F)` with `cell` the player's
  own (the contact tier's key under the eye);
- the head room is the layers `F + 1` and `F + 2` of that cell, the player's
  body after the lift. `F + 1` is where the head is now and is open. `F + 2`
  must be open too (air, water or a light);
- then the edit is made as any place is (the durable path), and a lift of
  one metre is asked of the walker.

A target in a neighbouring cell that the capsule's rim overlaps is still
refused: the walker is lifted straight up, never sideways.

## 2. The lift

`walking::Lift(f32)`, a resource: metres to raise the walker this step.
`resolve_ground` adds it along the walker's up before it resolves the ground,
keeps the lift as the previous position (no speed from it) and clears it.
The view eases up as on an auto step.
