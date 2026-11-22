# Tasks: land-transport

Phase 5. Only section 1 is work now; the rest waits for the owner to approve
the mockup and then to ask for phase 5.

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
