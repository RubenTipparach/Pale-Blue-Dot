# Tasks

Step 2c of the owner's plan. The layers are shown first in the `world-map`
mockup. The Rust starts after the map screen and its layer registry exist
(`world-map` groups 3 and 5).

## 1. On the mockup

- [ ] 1.1 The mockup's fish layer from the `fish_ranges` fields and the `fauna.ron` rules: a species picker, "now" at a chosen day, and the all-year and part-year ranges. Verify: a headless screenshot per species, and the ranges match `docs/wiki/fish-ranges/` for the same run.
- [ ] 1.2 The mockup's climate layer: mean temperature, rain, sea ice and class, with a season slider, from the same run's fields. Verify: a headless screenshot per view.
- [ ] 1.3 **Gate:** the owner approves the two layers, and answers the open questions, the planet's cooling first. Record the words in `proposal.md`. Verify: the quote is present.

## 2. One rule for the spawner and the map

- [ ] 2.1 `fauna::can_spawn`, called by the spawner in place of its inline conjunction. Verify: the existing fauna tests pass unchanged, including `a_species_spawns_only_in_its_water_at_its_temperature` and `on_day_one_no_open_water_is_without_a_species`.
- [ ] 2.2 The static class-and-depth raster, with river pixels max-pooled, built with the base map; and the "now" mask built on the pool, timed. Verify: the build time is recorded in the risk note, and a test that 10,000 seeded water points get the same answer from the mask and from `can_spawn`.

## 3. The climate asset

- [ ] 3.1 `pbd_core::climate`: the format, the header, the provenance check and `sample(plane, direction)`. Verify: core tests of a round trip, a refused seed, a refused generator version, and a stale digest reported as stale.
- [ ] 3.2 The instrument bakes a year and writes `assets/climate/tenebris-v<generator>.bin`, with a `--check` mode. Verify: `--check` passes against the committed asset, and the run's log is committed next to it.
- [ ] 3.3 The fast staleness test against `assets/config/atmosphere.ron`. Verify: it fails when one setting is changed in a scratch copy, and names the re-bake command.
- [ ] 3.4 `tools/fish_ranges.py` reads its rules from `fauna.ron` and its fields from the same run. Verify: `docs/wiki/fish-ranges/` is regenerated with no change in the species table.

## 4. The layers in the game

- [ ] 4.1 The fish layer in the map's registry, with a species picker in the legend and both ranges. Verify: an app test that the perch marks no open sea and that frozen water marks nothing. Captures in `docs/screenshots/climate-and-fish-maps/`.
- [ ] 4.2 The climate layer with its four views and the season control, and "unavailable" and "stale" states in the legend. Verify: an app test for each state, and captures of each view.
- [ ] 4.3 The field guide's range thumbnail and "show on map". Verify: an app test that the action opens the map on the fish layer with that species.

## 5. The owner's check

- [ ] 5.1 Sync `world/climate` and the `player/map` additions into `openspec/specs`, naming each test. Verify: `openspec validate --all`.
- [ ] 5.2 The owner accepts the layers against the mockup. Verify: the quote is in `proposal.md`. Archive.
