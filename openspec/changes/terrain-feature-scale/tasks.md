# Tasks

## 1. Measure (done)

- [x] The roughness instrument: over land, the share of adjacent cells whose
      quantised caps differ and by how much, plus the coarsest and finest
      feature each term carries. An ignored test in `pbd-core`.
- [x] The same numbers off the reference generator, by a scratch example
      against the built `tenebris-core` (not committed): 44.0% of neighbours
      step, mean 0.56 m, finest feature 11.7 m.

## 2. Decide

- [x] The owner picked from labelled renders of three candidates: the FINEST,
      which is the reference's own 188 m moisture scale. Ground roughness was
      identical across all three and is not a choice.

## 3. Build

- [x] The two kinds in `TerrainConfig`: `continent_scale` and `rocky_scale`
      stay unit-sphere frequencies; `mountain_m`, `hill_m`, `detail_m`,
      `river_m` and `moisture_m` are feature sizes in metres and
      `scale_of` derives the frequency from `radius_m`.
- [x] The land-scale amplitudes in metres (`hill_amplitude_m` 6,
      `detail_amplitude_m` 2), divided back out of the budget at the point of
      use. `land_scale_m` did not need re-measuring: the summit stayed inside
      the 130-180 m the test pins.
- [x] `radius_m` is pinned to `PLANET_RADIUS` by a test, since two places now
      hold the body's size.
- [ ] The tuning numbers into a validated RON asset with a test pinning the
      shipped file to the code defaults, as `water.ron` is. Still worth doing
      and still not done: this change was a rebuild per candidate.
- [ ] `GENERATOR_VERSION` moves.

## 4. Prove

- [x] The roughness report against the reference's numbers, in the write-up,
      and as a real test: `the_ground_is_as_rough_as_the_reference` pins a
      third of neighbours stepping and the finest feature under seven cells.
- [x] `a_kilometre_of_land_crosses_more_than_one_biome`, which is what the
      moisture decision was about.
- [x] `meadow`, `coast`, `orbit` and `surface` captures, and a river at ground
      level through a new `--view river` that finds one by asking the
      generator's own carve rather than guessing from heights.
- [x] The existing suites pass. Two changed their ASSUMPTIONS, both recorded
      in the comparison doc: the shore probe took one sample 100 m out, which
      a real coastline can put back on dry land, and the fall test allowed one
      tick where contact detection costs two at 17 m/s.
