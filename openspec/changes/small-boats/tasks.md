# Tasks

The owner's request (2026-11-16), quoted in the proposal. Tugs are out of
scope (they need a dock).

## 1. Craft (`pbd-core`)

- [ ] 1.1 `Kind::Tube` and `Kind::Raft` (keys `tube`, `raft`), each with its own `LoonSpec` (`tube()`, `raft()`) and hull, stepped by the Loon's model; `vehicles.ron` regenerated. Verify: core tests that each floats upright at rest with its paddler, deck clear, and moves ahead on a forward stroke.
- [ ] 1.2 `VehicleRecord.made`, written only when set. Verify: a record round trip; an old record writes byte for byte as before.

## 2. Recipes and goods

- [ ] 2.1 Goods `inner-tube`, `raft`, `rowboat`, appended, stack 1, 16 px icons with provenance. Verify: the goods table test.
- [ ] 2.2 Recipes INNER TUBE (hand), RAFT and ROWBOAT (workbench), in `recipes.ron` and the code default. Verify: the recipe data test.

## 3. Launch and take up (`pbd-app`)

- [ ] 3.1 The place button with a boat good on water in reach launches the craft (snapshot, then slots). Verify: an app test that the good leaves the slots and a made craft of that kind is in the fleet.
- [ ] 3.2 The dig button held 1 s on a made, empty, unoccupied craft takes it up (slots, then snapshot). Verify: an app test; a harbour boat is refused.

## 4. The raft's hold

- [ ] 4.1 The `hold` record (ten stacks, the pack's stack text) and moves between it and the slots in one line. Verify: core tests of the moves; a save test of a reopened hold.
- [ ] 4.2 The pack opens aboard the paddled boats and shows the HOLD row for a raft aboard or in reach. Verify: a capture.

## 5. Fishing afloat

- [ ] 5.1 The angler is the seat aboard a tube, raft, canoe or rowboat; G picks the rod there; the rod is drawn in the seat view; boarding or leaving winds the line in. Verify: an app test that a cast from a raft's seat puts the float on the water; a capture.

## 6. Checks and captures

- [ ] 6.1 `cargo fmt`, clippy, tests, `openspec validate --all`.
- [ ] 6.2 Captures in `docs/screenshots/small-boats/`: the tube, the raft and the rowboat on water, the hold row, a cast from the raft's seat. Each with its launch line. (Cloud: lavapipe, no frame numbers.)
- [ ] 6.3 Video for the owner's batch (deferred with the others).
