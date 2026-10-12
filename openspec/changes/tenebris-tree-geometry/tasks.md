# Tasks

## 1. The geometry

- [x] Replace the three-box branch in `planet_surface.wgsl` with hex prisms on
      the cell's own corner rays: a trunk at 0.20 of the cell and two leaf
      layers at `0.65 + 0.35 * hash01(id, layer)`, each one metre tall, on the
      radial axis the walls already use. The leaf width is the reference's
      `block_hex_width` hash ported as written.
- [x] Take the trunk height from the BIOME, as Tenebris does, rather than
      from the material as this task first proposed: `3 + ((h >> 8) & 1) +
      extra`, with the reference's own extras (jungle 2, swamp 3, tundra 3,
      fields 0). That needed the biome in the record, which it now carries in
      the surface word beside the material; the density table moved onto it
      too, so both read one fact from one place.
- [x] The cold-scrub pine: one wood block and a leaf cone whose width sweeps
      0.95 to 0.15 over the run. Its run is eight or nine one-metre layers in
      the reference and is the same SPAN in the two layers the vertex budget
      carries, so the taper is two segments rather than nine - the one
      deliberate divergence in the shape.
- [x] Raise the foliage vertex budget from 108 to 198 and move the `DrawArgs`
      and the pinned count together. The regression's eligibility fixtures were
      rewritten at the same time: they pinned a snapshot of the old hash
      contract, and they now prove the RULE, four seeds straddling the four
      rates so each pair fixes one clause of it.

## 2. Prove it

- [x] Still frames with the measured height and canopy width in the write-up.
      Found on the way and worth more than the frames: every ground preset
      stands at the spawn, and the spawn is jungle at 87 m, so all of them
      photographed a closed canopy at the jungle's 45% - which is 2.9% of the
      sphere, while pasture is 36% and nothing could photograph it. `--view
      meadow` walks to the first pasture cell and stands in it.
- [ ] Measure the frame cost in a release scene at the spawn, where the forest
      is densest, and report it against the box tree. The vertex count nearly
      doubles per instance; whether that is visible is a number, not a guess.
- [x] `docs/tenebris-comparison.md`: the tree rows of the dimensional table,
      and a section measuring the new geometry against the reference's own
      frame at the same tile size, camera height, crop and zoom.
