# Proposal: the flow hook, and why the field behind it is empty

## Why

Two of the five cap-pass terms the port is missing are flow: the noise sample
point advected along a per-vertex flow vector (`v_flow_uv`), so a river
streams, and a radial scroll on vertical water faces at
`flow_uv_speed_falling`, so a waterfall falls. In Tenebris the vector comes
from `world_water.rs`, a 1,955-line per-voxel fluid state with a scheduler,
saved and networked, and `flow_direction` derives it per tile for the mesher.

This project has no water voxels: terrain is a heightfield of columns and the
volumetric engine is `voxel-engine-foundation`, designed and not built. There
are also no rivers in the generator, so a flow field derived from anything
would be a picture of a world that does not exist.

## What changes

The shader terms are ported so the cap pass is Tenebris's term for term, and
each water cell carries a flow vector the cap reads. The vector is **zero for
every cell**, and the reason is recorded here rather than in a comment: it is
zero because the world has no rivers, not because the term is unfinished.

## What unblocks the rest

`voxel-engine-foundation` for the water state, and a river pass in the
terrain generator for somewhere for it to flow. When either lands, the field
is filled and nothing in the cap pass changes.
