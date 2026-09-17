# Tasks

## 1. Night-side rim floor
- [x] Replace the bare `daylight` multiplier on the rim term in
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

## 3. Restore the one faithful water renderer
- [x] Bind `water.wgsl` through a dedicated live pipeline with the scene-colour
      and scene-depth inputs required by its refraction and path-length terms.
      It draws inside the `water-composite` node, after the compose pass, into
      the post-process destination, reading the source; no depth attachment.
- [x] The vertex stage pulls the cap from the persistent `Cell` record, the way
      the terrain does, instead of expecting a water vertex stream that was
      never built: the visibility compute emits a third list of water cells
      and a third indirect draw.
- [x] Fix the two one-line omissions measured against `water.fs.glsl` before
      binding: the per-term foam weights (`foam_crest_weight` 0.55,
      `foam_slope_weight` 0.26) and the specular `sun_tint`.
- [x] Water cells draw their **seabed** in the terrain pass at `R + height`
      with real neighbour heights in the corner record, and the terrain
      shader applies the port's underwater absorption to submerged fragments,
      so refraction and absorption have a floor to see.
- [x] Remove the inline water approximation from `planet_surface.wgsl` so there
      is no second live water look or wave model.
- [x] The cap's knobs are `assets/config/water.ron`, serde over RON with
      `#[serde(default)]`, the format `per-body-rendering` decided; the Rust
      defaults are the one source and a test loads the shipped file.
- [x] Validate the shader and its actual render-graph bindings, including the
      scene texture/depth layouts; a parser-only check is insufficient.
- [ ] Capture fixed-camera, fixed-time reference frames above and below the
      surface and a crossing sequence. Compare wave phase and normals, Fresnel,
      refraction, foam, absorption, colour and geometry continuity with the
      faithful port before asking the owner to accept visual parity.
      Captured above (`shore` at five heights), below (`dive`) and in the
      band (`wade`) on lavapipe; three look findings are recorded in
      `docs/tenebris-comparison.md` under "Status after implementation" and
      the owner has not yet judged them. The two water requirements in this
      change's spec delta stay here until that judgement.

## 4. Hold the gold standard

The hex size is decided: 2.833 m tile, 1.000 m cell height, off Tenebris's main
planet. What is open is the radius, since the two are locked by
`R = 300 m * 2^(L - 7)`. See `proposal.md` for the ladder and its costs.

- [x] Adopt the handoff's settled radius: 4,800 m at level 11 underfoot. The
      implementation depends on hexagon LOD; a uniform whole-globe level 11 is
      not the selected implementation.
- [ ] `PLANET_RADIUS` and `SUBDIVISIONS` move together to that pair, and
      `ELEVATION_STEP` goes from 6 m to 1 m.
- [ ] **Cut the relief to ~100-150 m peaks.** Measured by sampling
      `surface_height` over 400,000 directions, the current terrain runs
      **-516 m to +432 m** - 948 m of relief, about 10.8% of the 4,000 m radius.
      The target makes mountains climbable rather than scenery, and it decides
      how many one-metre layers a column needs once the voxel engine lands.

      Reference point: Tenebris, the project taken as definitive for hex size and
      gravity, uses a **128-layer column with sea level at index 64** -
      `MAX_LAND_HEIGHT` 40 m above sea, `MAX_OCEAN_DEPTH` 24 m below, reaching
      ~62 m at worst on a seeded world. So ~100-150 m peaks are roughly twice
      Tenebris's column budget in absolute terms, and far less than the ~640 m
      that scaling its proportions to a 4,800 m body would give. Size our column
      against the chosen figure, not against the old relief.
- [ ] Scale the dependent numbers in the same change: the atmosphere shell and
      cloud layer (currently `PLANET_RADIUS + 800` and `+ 600`, placed to clear
      the old +426 m peaks - at ~150 m they come down with the terrain), the
      2,300 m foliage range, and the 3,200 m draw-budget switch, which is set
      above `2300 + max_peak` on purpose so the cutoff never clips a tree that
      would have been drawn.
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

## 5. The topology record, if a bigger body is wanted
- [ ] 92 of the 128 bytes per cell are pure topology (direction, six corner
      rays), identical for every body at a level, and each corner ray is shared
      by three cells. Store corners once and index them.
- [ ] Measure the result. It should reach roughly 50 bytes a cell, which buys
      about one level on the ladder.
