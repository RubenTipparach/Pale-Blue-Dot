# Tasks

- [x] 1. Measure: before stills from orbit (sun above the rim, on it, through the air, half behind the limb, behind the planet), across the shell's top (900 m and 1050 m), on the ground at noon and sunset, in cloud (`docs/screenshots/sun-in-space/`)
- [x] 2. `sky_atmosphere.wgsl`: the sun's term as one function; the dome adds it only from inside the shell
- [x] 3. `sky_atmosphere.wgsl` under `SUN_DISC`: the camera-facing quad at 150 km, collapsed from inside; the fragment's disc, glow, shadow and the air's transmittance, faded in over 200 m past the shell's top
- [x] 4. `sky.rs`: `SunDiscMaterial` (the dome's parameters, copied each frame), its quad entity at the planet's centre, `depth_bias` 1 so it draws after the dome, `NoFrustumCulling`
- [x] 5. A test holding the dome's sun to the inside of the shell and the two draws to one sun function
- [x] 6. After stills, the same list; the ground and cloud shots unchanged; no new `LATE_PIPELINE`, no shader errors; README and contact sheets
- [x] 7. `cargo fmt --all -- --check`, `cargo test --profile fast -p pbd-app --lib`, `openspec validate sun-in-space`
- [ ] 8. The step's video (held for the batch, `CLAUDE.md`)
