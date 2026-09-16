# The LOD seam question: a self-contained brief

This file is written to be handed to someone, or something, with no access to
the rest of the repository. Everything needed to answer the question is in it.

## The question, in one sentence

> When two levels of detail meet on a spherical hexagon grid, how does a tile
> decide **alone, on the GPU, from its own direction and the camera** whether to
> draw itself and at which level - so that the surface stays closed, nothing is
> drawn twice, and the boundary does not visibly sweep across the ground as the
> camera moves?

Everything below is context for that sentence.

## The grid

A planet's surface is the **dual of a midpoint-subdivided icosahedron**. Start
from a 12-vertex icosahedron; each subdivision splits every triangle into four
and normalises the new midpoints onto the sphere. At level `L` there are
`10 * 4^L + 2` dual cells: hexagons, plus exactly twelve pentagons at the
original icosahedron vertices.

A dual cell sits on a **primal vertex**. Two cells are adjacent exactly when
their primal vertices share a primal edge, so the centre-to-centre distance
between neighbours IS the flat-to-flat width of the shared tile.

Measured mean tile width on a body of radius `R`:

```text
width = 1.2087 * R / 2^L
```

### The property the whole scheme rests on

Subdivision **appends** midpoint vertices after the existing ones, so a vertex
keeps its index forever. Measured:

```text
L2->L3: first     162 of     642 vertices identical, at the same index
L3->L4: first     642 of   2,562 vertices identical, at the same index
L4->L5: first   2,562 of  10,242 vertices identical, at the same index
L5->L6: first  10,242 of  40,962 vertices identical, at the same index
L6->L7: first  40,962 of 163,842 vertices identical, at the same index
```

Consequences:

- **A coarse level's tiles are a PREFIX of the fine cell array.** Cell `i` is the
  same direction at every level fine enough to contain it. There is nothing to
  merge and no second mesh to keep in step.
- **A coarse tile's height is free**: its centre *is* a fine cell's centre, so
  its height is `heights[i]` from the array that already exists.
- Define `intro(i)` = the level at which cell `i` first appears = the smallest
  `L` with `i < 10 * 4^L + 2`. A tile "exists at level T" iff `intro(i) <= T`.

What a level *does* need of its own is its **corner ring**, because a coarse
cell's corners are its own triangulation's face centres, not the fine one's.
Summed over all levels that is `4/3` of the finest level alone.

## The fixed parameters

| | |
| --- | --- |
| Body radius | 4,800 m |
| Finest level (underfoot) | 11, giving 2.833 m mean tile width |
| Vertical quantum | 1.000 m |
| Player eye height | 1.6 m |
| Cells at level 11 | 41,943,042 (**not** all resident - see the constraints) |

2.833 m and 1.000 m are a fixed spec: a cell is the same size on every body in
the game, so a player who digs a hex of dirt on one planet finds the same size
hexes on the next. That standard binds **the tier the player occupies**. A
coarser tile on the far limb is not a violation, because nobody is standing on
it. This is precisely why LOD is mandatory rather than an optimisation: a
uniform level 11 over the whole globe is 42 million cells and about 5 GiB of
topology.

## The constraints the answer must satisfy

1. **One-way to the GPU.** Topology and heights upload once. The level per tile,
   the visibility cull and the draw arguments are all computed in a compute pass
   and consumed by an indirect draw. **Nothing about a tile is read back, and no
   per-tile visibility or level state is maintained on the CPU per frame.** A
   solution that keeps a per-frame set of "tiles currently detailed" on the CPU
   is rejected by construction, however well it works.
2. **The CPU stays authoritative for terrain**, edits, collision and
   persistence. LOD is a drawing decision only. A GPU-chosen level must never
   decide where a player stands.
3. **Closed surface.** No crack to space between two levels, and no ground drawn
   twice (the shading is not idempotent, so a double-draw is visible).
4. **No crawling boundary.** The level change must not read as a line travelling
   over the terrain as the camera moves. Hysteresis is allowed; shimmer is not.
5. **A tile must be able to decide alone.** It may read its own direction, its
   own index, the camera pose, the body pose and global constants. It may not
   read its neighbours' decisions, because they are being computed in parallel
   in the same dispatch.

## Why a naive answer fails

Pick a target level `T` per tile from its distance to the camera, then draw tile
`i` iff `intro(i) <= T`.

Within a region of constant `T` this is exactly right: the drawn set is the
level-`T` prefix, which tiles the sphere once. It fails at the boundary. Where a
`T=11` region meets a `T=10` one, the coarse side draws tiles whose footprints
extend into the fine side, and the fine side draws tiles underneath them. You get
overlap on one side of the line and, if the levels disagree the other way, a gap.

So the difficulty is not choosing a level. It is making neighbours agree about
the level **without letting them talk to each other**.

## Three candidate directions

Not exhaustive, and not ranked by confidence - the point of the exercise is to
decide between them on evidence.

1. **Quantise the level from an angular band around the sub-camera point**, not
   from a free per-tile estimate. Every tile whose direction lies within an
   angular band of the point directly beneath the camera gets the same `T`.
   Agreement is then by construction, because the band is a function of the
   tile's direction alone and is continuous in it. The seams become circles
   whose positions are known in closed form, which is what makes case 3 below
   tractable. Closest to what the reference implementation does with altitude
   tiers.
2. **Derive the level from the tile's own index.** `intro(i)` is computable as
   `ceil(log4((i - 2) / 10))`. Within a band the drawn set is a prefix and
   cannot double-draw. This does not by itself solve the boundary; it is the
   rule for *which* tiles draw once `T` is fixed.
3. **Close the boundary with skirts.** A tile at a band edge extends a wall down
   to the coarser neighbour's height - the same trick the surface pass already
   uses for ordinary terrain steps between adjacent columns, so it needs no new
   machinery. This is the GPU equivalent of building boundary geometry, computed
   rather than uploaded.

A likely-looking combination is 1 + 2 + 3: band-quantised `T`, prefix rule for
membership, skirts to close the circles. That is a hypothesis, not a conclusion.

## What counts as an answer

A **still frame** of two adjacent bands at the worst angle - grazing, where the
overlap is widest - plus:

- the rule stated precisely enough to implement,
- what happens at a pentagon on a band boundary (there are twelve, and they have
  five neighbours, not six),
- the hysteresis rule that stops the band shimmering as the camera moves,
- and an honest account of what the rule costs per tile per frame, since it runs
  over every tile in a compute dispatch.

A prototype is worth more than an argument here. A seam is the kind of thing a
picture settles and prose does not.

## Explicitly out of scope

- Editable voxels, caves, streaming and durable edits. Different problem.
- Choosing the body radius or the hex size. Both are fixed above.
- Porting a triangle-mesh impostor for the far tier. The far tier is hexagons;
  that is the entire point of the change this brief belongs to.
