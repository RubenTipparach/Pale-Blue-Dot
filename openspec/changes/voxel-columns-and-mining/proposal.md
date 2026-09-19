# Proposal: voxel columns, so the world has an inside

## Why

**The owner asked to walk into caves and overhangs like Minecraft, and to dig
and place blocks.** Both are the same blocker, and it is structural rather than
a matter of effort: `SurfaceContact` carries one `radius` per direction, so the
world is a heightfield. A heightfield has exactly one surface per direction. It
cannot express a ceiling, it has no interior to enter, and it has no block to
remove - lowering a column is not mining, it is erosion.

## The cheap way in, which is not the standing design

`openspec/changes/voxel-engine-foundation` is the repo's answer to this and it
is large: streamed radial chunk slabs, a chunk store with lifecycle states,
versioned jobs, GPU face extraction, cross-chunk light exchange, negotiated
capability tiers. It is explicitly "the standing design, not scheduled work".

**The reference does not do that.** `tenebris-core/src/world.rs` stores
`blocks[tile * WORLD_MAX_DEPTH + depth]` - one byte per layer, 128 layers per
tile, one flat array. That is volumetric: a column can be air at one layer and
solid at the next, so a ceiling, an overhang and a dug-out cave all fall out of
the representation rather than being features built on top of it.

On this body that is affordable, and the arithmetic is the whole argument.
Columns are needed only where a player can reach them, which is the **finest
tier**: a 45 m base tile is not something anyone digs. The finest level holds at
most `FINE_CAPACITY` of 65,536 records, and the measured relief runs -125 m to
+158 m, so a column spanning the whole relief at the shipped 1 m layer is about
320 bytes:

| | |
| --- | ---: |
| Finest-level records, at capacity | 65,536 |
| Layers per column, covering -145 to +175 m | 320 |
| **Column storage** | **21 MB** |
| The record buffer it sits beside, today | 59.2 MB |

So the thing that unlocks caves, overhangs, digging and placing costs about a
third of what the terrain records already cost, and needs no streaming
machinery. The standing design remains the right answer for a planet dug to its
core; it is not what is needed to walk into a cave.

## What "like Minecraft" needs that the reference does NOT have

Worth separating, because it decides how much is a port and how much is ours:

- **Overhangs and dug-out caves come free with columns.** They are the
  representation, and the reference's own comments about walking under a cave
  roof are about exactly this.
- **Natural cave systems are NOT in the reference.** Its `gen_column` fills
  every layer from a per-column profile - solid below the surface, air above -
  and carves nothing. Every cave in Tenebris is one a player dug. A Minecraft
  world's caves are generated, so that is an addition of ours: a 3D field
  sampled down the column, which this project's noise already supports because
  `gnoise3d_seed` takes a point rather than a direction.

## What changes

- **`pbd_core::column`**: a column of materials over a fixed radial span, built
  from the existing `surface_altitude` for its top and carved by a 3D field for
  its caves. The generator stays the one source of the surface, so the terrain
  does not fork into a heightfield version and a column version that drift.
- **Collision against columns**: `SurfaceContact` grows from one radius to the
  solid RUN a point is in or above - the floor below you and the ceiling over
  your head. This is where walking into a cave actually happens.
- **Rendering the exposed faces**: the surface shader draws a cap and walls for
  each solid run rather than one cap at one height.
- **Mining and placing**: set a layer, re-upload that record, and write the edit
  to durable storage **on the same input frame**, which is this repo's
  non-negotiable rule and the reason a dug cave survives a reload.

## The tiers stay split, which is what keeps it affordable

Columns are a **finest-tier** feature. The coarse tiers keep the heightfield
they already draw, exactly as ground clutter is a finest-tier feature and the
LOD bands are unchanged by it. A player digs where they stand; a hillside twelve
kilometres away is a surface, and drawing it as one is correct rather than a
compromise.

## Non-goals

- The streamed chunk store, versioned jobs and capability tiers of
  `voxel-engine-foundation`. Still the standing design; still not this.
- Baked voxel light. The existing sky occlusion is what a face is lit by until
  a cave is dark enough to need its own answer.
- Multiplayer, and any edit protocol beyond writing this player's edit durably.
