# Tasks

## 1. Measure

- [ ] The current relief range and land fraction on the 4,800 m body.
- [ ] The reference's land fraction, height histogram, island and river
      counts off `planet_gen.rs` at its default seed and at one seeded world.

## 2. Port

- [ ] `pbd-core::terrain::noise`: gradient noise seeded per octave, pinned
      against the reference at one seed and one direction.
- [ ] `pbd-core::terrain::altitude`: continent, mountains, hills, detail, the
      ocean power curve, islands, rivers, the shore easing, the rocky uplift,
      with every threshold a field of a terrain config with units.
- [ ] `pbd-core::terrain::biome` and the top-block rule.
- [ ] `planet_terrain` calls it; the generator version moves.

## 3. Prove

- [ ] The tests in the design: land fraction, crisp coasts, rivers reach the
      sea, mountains stand on land, the existing shore and swim tests.
- [ ] Captures beside today's and beside the Tenebris frames.
- [ ] The owner's eye in the running game.
