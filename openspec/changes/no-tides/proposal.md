# Proposal: no tides

## Why

The owner, 2026-11-19: "Remove literal tide feature."

The "literal tide" is the sea's surface rising and falling with the moon:
3 m a day at one place (`tide_range_m: 3.0` in `water.ron`), built on
2026-11-15 as `ocean-environments` task 5.1 (`pbd_core::sea::tide_m`, the
moon's second Legendre term, added to the one sea function on the CPU and in
`sea.wgsl`). After this change the sea's surface stands at one level. The
waves still move on it: the waves are not the tide and stay as they are.

The tide was never saved. It was derived from the clock and the moon's
direction, so removing it changes nothing a save holds.

## What changes

- **The tide's code goes, not just its range.** `tide_m`, `tide_high_m`,
  `LocalSea::with_tide`, the `tide` lanes of `LocalSea`, `SeaGpu`, the app's
  `Sea` and `SeaNow`, `Surroundings::tide`, `sea.wgsl`'s `sea_tide` and the
  `tide` lane of its `SeaView` uniform, `water_deep.w` as "the tide at the
  camera", the `--tide high|low` capture flag, and the `tide_range_m` config
  field. Every reader of it measures from the sea's mean surface (the drawn
  sheet, half a metre under the sea's level) plus the waves, where it read
  the waves before. Design 2 lists each reader and what it does now.
- **The high-water line comes down to the sea's level.** It was the tide's
  highest less the sheet's offset, 1.5 m over the sea's level. It is now the
  sea's level itself, which stands half a metre over the drawn sheet: that
  half metre is the waves' margin at a shore, where a wave is held to 0.6 of
  the water's depth. The sheet's listing, the flooded-ground bit, the sea's
  reach and the eye's water read it (design 3).
- **An old `water.ron` still loads.** A file that still has `tide_range_m`
  is read and the value ignored, so an older config next to a newer game
  does not stop it starting.
- **Plans for more tide are dropped.** `ocean-environments` design 10 and
  task 5.1 say the tide was built and then removed at the owner's word;
  `dry-holes` task 4.1, "the tide as flow", is dropped, because there is no
  tide left to flow.

## What does not change

- **World generation.** No generator version moves, and no generator's code
  path or value is touched: terrain, biomes, the beach and its back, rock,
  trees as the generator places them (`pbd_core::flora`, `planet_gen`). Their
  digests stay as they are. Comments in `pbd_core::flora` that say "at any
  tide" are reworded at most.
- **Saves.** No format change. Nothing of the tide was ever stored.
- **The waves**, the swell and the sea state, the waterfall look, rivers and
  the flowing sea of `dry-holes`.

## What a player sees differently

- The sea no longer moves up and down a beach over the day. Its waterline is
  where the day's mean was.
- The strip of beach between the sea's level and 1.5 m over it, which the
  high tide covered, is dry ground now. The flooded-ground bit is no longer
  set there, so the plants and trees the flora rules already put on such
  ground can grow on it. What grows where is still the generator's own rule;
  only the line under which nothing grows moves down to the sea's level.
- An ice floe is laid where the water is deep enough for its ice at the
  sea's mean, where it needed that depth at low water: a few floes appear
  in water a metre shallower than before.
- A generated hollow behind a beach that stands over the sea's level (a
  beach a metre up) is no longer reached by the sea's surface, so no sheet is
  drawn over it: it was the tide's, and there is no tide. A hollow behind a
  beach at the sea's level keeps `ocean-environments` B22's sheet.

## Impact

- `pbd-core`: `sea.rs`, `vehicle/mod.rs`, `vehicle/tests.rs`,
  `examples/vehicle_probe.rs`.
- `pbd-app`: `sea.rs`, `config.rs`, `planet.rs`, `planet_terrain.rs`,
  `planet_column.rs`, `planet_water.rs`, `planet_lod.rs` (a test),
  `planet_contact.rs`, `walking.rs`, `audio/hooks.rs`, `fish.rs`,
  `fish/places.rs`, `fish/tests.rs`, `vehicles.rs`, `vehicles/place.rs`,
  `vehicles/harbour.rs`, `vehicles/launch.rs`, `water_flow.rs` (a comment),
  `desktop.rs`, `desktop/scene.rs`, `desktop/eye_path.rs`,
  `sea_gpu_tests.rs`, `shader_tests.rs` (a comment).
- Shaders: `sea.wgsl` (the uniform shrinks by its last `vec4`, in step with
  the Rust `SeaView`), `water.wgsl`, `planet_surface.wgsl`,
  `planet_visibility.wgsl`, `sky_atmosphere.wgsl` (a comment).
- Config: `assets/config/water.ron` loses `tide_range_m`.
- Specs: `world/water-environments` (the composite's waterline) and
  `player/fishing` (the line over the water) lose their tide wording.
- Frame time: not measured (a cloud session has no GPU). The sea's shader
  does one dot product less per vertex; nothing is expected to move.
