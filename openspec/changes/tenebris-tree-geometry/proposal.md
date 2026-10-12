# Proposal: a tree is Tenebris's tree, not three boxes

## Why

**Standing instruction from the owner: "we should use the same tree geometry as
Tenebris."** What this project draws instead is three axis-aligned boxes stacked
on a cell: a trunk box, a wide canopy box and a narrower one on top. It was
authored against the 19 m tile of the old 4,000 m body and was carried through
the rescale by one multiplier, `scale = (0.85 + random*0.50) * 0.3`, which
shrank it without changing its shape. Measured on that expression at the mean
roll, our tree is about **11 m tall with a 6 m canopy** on a 2.833 m tile: a
crown that overhangs its own cell by more than a cell on each side, and a tree
twice the height of the reference's.

Tenebris's is not a mesh at all, and that is the point of the port.

## What a Tenebris tree actually is

Measured off `tenebris-rs`, worldgen in `tenebris-core/src/world.rs` and the
mesher in `tenebris-client/src/hex_mesher.rs`:

- **It is voxels in ONE column.** The generator writes wood and leaf blocks into
  a single tile's 128-byte column and never touches a neighbour. A canopy that
  looks wider than the trunk is a *mesher* effect, not extra cells.
- **It is drawn as ordinary hex prisms, shrunk toward the tile centre.**
  `block_hex_width` returns **0.20** for wood and **0.65 + 0.35 * hash01(tile,
  depth)** for a leaf, and `shrink_corner` lerps each hex corner toward the
  centre by that factor. The radial extent, one metre, is never changed. So a
  trunk is a skinny hex prism about **0.57 m across** and a leaf layer is a hex
  prism of **1.84 to 2.83 m**, each layer a different width from its own hash,
  which is what makes a canopy irregular rather than stamped.
- **A broadleaf tree is N wood blocks and exactly TWO leaf layers.** The leaf
  loop only writes where the cell is air, so the two lowest nominal leaf cells
  are already wood and are skipped.
- **Height is per biome**, from `trunk = 3 + ((h >> 8) & 1) + extra_trunk`:

| biome | density /256 | extra trunk | wood | leaves | total |
| --- | ---: | ---: | ---: | ---: | ---: |
| Fields / Beach | 13 | 0 | 3-4 | 2 | **5-6 m** |
| Jungle | 115 | 2 | 5-6 | 2 | 7-8 m |
| Swamp | 34 | 3 | 6-7 | 2 | 8-9 m |
| Tundra pine | 2 | 3 | **1** | 8-9 | 9-10 m |

  A pine is the exception in shape as well as height: one visible wood block and
  a cone of leaf layers whose width sweeps **0.95 down to 0.15** over the run.

The densities are already ours: the last commit put forest at 34, grass at 13
and scrub at 2 out of 256, which are Tenebris's swamp, fields and tundra rates.
What was not ported is the geometry those rates place.

## What changes

`planet_surface.wgsl`'s foliage branch stops building boxes and builds hex
prisms on the cell's own corner rays, which the `Cell` record already carries:

1. **The trunk** is one prism at 0.20 of the cell, `3 + bit + extra` metres tall,
   where the extra comes from the cell's material as the density already does.
2. **Two leaf layers**, one metre each, at `0.65 + 0.35 * hash01` of the cell,
   hashed per layer off the stable cell ID so a canopy steps rather than stamps.
3. **The cone** for the cold-scrub material, as Tenebris's pine: one wood block
   and a tapering leaf run.

The vertex budget moves with it, and that is the cost to weigh: a closed hex
prism is 36 side vertices plus 18 per cap, so a trunk with a top cap and two
capped leaf prisms is **198 vertices** against the 108 the three boxes use. The
foliage draw is separate and indirect already, and only the three finest levels
within 1,200 m submit to it, so the change is bounded; it still wants measuring
in a release scene rather than asserting.

## What this does not change

Where trees stand. The eligibility rule, the per-material densities and the
level rule landed in the previous commit and are already Tenebris's.
