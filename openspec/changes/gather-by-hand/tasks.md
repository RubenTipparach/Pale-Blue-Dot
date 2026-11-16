# Tasks

Owner's request 2026-11-16; written up, not started.

- [ ] 1 `pbd_core::scatter::pieces`, the shader's pebble, bush, dead shrub and fern placement ported. Verify: core tests of the rolls against known ids; the GPU test against `planet_surface.wgsl` (design 2).
- [ ] 2 The `clutter` record and regrowth. Verify: round trip; a piece taken is taken until three days pass.
- [ ] 3 F aims at clutter and picks it up (design 1, 3), `How::Gathered` counted as `gather` and `find`. Verify: an app test picks a pebble and a bush; "Gather by hand" closes on F alone.
- [ ] 4 The draw drops taken pieces and dissolves them out and in (design 5). Verify: captures before and after a pick-up; a test of the list's reach.
- [ ] 5 `PickUp` in the showcase and `phase-1-trunk.ron` (design 7). Verify: the parse test; the owner's run.
- [ ] 6 Screenshots to the owner; the video waits for the batch.
