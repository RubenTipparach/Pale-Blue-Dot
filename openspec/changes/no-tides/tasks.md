# Tasks

## 1. Write-up

- [x] 1.1 Proposal, design (every reader of the tide and what it does
  instead), spec deltas; `ocean-environments` and `dry-holes` stop planning
  tides. Verify: `openspec validate --all`.

## 2. Remove the tide

- [x] 2.1 `pbd-core`: `tide_m`, `tide_high_m`, the `tide` lanes of
  `LocalSea` and `SeaGpu`, `with_tide`, `Surroundings::tide`; `reach` is the
  waves'. Verify: `cargo test -p pbd-core`, the generator digests
  (`planet_gen`) unchanged.
- [x] 2.2 `sea.wgsl`'s `sea_tide` and its uniform lane, with the Rust
  `SeaView`; `planet_surface.wgsl`, `planet_visibility.wgsl`, `water.wgsl`.
  Verify: `sea_gpu_tests`, `shader_tests`.
- [x] 2.3 The app's `Sea` and `SeaNow`, `move_moon`, `--tide`,
  `tide_range_m` (an old file's key read and ignored), the high-water line
  at the sea's level (design 3), and every reader in design 2. Verify: the
  sea, walking, fish, vehicles, `planet::column`, `planet::water`,
  `water_flow` and audio tests; `an_old_water_ron_with_a_tide_still_loads`.
- [x] 2.4 Move the modified requirements into `openspec/specs` in the same
  commit, and there rename the two scenarios that still say "low tide" and
  "low water" ("A jump on a beach", "A wave trough and a bite") in the main
  specs and these deltas together (the validator holds a delta's scenarios
  to the current spec's names). Verify: `openspec validate --all`.

## 3. Captures

- [x] 3.1 The beach of `sea-hole-beach` at high tide before and with no
  tide after, side by side, in `docs/screenshots/no-tides/` with a README
  (lavapipe, a fixed step). Verify: the README.
