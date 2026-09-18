# Tasks

## 1. Measure

- [x] The current relief range and land fraction on the 4,800 m body:
      land 51.6%, -232 to +144 m, over 200,000 directions.
- [x] The reference's distribution off `planet_gen.rs` (a scratch example
      against the built crate, not committed): default seed land 77.4%, -4.6
      to +36.8 m, Fields everywhere; seeded land 47.1%, -16.7 to +34.3 m,
      the biome shares in the design.

## 2. Port

- [x] `pbd_core::planet_gen::gradient_noise`, `fractal`, `ridged`: pinned
      bit for bit against the reference at four seeds and points.
- [x] `pbd_core::planet_gen::surface_altitude`: continent, mountains, hills,
      detail, the ocean power curve, islands, rivers, the shore easing, the
      rocky uplift, every threshold a field of `TerrainConfig` with units and
      one const of defaults, `TerrainConfig::TENEBRIS`.
- [x] `biome_at` and `top_material` (the reference's top-block rule, on the
      core's `Material`, extended by the blocks it hands out).
- [x] `planet_terrain` calls it and maps the material and biome to the
      surface shader's index; `GENERATOR_VERSION` is 2.

## 3. Prove

- [x] The tests in the design: the budget and land fraction, mountains stand
      on land (no sea within three cells of a peak), rivers reach the sea (a
      downhill walk from every channel sample ends below it), every biome
      and material occurs, the beach is sand; the existing shore, swim and
      walker suites pass. The one-metre-fall test now spawns at an inland
      cell's centre rather than wherever the generator's coast search lands.
- [x] Captures beside today's and beside the Tenebris frames, in the
      comparison doc ("The terrain generator, ported").
- [ ] The owner's eye in the running game.
