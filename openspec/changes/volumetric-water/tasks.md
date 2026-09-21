# Tasks

## 1. Measure
- [x] `water_below_sea_in_the_spawn_tier`: the dry-cave and dry-tube
      columns and layers in the spawn tier. Numbers in the design.

## 2. Water the renderer and the walker can see
- [x] `WATER` (13) in `render_code`, its own code at last; the tile test
      holds that water has NO tile, since water draws no face, and that it
      no longer shares air's code.
- [x] The column record carries the top of its water where the shader reads
      it (`more[3]`, nine bits between the rim flag and the torch);
      `state_word` packs the column's half of that word for the build and
      the repack alike, so an edit cannot disagree with a build.
- [x] `planet_surface.wgsl`: the submerged term takes the column's own water
      surface inside the tier, capped at the sheet, and the radius rule
      stands outside it. `column_water_top_m` reads the record; a column
      with no water has its surface a thousand kilometres down, so the
      reader is one `min` and never a branch.
- [x] `submersion` asks the column at the eye: `EyeWater::Air` is dry at any
      depth, `Water` is read against ITS OWN surface, `Unknown` is the height
      field. `publish_eye_water` answers it in the main world, where the
      authoritative columns are, and the render world reads the extracted
      resource.
- [x] `Column::water_surface` and `Contact::water` in the core;
      `PlanetContact::stand` takes `water_depth` from them inside the tier.
      Tests: a seabed column reports the water standing on it, a cave under
      land below sea level reports none, and the same cave flooded reports
      the pool's own surface.
- [x] `--view seacave`: the spawn moves to the shore (`nearest_ground_near`,
      the locator the measuring test uses, so the picture and the numbers
      are taken of one place) and the camera picks a chamber whose roof is
      below sea level. `docs/screenshots/cave-below-sea-before-after.png`:
      blue off 99.04% of the frame, red and green untouched; and a frame
      taken in a seabed nook instead is byte identical before and after, so
      the sea did not move.

## 3. Flooding by connectivity
- [ ] The bounded flood at `column::build` and on an edit; tests for a
      tunnel into the sea (floods) and a sealed pocket (dry).

## 4. Flow
- [ ] Held for the fluid state: levels, spread, sources, the sheet at a
      cell's level.
