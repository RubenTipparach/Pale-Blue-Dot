# Tasks

The design is large and mostly unbuilt. These are its natural seams, ordered so
each one is verifiable before the next begins. None is scheduled; a task taken
up should become its own smaller change with its own specs.

## 1. Chunk store and addressing
- [ ] `ChunkKey = (body_id, topology_version, surface_patch, radial_slab)` as a
      core type, with the lifecycle states from the design's state diagram.
- [ ] Occupancy bitsets, palette/RLE columns, revision counters and neighbour
      halos, as a chunk-store resource with task ownership.
- [ ] Bedrock as an implicit lower bound and a positive per-body inner radius,
      so a radial prism can never cross the centre or invert.

## 2. Generation into chunks
- [ ] Independent random streams from
      `(world_seed, body_id, generator_version, feature_tag)`.
- [ ] Low-frequency climate and drainage fields body-wide; expensive cell
      materialisation per requested patch only.
- [ ] Versioned chunk jobs carrying `(chunk_key, revision, origin_epoch)`, with
      a late job unable to resurrect an evicted chunk.

## 3. Durable edits
- [ ] Every accepted mutation enters the durable transaction path immediately.
      A queued write is not a save; acknowledge only after the backend succeeds.
- [ ] Edited radial slabs stored densely only where an edit exists.
- [ ] Migration or continued old-generator support when the generator version
      moves. Never silent regeneration beneath a player's edits.

## 4. GPU face extraction and light
- [ ] Bind `hex_faces.wgsl` and `voxel_light.wgsl` to real pipelines, with the
      capacity and overflow protocol the design specifies: publish a complete
      generation or none.
- [ ] Cross-chunk light exchange; freeze old valid lighting rather than showing
      a partial rebuild.

## 5. Collision against voxels
- [ ] Streamed collider replacement, cave ceilings, overhangs and chunk seams.
- [ ] Contact accuracy held to the `planet/contact` capability's existing
      requirements at the new resolution.

## 6. Tiers and budgets
- [ ] Capability tiers negotiated from observed adapter limits.
- [ ] Reproducible release-scene measurement reporting hardware, resolution,
      seed, warm-up, percentiles, upload bytes and memory, against a kept
      baseline binary.
