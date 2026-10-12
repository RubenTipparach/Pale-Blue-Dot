# Design: hex prisms on the cell's own corners

## The record already carries everything

`GpuCell` holds the cell's direction and its six corner rays, which is exactly
what a hex prism needs: `shrink_corner` in the reference is
`center + (corner - center) * width`, and here the corners are unit rays, so the
same lerp on the rays followed by a scale to the radius gives the prism. The
terrain walls in `planet_surface.wgsl` already build quads from consecutive
corner rays at two radii; a tree part is the same construction at a shrunk
corner set and a one-metre radial span.

A pentagon has five corners, and the degree is in the record's low byte. The
prism follows the degree like the cap and the walls do; nothing special is
needed for the twelve.

## The vertex layout

Per instance, in the foliage draw, after the terrain's 60:

| part | vertices | what |
| ---: | ---: | --- |
| trunk sides | 36 | 6 quads, 2 triangles each |
| trunk top cap | 18 | 6 triangles from the centre |
| leaf layer 1 | 72 | 36 sides, 18 top, 18 bottom |
| leaf layer 2 | 72 | as above |
| **total** | **198** | |

A pentagon collapses the sixth quad and the sixth cap triangle to degenerate
triangles, the way the existing cap and wall path already does, so the count is
fixed and the draw stays indirect with no second argument set.

The trunk's own bottom cap is never drawn: it sits on the terrain cap. Its top
cap is drawn, because the leaf prism above is narrower than the cell and does
not cover it, which is the same rule the reference states as "wood is
non-covering".

## What the hash decides

One hash of the stable cell ID, already in the record, split into fields the way
Tenebris splits `tile_hash`:

- the density roll, which the visibility pass already takes as `hash(id) & 0xff`
- `(h >> 8) & 1`, the trunk's extra metre
- `(h >> 16)` and `(h >> 24)`, a byte per leaf layer for its width

Splitting one hash rather than hashing per part keeps the trunk and the crown
from drifting apart if one of them is ever re-rolled.

## What is deliberately NOT ported

- **The voxel column.** Tenebris plants wood and leaf blocks because it has a
  volumetric world and a player who can chop them. Here a tree is cosmetic
  geometry on a heightfield, so the shape is ported and the storage is not.
  When the voxel engine lands the storage question comes back with it.
- **Vines, mushrooms and redwoods.** Those are Sequoia's roster and a swamp
  extra. This preview has one planet and three vegetated materials.
- **Collision.** The reference makes tree voxels walk-through outside a 0.75 m
  trunk cylinder, to work around a bug its own rules file still lists as open.
  Our trees are not in the contact path at all and should stay out of it.
