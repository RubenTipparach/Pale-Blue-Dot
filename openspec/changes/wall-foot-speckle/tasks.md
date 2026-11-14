# Tasks

## 1. Write-up
- [x] 1.1 Reproduce in a capture and bisect by switches (design 1-2).
- [x] 1.2 Proposal, design, spec delta.

## 2. Fix
- [x] 2.1 `LAYER_SNAP_M` and `sample_layer` in `planet_surface.wgsl`; `lamp_near` and `bounce_near` sample through it.
- [x] 2.2 Test: both samplers use the helper and the constant; a floor's f32 sample at the game's radius straddles a whole layer and lies within the snap.

## 3. Verify
- [x] 3.1 Before-and-after crops in `docs/screenshots/wall-foot-speckle/` with a README naming each place and flag.
- [x] 3.2 `cargo fmt --all -- --check`, `shader_tests`, `openspec validate wall-foot-speckle`.
- [ ] 3.3 The owner's look at the same spots in play (night and dusk, Holbrook's lane and the coast); then sync the requirement into `openspec/specs/planet/light/spec.md` and archive.
- [ ] 3.4 Video for the batch (CLAUDE.md: screenshots stand in for now).
