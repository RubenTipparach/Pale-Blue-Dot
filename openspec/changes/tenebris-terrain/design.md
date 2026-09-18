# Design: porting the generator, and what has to be re-authored

## The port, term by term

The altitude function is ported as written, with these deliberate changes:

1. **The noise primitive.** Tenebris's `gnoise3d_seed` is 3D gradient noise
   with the seed folded into the lattice hash and stirred per octave (`seed +
   i * 1117`). Ours is value noise with one hash. The primitive is ported too:
   value noise is blobbier at the same octave count, and "blobby" is half of
   the complaint. It goes in `pbd-core` beside the terrain, with a pinned
   sample so the port can be checked against the reference bit for bit at one
   direction.

2. **Scales stay on the unit sphere; heights are re-authored.** Every scale in
   the table is a frequency on the unit sphere, so a continent that is a
   quarter of Tenebris's 300 m body is a quarter of this 4,800 m one: the same
   map at sixteen times the size, which is right for a body sixteen times the
   size. Heights are absolute metres and do not scale: Tenebris's 40 m peaks on
   a body a walker can cross in ten minutes would be a plain here. The budget
   settled in `hexagon-lod` stands: **summits near 150 m**, and the sea keeps
   more of its range than the land so a swimmer can submerge (the
   `OCEAN_RELIEF` finding from the water work).

   | Tenebris | ours |
   | --- | ---: |
   | `MAX_LAND_HEIGHT` 40 m | 150 m |
   | `MAX_OCEAN_DEPTH` 24 m | 80 m |
   | `ROCKY_UPLIFT_M` 22 m | 60 m |
   | `ISLAND_HEIGHT_M` 10 m | 12 m |
   | `RIVER_DEPTH_M` 3 m, `RIVER_MAX_ELEV_M` 14 m | 3 m, 40 m |
   | `SHORE_BAND_M` 6 m | 6 m |
   | `BEACH_BAND` 2 m | 2 m |
   | `MOUNTAIN_ELEV_M` 24 m | 70 m |
   | `MOUNTAIN_SNOWCAP_ELEV_M` 40 m | 120 m |
   | snow line 22 m, stone 32 m | 90 m, 110 m |

   Bands that a walker experiences at their own scale (a beach, a river's
   depth, a shore's easing) keep Tenebris's metres. Bands that are a fraction
   of the peak height scale with the peak.

3. **Seeded from the world seed, always.** No `biomes_active` gate; the
   extensions are the generator. The seed is the world's, and the generator
   version is in the world ID.

4. **The relief multiplier goes.** With the heights authored in metres against
   the budget there is nothing to cut. The quantisation to one-metre steps
   stays: it is the voxel column, not a look.

## What is measured before any of it is written

- The current relief range over 400,000 directions (was -516/+432 m on the
  4,000 m body; the rescale's own number is recorded in its change), so the
  port has a before.
- The reference's distribution: land fraction, height histogram, island and
  river counts on the main body at its default and at one seeded world, off
  the pure function, so the port can be held to the same distribution at this
  body's scale rather than to a picture.

## What proves it

- A pinned sample of the noise primitive against the reference at one seed
  and one direction.
- Land fraction within a few percent of the reference's seeded world.
- Coasts: the fraction of shoreline cells whose land neighbour is under the
  beach band, which is what "crisp" means numerically.
- Rivers reach the sea: a channel cell's downhill walk ends below sea level.
- Mountains stand on land: no cell above `MOUNTAIN_ELEV_M` within N cells of
  the sea.
- The existing tests hold: the sphere still has substantial land and ocean,
  the shore walk still finds a coast, the swim still submerges.
- Captures: `coast`, `surface`, `orbit` and the shore, beside the same views
  today, and beside the Tenebris frames in `docs/screenshots/tenebris-*.png`.

## Open

- **Whether `Sequoia`'s variants port.** Tenebris carries a second tileset's
  thresholds through the same function behind `tileset ==` checks. They are a
  second planet's data and belong in per-body config (`per-body-rendering`),
  not in the generator. Recommend: port the Tenebris main body only, with
  every threshold in the table above a field of a body's terrain config.
- **The moisture field's scale** on a body sixteen times larger: at Tenebris's
  1.6 a biome is a continent here. Likely 3 to 4; measured, not guessed.
