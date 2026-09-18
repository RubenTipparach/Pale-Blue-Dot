# Tasks

## 1. The geometry

- [ ] Replace the three-box branch in `planet_surface.wgsl` with hex prisms on
      the cell's own corner rays: a trunk at 0.20 of the cell and two leaf
      layers at `0.65 + 0.35 * hash01(id, layer)`, each one metre tall, on the
      radial axis the walls already use.
- [ ] Take the trunk height from the material as Tenebris takes it from the
      biome: `3 + ((h >> 8) & 1) + extra`, with forest at Tenebris's swamp
      extension of 3, grass at fields' 0, and cold scrub as the pine.
- [ ] The cold-scrub pine: one wood block and a leaf cone whose width sweeps
      0.95 to 0.15 over the run, which is the mesher's only per-shape width
      override in the reference.
- [ ] Raise the foliage vertex budget from 108 to what the prisms need (198 for
      a capped trunk and two capped leaf layers) and move the foliage draw's
      `first_vertex`, the `DrawArgs` in `planet_visibility.wgsl` and the pinned
      counts in the GPU regression together.

## 2. Prove it

- [ ] A still frame at the spawn and one from the `seam` preset, before and
      after, with the tree's measured height and canopy width in the write-up
      rather than an adjective.
- [ ] Measure the frame cost in a release scene at the spawn, where the forest
      is densest, and report it against the box tree. The vertex count nearly
      doubles per instance; whether that is visible is a number, not a guess.
- [ ] `docs/tenebris-comparison.md`: replace the tree row of the dimensional
      table with the measured new geometry.
