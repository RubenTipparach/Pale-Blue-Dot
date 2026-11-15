## 1. Core

- [x] 1.1 `sky_light::night`, the sky's night term as one function, read by `sky_radiance` and the far lights
- [x] 1.2 `cell_key::direction`: a key's lattice point, by the midpoint descent; a test against the lattice's corners and midpoints
- [x] 1.3 `Edits::revision`, bumped by every `set`
- [x] 1.4 `pbd_core::night_lights`: the config and its validation, `strength` (kind, night, near fade), `player_clusters`; tests: day/night/near, the threshold (11 lights none, 12 one), order independence, two bases apart stay two
- [x] 1.5 `flora`: a test that a smaller share keeps a subset of the glowers, about a quarter at 0.01 against 0.04; `GLOW_FLOWER_LEVEL` 6 -> 3

## 2. App

- [x] 2.1 `assets/config/night_lights.ron`, loaded and validated at startup
- [x] 2.2 `night_lights.rs`: the `NightLights` resource (sites from `WorldSites`, player clusters from the save's edits, rebuilt on a new revision), extracted to the render world
- [x] 2.3 `PlanetParams.night`, `night_lights`; the 64 strongest each frame
- [x] 2.4 `planet_surface.wgsl`: `far_night_lights`, added in `finish` before the haze, thinned by the cover; the flower head at `GLOW_HEAD_GAIN`
- [x] 2.5 `column.ron` `glow_flower_chance` 0.01 and the config default
- [x] 2.6 No `LATE_PIPELINE` in a capture's log

## 3. Show

- [x] 3.1 Before and after: the night side from 9 and 20 km over the home continent; a town at dusk from 3 km; the same spot by day; a deck over a town at night; a meadow of glowing flowers at night from the ground. `docs/screenshots/night-lights-from-space/` with a README
- [ ] 3.2 Frame cost: not measured here (the owner's perf sitting)

Notes (2026-11-14):
- 3.1's shots are in `docs/screenshots/night-lights-from-space/`. The video for the batch stays open (CLAUDE.md, screenshots stand in).
- The deltas stay in this change until the owner has looked; they are synced to `openspec/specs` at archive.
- `planet_gen::tests::version_4_*` and `version_5_*` fail on this Mac at `2c17e84` too (digests taken on the desktop); nothing here touches the generator.
