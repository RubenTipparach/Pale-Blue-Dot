# Tasks

- [x] 1. `sky_atmosphere.wgsl`: over the ground (the ray reaches the sea sphere) from outside the shell, the hand-over weight `w = 1 - exp(-(h - top) / 1050)` on the dome's light and opacity
- [x] 2. The same rays: the Rayleigh term's gain `mix(1 / 4 pi, 1, smoothstep(a0, 0.94, opacity))`
- [x] 3. `a0` chosen from stills at 1-3 km and from orbit (the limb meets the sky above it): 0.3, the first trial, kept (`GROUND_GAIN_FROM`)
- [x] 4. The before and after ladder at the owner's spot (600-2000 m, 3000 m, 9000 m), its contact sheet and the ground patch's numbers in `docs/screenshots/high-haze/`
- [x] 5. `cargo fmt --all -- --check`, `openspec validate high-haze`
- [x] 7. The limb follow-up: the hand-over lifted toward the limb, shots in `docs/screenshots/high-haze/limb-*`
- [x] 8. The owner's 200 m hand-over to the original blue tint, `handover-200m.jpg`
- [x] 9. Thinner from far away, `far-thinned.jpg`
- [ ] 6. The step's video (held for the batch, `CLAUDE.md`)
