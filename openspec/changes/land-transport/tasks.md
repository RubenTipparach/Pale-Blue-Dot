# Tasks: land-transport

Phase 5, but for the hand cart and the horse cart, which are phase 3 (owner,
2026-11-22; section 5). Sections 1 and 5.1 are work now; the rest waits for
the owner to look at M38 version 2 and ask.

## 1. The mockup and the plan

- [x] 1.1 M38 Land Transport (`docs/mockups/land-transport.html`): Carts and
      horses, Cars, Trains and tracks, Side by side; one 3D view a tab, made
      when the tab opens; Drive off by default; lit at night by its own lamps.
- [x] 1.2 Screenshots of every tab by day and by night in
      `docs/screenshots/land-transport/` (headless Chromium on SwiftShader,
      software GL, in a cloud session; no frame times taken).
- [x] 1.3 This change: proposal, design, tasks, the `world/land-transport`
      spec delta; `openspec validate --all` passes.
- [x] 1.4 The row in `docs/mockups/mockup-work.md`.
- [ ] 1.5 Published and listed on the tracker (the owner's session).
- [ ] 1.6 The owner's review of M38; decisions turned down come back to
      design.md.

## 2. Road vehicles (waits for phase 5)

- [ ] 2.1 `pbd-core`: a road vehicle's placement as a pure function of the
      road cells and the distance along its drive line (design 3), with the
      60° bend arc (design 2), and tests on a slope, a bend and a bridge.
- [ ] 2.2 The horse as a ridden mount and a draught animal, by its animal ID.
- [ ] 2.3 Cart, wagon, car and truck models and their lamps, lit by the field.
- [ ] 2.4 Vehicle records on the durable path (design 8), with a save test.
- [ ] 2.5 Passing places on the world's roads (design 2), as a new road
      generator version so existing worlds keep theirs.

## 3. Track and trains (waits for phase 5)

- [ ] 3.1 Track pieces: straights and `k = 2`, `k = 3` curves on the grid
      (design 4), with a test that six curves and matching straights close.
- [ ] 3.2 The bed's cut and fill as cell edits (design 5), with a test of a
      grade over a terrace.
- [ ] 3.3 Platforms (design 6).
- [ ] 3.4 The tank engine, coach and wagon; trains as records (design 8).

## 4. Verification (waits for phase 5)

- [ ] 4.1 In-game captures beside the mockup's shots, by day and night.
- [ ] 4.2 A step video (or screenshots, while they stand in) for the owner's
      gate.

## 5. Phase 3: hand carts and horse carts (added 2026-11-22: "we also need to add basic carts to phase 3 as well, and horse carts too")

- [x] 5.1 Design 9 and 10 and M38 version 2: the Carts and horses tab marks
      the hand cart and the horse cart as phase 3 (a hand cart added beside
      them), and a Road or not tab drives both carts on a gravel road, a dirt
      track and open ground at design 9's factors, Drive off by default, lit
      at night. Screenshots in `docs/screenshots/town-missions-tree/`.
- [x] 5.2 `roads::wheel_factor` and its `roads.ron` table (design 9). Verify:
      `pbd-core` tests that a cart off the road moves at a quarter of its
      road speed, on dirt at 0.6, in a ford at 0.5, and at the lower factor
      with its axles on two surfaces.
- [x] 5.3 The wheel, hand cart and horse cart recipes, goods and icons; the
      place action that makes a vehicle record and the take-up that undoes
      it. Verify: a save test that a cart left with goods in its hold is
      where it was, loaded, on reopening.
- [x] 5.4 Pulling a hand cart and driving a horse cart; the hold on C.
      Verify: an app test that letting go keeps the record and its hold.
      Built (2026-11-23): the hold shows in the pack (I or Tab) beside a cart
      or from its seat, at its own size; a C key of its own for carts is not
      built.
- [x] 5.5 The horse: the `horse` species appended with status Bought, the
      stable keeper and the 120-coin sale, hitching in one transaction.
      Verify: a test that every existing world's wild herds are unchanged
      animal for animal, and that hitching writes both records or neither.
      Built (2026-11-23): the town's trader sells the horse (the stable
      keeper waits for a roster role, design 11); hitched is the cart's one
      record naming the horse, so it is written whole or not at all.
- [x] 5.6 The cart and horse models, lit by the field, looked at close up
      (CLAUDE.md, "No obvious primitives"). Verify: captures by day and at
      22:30 beside M38's shots. Built (2026-11-23): captures
      `docs/screenshots/land-transport/game-*` (lavapipe, no frame times);
      the motion is the owner's to judge on a run of their own.
- [ ] 5.7 Design 12: the carts in `timber` and `iron`, the horse and the herd
      in the `coat` texture, smooth-shaded lofted forms. Verify: builder
      tests that every model's UVs and normals are whole and smooth across a
      body; BEFORE | AFTER captures of each cart and each species, by day and
      at 22:30, no `LATE_PIPELINE`.
- [ ] 5.8 Design 13: a plodder hitched to the horse cart at its own speed.
      Verify: tests that a plodder is hitched by the one-record hitch, reads
      back after a reload, drives at `plodder_mps`, and that an old save's
      horse cart reads unchanged; a capture of a plodder in the shafts.
