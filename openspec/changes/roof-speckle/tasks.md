# Tasks

## 1. Write-up and instruments
- [x] 1.1 Proposal, design, spec delta.
- [x] 1.2 Instruments: `measure_faces_that_can_fight`, `measure_faces_on_a_lights_band_edge` (ignored tests), and the throwaway soffit override (design 2).
- [x] 1.3 Reproduce in a capture and name the faces (design 1-3).

## 2. Fix
- [x] 2.1 `BAND_SLACK_M` in `pbd_core::settlement::pieces`; `BAND_SLACK` in `field_lit.wgsl`'s `room_lights` on both edges.
- [x] 2.2 Tests: the shader's constant is the core's; no level room face lies on a widened edge at Holbrook at the game's radius (fails without the slack).

## 3. Verify
- [x] 3.1 Before-and-after shots in `docs/screenshots/roof-speckle/` with a README naming each place and flag.
- [x] 3.2 `cargo fmt --all -- --check`, the touched modules' tests, `openspec validate roof-speckle`.
- [ ] 3.3 The owner's look at the same spots in play (night and dusk, the newel house's upper floor); then sync the requirement into `openspec/specs/planet/light/spec.md` and archive.
- [ ] 3.4 Video for the batch (CLAUDE.md: screenshots stand in for now).
