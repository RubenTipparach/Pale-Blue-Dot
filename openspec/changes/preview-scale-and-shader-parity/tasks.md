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

## 3. Hold the gold standard

The hex size is decided: 2.833 m tile, 1.000 m cell height, off Tenebris's main
planet. What is open is the radius, since the two are locked by
`R = 300 m * 2^(L - 7)`. See `proposal.md` for the ladder and its costs.

- [ ] Owner picks a radius from the ladder. 600 m holds the standard at today's
      exact level and memory budget; 1,200 m costs 320 MiB; 2,400 m needs the
      topology record shrunk first.
- [ ] `PLANET_RADIUS` and `SUBDIVISIONS` move together to that pair, and
      `ELEVATION_STEP` goes from 6 m to 1 m.
- [ ] Scale the dependent numbers in the same change: the 4,800 m atmosphere
      shell, the 4,600 m cloud layer, the 2,300 m foliage range, the 3,200 m
      draw-budget switch, and the terrain amplitude (positive relief currently
      peaks near 426 m, which is 71% of a 600 m radius and absurd on one).
- [ ] Re-tune the tree geometry in `planet_surface.wgsl`, which is authored for
      the old scale: trunks span 18 m and canopies reach 33 m, against roughly
      6 m for a Tenebris tree.
- [ ] Re-tune the atlas UV divisors, `/28.0` on the cap and `/18.` down a wall,
      which were chosen against a 19 m tile.
- [ ] Update the pinned tile width in
      `measured_tile_width_follows_the_subdivision_law_and_pins_the_shipped_scale`,
      move the `planet/scale` requirements from this change into
      `openspec/specs/`, and retake every capture in
      `docs/tenebris-comparison.md`, all in the same commit.

## 4. The topology record, if a bigger body is wanted
- [ ] 92 of the 128 bytes per cell are pure topology (direction, six corner
      rays), identical for every body at a level, and each corner ray is shared
      by three cells. Store corners once and index them.
- [ ] Measure the result. It should reach roughly 50 bytes a cell, which buys
      about one level on the ladder.
