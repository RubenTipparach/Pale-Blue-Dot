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

   The normalised height the fields sum to is well under one, so a "max
   height" knob is not a summit: the reference's 40 m reaches 34 with its
   uplift, and a knob is worth about half of itself at the top. Measured on
   the port over 100,000 directions, and then authored to the budget:

   | knob | Tenebris | ours | what it measures as |
   | --- | ---: | ---: | --- |
   | metres per unit of land height | 40 | 210 | summit 153 m |
   | metres per unit of ocean depth | 24 (x2 seeded) | 320 | floor -87 m |
   | land bias | 0.20 - 0.16 | 0.0 | land 49.5% (reference seeded 47.1%) |
   | `rocky_uplift_m` | 22 | 60 | |
   | `island_height_m` | 10 | 12 | |
   | `river_depth_m`, `river_max_elev_m` | 3, 14 | 3, 40 | |
   | `shore_band_m`, `beach_band_m` | 6, 2 | 6, 2 | |
   | `mountain_elev_m` | 24 (0.7 of the summit) | 105 | Mountains 0.5% (reference 0.9%) |
   | snow line, stone line | 22, 32 | 100, 140 | |
   | `mountain_snowcap_elev_m` | 40 | 150 | |

   The biome shares that come out, against the reference's seeded world:
   Ocean 49.5 (45.4), Beach 4.8 (15.7), Fields 36.0 (27.8), Desert 3.0
   (1.7), Jungle 2.9 (1.0), Swamp 0.1 (0.1), Mountains 0.5 (0.9), Tundra 3.1
   (7.3) percent. The beach is thinner because two metres of a 210 m scale is
   a thinner band than two of forty, and the tundra smaller because the same
   cold latitude covers the same cap and the reference's is snow to a lower
   line. Both are the reference's rules at this scale, not retunes.

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

## Decided by the owner

- **The height budget stands at 150 m summits** (with 80 m basins), not
  Tenebris's 40/24: heights are absolute metres and do not scale with a body
  sixteen times larger.
- **Scope is Tenebris's main body only.** Sequoia's variants (its thresholds,
  its river and island gates, its snow line) are a second planet's data and
  belong in per-body config when that lands, not in the generator. Every
  threshold in the table above is a field of one terrain config with units,
  so a second body is a second file.
- **The moisture field's scale** on a body sixteen times larger is measured,
  not guessed: at Tenebris's 1.6 a biome would be a continent here.
