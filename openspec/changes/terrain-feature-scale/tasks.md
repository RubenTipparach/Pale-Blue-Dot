# Tasks

## 1. Measure (done)

- [x] The roughness instrument: over land, the share of adjacent cells whose
      quantised caps differ and by how much, plus the coarsest and finest
      feature each term carries. An ignored test in `pbd-core`.
- [x] The same numbers off the reference generator, by a scratch example
      against the built `tenebris-core` (not committed): 44.0% of neighbours
      step, mean 0.56 m, finest feature 11.7 m.

## 2. Decide

- [ ] The owner picks between the candidates rendered from this design: how
      rough is right underfoot, and how big a biome patch should be.

## 3. Build

- [ ] The two kinds in `TerrainConfig`: a scale that is authored and one that
      is derived from the radius and a feature size in metres.
- [ ] The land-scale amplitudes in metres rather than as a share of the summit,
      with `land_scale_m` re-measured to hold 150 m.
- [ ] The tuning numbers into a validated RON asset with a test pinning the
      shipped file to the code defaults, as `water.ron` is.
- [ ] `GENERATOR_VERSION` moves.

## 4. Prove

- [ ] The roughness report against the reference's numbers, in the write-up.
- [ ] `meadow`, `coast` and `orbit` captures before and after, and a river
      photographed at ground level, which no preset can do today.
- [ ] The existing suites: the budget and land fraction, mountains on land,
      rivers reaching the sea, the shore walk and the swim.
