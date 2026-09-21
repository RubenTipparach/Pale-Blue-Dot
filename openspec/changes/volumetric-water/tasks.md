# Tasks

## 1. Measure
- [x] `water_below_sea_in_the_spawn_tier`: the dry-cave and dry-tube
      columns and layers in the spawn tier. Numbers in the design.

## 2. Water the renderer and the walker can see
- [ ] `WATER_CODE` in `render_code`; the tile-name test learns a faceless
      material; the constants test pins the code.
- [ ] `planet_surface.wgsl`: the submerged term gated by the air-side layer's
      material in the tier; heightfield rule off the tier. A cave capture
      (`--view cave` and the owner's picture) before and after.
- [ ] `submersion` asks the eye's column first; tests at a dry cave eye and
      a sea eye.
- [ ] `Column::contact` answers the water run; `PlanetContact::stand` sets
      `water_depth` from it in the tier; a test that a walker in a cave
      below sea level does not swim.

## 3. Flooding by connectivity
- [ ] The bounded flood at `column::build` and on an edit; tests for a
      tunnel into the sea (floods) and a sealed pocket (dry).

## 4. Flow
- [ ] Held for the fluid state: levels, spread, sources, the sheet at a
      cell's level.
