# Tasks

## 1. The core: an edit is data, and a column applies it
- [x] `pbd_core::column::Edits`: stable-cell-ID keyed, a stable `Vec` of
      (layer, material) per cell, with `dig`, `place` and `apply`.
- [x] `generate` takes `&Edits` and applies them after the carve, so a column
      stays a pure function of its inputs.
- [x] Tests: the same column twice, an unedited column untouched, bedrock
      refused, a dug layer air, a placed layer its material.

## 2. The core: the march that finds the target
- [x] `column::Target { cell, layer, place_layer }` and a march over a
      cell-and-layer lookup, 0.25 m steps to `DIG_REACH_M`.
- [x] Tests: a ray down onto flat ground takes the top layer and places on the
      one above it; a ray into the sky takes nothing.

## 3. The app: input, inventory, and the rebuild
- [x] Left click digs, right click places, on the walker's own look ray.
- [x] `Slots::give` on a dig, `take_held` on a place - hotbar-first, which is
      what those functions already do.
- [x] Regenerate the edited column, repack it and its neighbours' records,
      rerun the mouth rule for the edited cell, and refresh the contact.
- [x] A place is refused where the player's own capsule stands.

## 4. Durability
- [x] Append each accepted edit to the save and flush before returning.
- [x] Replay the save into `Edits` at startup.
- [x] Test: a round trip through the file gives the same `Edits`.

## 5. Proof
- [x] `--dig N` and `--place`, with `--walk`: a headless run has no mouse, and
      a picture of a hole is the only thing that says the verb works end to
      end. It digs straight DOWN, because the one thing certainly within reach
      of a standing player is the ground under them.
- [x] Measured end to end: dig 2 in one run, then a run with no flags at all
      loads "2 edits across 1 cells" from the save and the walker stands in
      the hole it dug last time.
- [ ] The owner's in-game confirmation, which is the only thing that counts
      for feel: the reach, what a click takes, and whether a placed block
      lands where the reticle says.
