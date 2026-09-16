# Proposal: hexagon LOD, decided and drawn on the GPU

## Why

Holding the gold standard (2.833 m tiles, 1 m cells) with ONE subdivision level
for the whole globe locks the radius to `R = 300 m * 2^(L - 7)`, and the
authored 4 km catalog then needs level 11: 42 million cells and 5 GiB of
topology. That is the wrong conclusion drawn from the right rule. The standard
is about the tile a player STANDS ON. Nothing about it says a tile on the far
side of the planet must be the same size, and a uniform level is what makes it
look as though it does.

The JS tree solved this and its answer is the one to take: **hexagons at every
distance**. `createDistantLODMeshes` builds three more `GoldbergPolyhedron`
instances at `subdivisions - 2`, `- 3` and `- 4` and swaps between them by
altitude. Coarser tiles, still tiles. `tenebris-c` and `tenebris-rs` gave that
up for a triangle icosphere impostor (`distant.fs.glsl`), and the seam between
a hexagon world and a smooth triangle ball is the thing the owner does not want.

## The property that makes it cheap

Measured, not assumed. The subdivision appends midpoints after the existing
vertices, so a vertex keeps its index forever:

```text
L2->L3: first     162 of     642 vertices identical, same index
L3->L4: first     642 of   2,562 vertices identical, same index
L4->L5: first   2,562 of  10,242 vertices identical, same index
L5->L6: first  10,242 of  40,962 vertices identical, same index
L6->L7: first  40,962 of 163,842 vertices identical, same index
```

`tenebris-rs`'s `goldberg.rs` states this as a contract it must never break,
because those indices appear in saves and wire data. Two consequences:

- **A coarse LOD tile is a PREFIX of the fine cell array.** Cell `i` is the same
  direction at every level fine enough to contain it. Nothing merges, nothing
  remeshes, and there is no separate coarse mesh to keep in step with the fine
  one.
- **A coarse tile's height is free.** Its centre IS a fine cell's centre, so its
  height is `heights[i]` out of the array that already exists. Coarse levels add
  no per-planet data at all.

What a level does need is its own corner rings, since a coarse cell's corners
are its own triangulation's centres rather than the fine one's. Summed over
every level that is `4/3` of the finest level alone: **all LOD tiers together
cost about 33% more topology than the finest tier by itself.**

## What changes

- Tiles are drawn at a level chosen from distance, hexagonal at every tier, out
  to the far LOD. The triangle impostor is not ported.
- **The whole path is one-way to the GPU.** Topology and heights upload once;
  the LOD level per tile, the cull and the draw arguments are all computed in
  the compute pass and consumed by the indirect draw. Nothing about a tile comes
  back to the CPU. `planet_visibility.wgsl` already does the cull and the
  compaction this way, so this extends one pass rather than adding a second
  system. The JS version builds its LOD meshes in a CPU worker and uploads
  them; that half is explicitly NOT what is being copied.
- The radius stops being locked to the level. A body carries the gold standard
  underfoot without holding its whole surface resident at that level.

## What this does not change

The CPU stays authoritative for terrain, edits, collision and persistence. LOD
is a drawing decision, so it is derived state and lives entirely on the GPU
side of that line. A GPU-chosen LOD level never decides where a player stands.

## The hard part, stated rather than hidden

Two LOD bands meet somewhere, and a coarse tile's footprint overlaps the fine
tiles beside it. That is the crack-or-overlap every LOD scheme pays for. JS
answers it on the CPU with per-tile bookkeeping (`lodTileVisibility`,
`tilesWithDetailedGeometry`) and a `boundaryWallsGroup` that fills the gap at
the render-distance edge. A one-way GPU selection cannot keep a `Set` per frame,
so it needs a rule a tile can evaluate alone, from its own direction and the
camera, and still agree with its neighbours. `design.md` carries the candidates
and the one that looks right; this is the part to prototype before building.

## Non-goals

- Editable voxels, caves and streaming. That is `voxel-engine-foundation`. This
  change is about how much of a surface is drawn and at what size, not about
  what a cell is made of.
- Porting the JS CPU worker, its chunk bookkeeping, or its boundary walls.
- Choosing the body radius. This change is what makes that choice stop mattering
  as much; it does not make it.
