# Design: the term, with nothing to drive it yet

The advection in `water.fs.glsl` line 224 moves the fbm sample point by
`-(tangent * flow.x + bitangent * flow.y) * time * 0.35`, scaled by how
horizontal the face is, and by `radial * time * flow_speed_falling` scaled by
how vertical it is. The port reproduces that with the tangent frame Tenebris
builds (`tangent_u` off X or Y, projected off the face normal), reading a
`vec2` flow per cell.

The flow lives in the per-cell record as two more `f32` in the `corners[5].w`
slot's neighbour, which is the one place the record has room without growing
past 128 bytes: pentagons never use corner five, and hexagons need its ray. So
it does not fit there, and the honest answer is a **separate storage buffer of
`vec2<f32>` per cell**, bound to the water pass only, zero-filled today. A
second buffer is the cost of not lying about the record.

Nothing on the CPU computes it. When `voxel-engine-foundation` supplies water
state, `flow_direction` is ported into `pbd-core` beside the other world rules
and this buffer is what it writes.
