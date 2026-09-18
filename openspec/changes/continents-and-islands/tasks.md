# Tasks

## 1. Measure

- [x] Flood-fill the land on a level-6 dual sphere and report the connected
      masses. `landmass_report` in `planet_terrain.rs`: the shipped world is
      48.1% land in 28 masses, the largest holding 90.2% of it.
- [x] Sweep `continent_scale` against `land_bias` and report land fraction,
      mass count and the largest mass's share. `continent_sweep`, same file.
      It is what found that the land FRACTION governs the largest mass more
      than the frequency does - above about 45% land the sphere percolates and
      joins up however finely the field is cut.

## 2. Show the owner

- [ ] Render the three candidates from orbit with their numbers burned into
      the frame, the way the moisture scale was settled.
- [ ] Take the owner's pick.

## 3. Apply it

- [ ] The chosen `continent_scale` and `land_bias` in `TerrainConfig::TENEBRIS`.
- [ ] `GENERATOR_VERSION` to 4 in the same commit: it is part of saved world
      IDs and the coastlines have moved.
- [ ] A test that pins what was chosen - the land fraction inside a band, and
      the largest mass under a share of the land - so a later tuning cannot
      quietly restore the supercontinent.
- [ ] Re-check the distribution report: the biome shares move with the land
      fraction, and swamp was already at 0.1%.

## 4. Prove it

- [ ] Orbit, coast and a ground preset re-captured.
- [ ] `docs/tenebris-comparison.md`: the measured before and after.

## 5. Held

- [ ] The island FIELD (`island_scale`, `island_threshold`, `island_height_m`)
      only lifts ground already in shallow sea, so it makes atolls near coasts
      rather than mid-ocean chains. Reaching deeper water is a separate
      decision about what an island is, not a tuning of this one.
