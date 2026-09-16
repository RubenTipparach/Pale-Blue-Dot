# Tasks

## 1. Night-side rim floor
- [ ] Replace the bare `daylight` multiplier on the rim term in
      `planet_surface.wgsl` with `0.25 + 0.75 * daylight`, matching
      `hex_terrain.wgsl` and Tenebris's `distant_rim_floor`.
- [ ] Capture the night and orbit views before and after; the owner confirms
      the limb reads at night without washing the terminator.

## 2. Lift the literals into the uniform
- [ ] Extend `PlanetParams` with rim colour, rim power, rim intensity, fog
      height, fog density, terminator band and the ambient and sun tints.
- [ ] Read them in `planet_surface.wgsl` in place of the inline constants.
- [ ] Keep the shipped values byte-identical on the first pass, so the capture
      set is unchanged and the diff is provably a refactor.
- [ ] Only then give a second tileset its own values, and pin the Rust and WGSL
      layouts against each other the way `pipeline_tests` already does.

## 3. The scale decision
- [ ] Owner picks: shrink the body, scale the avatar, or wait for the streamed
      near-player grid. The numbers for each are in `design.md`.
- [ ] If the body shrinks, move the atmosphere shell, cloud layer, foliage
      range, draw-budget switch and terrain amplitude in the same change, and
      retake every capture in `docs/tenebris-comparison.md`.
- [ ] Update the pinned tile width in
      `measured_tile_width_follows_the_subdivision_law_and_pins_the_shipped_scale`
      and the table it points at, in the same commit.
