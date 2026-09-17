# Design

## The tier stack, from the JS tree

`Planet.ts` runs four tiers keyed on altitude, and every one of them is
hexagons:

| Tier | Altitude band (spawn planet) | What it draws |
| --- | --- | --- |
| DETAILED | 0 to 50 | real hex blocks, chunked, 16-24 tiles of render distance |
| INTERMEDIATE | 50 to 100 | simplified hex terrain |
| HIGH (chunked LOD) | 100 to 150 | 32 hex chunks: 12 icosahedron vertices + 20 face centres, frustum-culled per chunk by bounding sphere |
| DISTANT | 150 and up | three `GoldbergPolyhedron` at `baseSub - 2`, `- 3`, `- 4`, swapped at `distantLODAltitude`, `* 2.5` and `* 5` |

The chunking into 32 groups is worth keeping for a different reason than LOD:
it is what gives frustum culling something coarse to test. Twelve vertices plus
twenty face centres is a spread over the sphere that costs 32 bounding-sphere
tests instead of one per tile.

What is NOT worth keeping is where the work happens. That tree builds LOD
geometry in `lodGeometryWorker.ts` (733 lines, CPU, off-thread) and uploads
meshes, and it keeps `lodTileVisibility` and `tilesWithDetailedGeometry` on the
CPU per frame. This repository already has the better half of that: one
persistent storage buffer, a compute pass that culls and compacts, and an
indirect draw.

## What is uploaded, once

- **Per level, shared by every body**: the corner rings. A cell's own direction
  is its primal vertex and is already the same array across levels. Summed over
  levels this is `4/3` of the finest level.
- **Per body, once**: the height and material per cell at the finest level.
  Coarse levels index the same array, because centres nest.

Nothing per frame, nothing per LOD change, nothing back.

## How far the finest tier extends: 300 m

**~300 m of great-circle distance from the player**, and this figure is this
project's own rather than a ported one: Tenebris has no radial render distance
at all. It meshes the entire planet as one chunked mesh whenever the body is
active and culls only on a horizon/backface angular test, with its single
distance number (`MAX_DETAIL_DIST_M = 5000.0`) applying to the WHOLE BODY rather
than to tiles within it.

The fine tier turns out to be cheap, which is what makes the choice easy.
Measured on a 4,800 m body at 2.833 m tiles, counting cells inside a cap of
angular radius `d / R`:

| fine tier extent | cells | topology at 128 B |
| ---: | ---: | ---: |
| 124 m | 6,950 | 0.8 MiB |
| **300 m** | **40,670** | **5.0 MiB** |
| 600 m | 162,520 | 19.8 MiB |
| 1,200 m | 647,543 | 79.0 MiB |

For scale, 1,200 m of full detail costs the same 79 MiB the preview currently
spends on the *entire globe* at 18.9 m tiles.

300 m is chosen against the horizon, which on a 4,800 m body is

```text
d = sqrt(2*R*h + h^2)
```

| eye or altitude | horizon | in tiles |
| ---: | ---: | ---: |
| 1.6 m (standing) | 124 m | 44 |
| 10 m | 310 m | 109 |
| 180 m (ship spawn) | 1,327 m | 468 |

So 300 m covers a standing player's horizon with well over twice the margin,
which matters because a band boundary sitting exactly AT the visible horizon is
the worst possible place for a seam. It also keeps full detail under a low
flyover. Beyond that the coarser bands carry the distance, which is what they
are for: a ship at its 180 m spawn altitude sees 1,327 m, and none of that
needs 2.833 m tiles.

## Choosing the level, on the GPU: DECIDED

A tile's level is quantised from its **great-circle distance to the player**,
with the band thresholds stored as cosines so no trig runs per tile:

```text
cos_angle = dot(tile_direction, normalize(player_pos - body_centre))
T         = the band cos_angle falls into
```

One dot product and a few compares per tile. Neighbours agree by construction,
because `T` is a continuous function of the tile's own direction and one global
vector, so the only disagreement is at a threshold - and a threshold is a circle
of known radius rather than an arbitrary boundary.

**From the player, not the camera.** Camera-anchored bands re-shuffle whenever
the player looks around or the view pulls back; player-anchored bands move only
when the player moves.

What remains: what closes the ring (skirts, most likely), the thresholds
themselves, hysteresis across a boundary as the player walks, and the twelve
pentagons. `SEAM-BRIEF.md` carries those.

## The reasoning that led there

The target level for a tile is a function of the angle it subtends from the
camera: the same tile wants finer subdivision when it is near and coarser when
it is far. A tile can compute that alone from its direction, the body radius,
the camera position and a screen-space error budget.

The rule has to satisfy two things a per-tile computation makes awkward:

1. **Exactly one tile covers each piece of surface.** Draw cell `i` at level `T`
   only when `i` exists at `T` (`i < 10*4^T + 2`) and no finer tile covering the
   same ground is also drawn.
2. **Neighbouring tiles must agree**, or the boundary cracks.

Candidates, cheapest first:

- **Quantise the level from a radial distance band, not from a per-tile
  estimate.** Every tile within an angular band of the sub-camera point gets the
  same `T`, so agreement is by construction and the only seams are the band
  boundaries, which are circles whose position is known. This is closest to what
  the JS altitude tiers already do and is the one to prototype first.
- **Let the level come from the tile's own index.** `intro(i)`, the level at
  which `i` first appears, is `ceil(log4((i - 2) / 10))`. A tile draws when
  `intro(i) <= T` and is skipped otherwise, so a band's tile set is a prefix and
  cannot double-draw within the band.
- **Skirts at the band boundary.** A tile at a band edge extends its wall down to
  the coarser neighbour's height, which is the same trick the surface pass
  already uses for terrain steps and needs no new machinery. This is the GPU
  equivalent of the JS `boundaryWallsGroup`, computed rather than built.

The prototype settles which. Per this repository's rules a new mechanic's feel
gets a mockup before it gets an implementation, and a seam is exactly the thing
a still frame decides.

## Why this dissolves the radius lock

With a uniform level, tile width is `1.2087 * R / 2^L` everywhere and holding it
at 2.833 m forces `R = 300 m * 2^(L - 7)`. With LOD, that equation binds only the
tier the player occupies. The resident finest tier is a cap around the player
rather than a globe, so its cell count is set by render distance, and the coarse
tiers covering the rest of the sphere are a prefix of the same array costing
`4/3` of one level in total.

The gold standard therefore reads as: **the tile a player can walk on, dig, or
stand beside is 2.833 m across and 1 m tall, on every body.** A tile on the far
limb being coarser is not a violation of that, because nobody is standing on it.

## What the far tier must still get right

The complaint that started this is a visual one, so the acceptance is visual:

- The silhouette stays polygonal. A planet seen from orbit is made of hexagons,
  and it never becomes a smooth ball.
- Materials and terminator read the same across a band boundary. The far tier
  shades through the same terms as the near tier rather than a separate impostor
  shader with its own hard-coded palette, which is the
  `preview-scale-and-shader-parity` complaint arriving from the other side.
- A band boundary does not sweep visibly across the ground as the camera moves.


## Implementation decisions, taken to build it

The owner's "implement all of that" reached this change, so the open items in
`SEAM-BRIEF.md` are decided here, on the evidence available, and the first
build carries them. Each is a decision that can be revisited from a picture.

### The prefix property is kept for the resident levels and cannot reach level 11

"A coarse tile is a prefix of the fine array" is true and is what makes the
resident coarse levels free: the base level's cells are the first `10*4^L+2`
of any finer level. It cannot be how the finest tier is addressed, because the
finest array is 42 million cells and is not resident. The fine cap is instead
generated **locally by lattice address**: a vertex at level `L` on icosahedron
face `f` is the lattice point `(i, j)` with `i + j <= 2^L`, its position is the
recursive midpoint construction (`even/even` inherits the level below, an odd
coordinate is the normalised midpoint of its two parents, the `odd/odd` case
along the diagonal `i + j = const`), and that construction is bit-for-bit the
one `dual_sphere` runs, so a fine point that is also a coarse point lands on
the same float. Corners and neighbours come from enumerating the level-`L`
triangles that intersect the cap (a quadtree descent from each face, pruned by
bounding cap) and building the dual from them exactly as `dual_sphere` does
from its own triangle list; shared vertices across faces dedupe on their bits.

### Levels, bands and the partition rule

| level | tile | drawn where |
| ---: | ---: | --- |
| 7 (base) | 45.3 m | everywhere the finer bands do not cover; resident whole, 163,842 cells |
| 8 | 22.7 m | within 2,400 m of the player |
| 9 | 11.3 m | within 1,200 m |
| 10 | 5.67 m | within 600 m |
| 11 | 2.833 m | within 300 m |

Resident: the base plus about 30,000 to 40,000 cells per fine level, some
300,000 in all, against 655,362 today.

**The partition is by the coarser level's cells, not by a circle through
tiles.** A partition by tile centres cannot be exact where two tile sizes
meet: a coarse tile whose centre is just inside the fine band has area
outside it that no fine tile covers, and one just outside has area inside it
that fine tiles also cover, so it gaps on one side and double-draws on the
other. The unit is therefore the coarse cell: a level-`L` cell `A` is **fine**
when `dot(d_A, p) > cos(D_{L+1} / R)`, and then its whole region is drawn at
level `L+1`. That is exact because the level-`L+1` cells inside `A` are known
from the lattice: the one centred on `A` itself (a vertex-centred cell, wholly
inside) and the six centred on the midpoints of `A`'s edges, each shared with
the neighbour across that edge. A midpoint cell `M_AB` is split exactly in
half by the coarse boundary between `A` and `B`, which runs along its own long
diagonal, so `M_AB` draws the half nearer whichever of `A`, `B` is fine and
discards the other per fragment: two dot products against two owner
directions it carries in its record.

A level-`L` tile therefore draws when it is not itself fine (its own centre is
outside the `L+1` band) and its coarse owner is fine (or `L` is the base).
Every test is a dot product of a direction the tile carries against one
global player direction, so neighbours agree by construction and nothing is
read from a neighbour.

### Closing the seam

Along the boundary between a fine coarse-cell `A` and a coarse one `B`, the
fine side is half of `M_AB` at its own height and the coarse side is `B`'s cap
at `B`'s height, and one of them must wall down to the other:

- **`B` higher.** `B`'s ordinary side wall toward `A` goes down to the height
  of `M_AB` instead of `A`'s; each record carries that **fine floor** per
  side, the height at the edge midpoint, computed at generation from the same
  `surface_height`.
- **`M_AB` higher.** `M_AB` emits one more quad, a **cut wall** along its
  diagonal from its own height down to `B`'s, which it already has as its
  same-level neighbour height on that side, because the fine cell centred on
  `B` shares `B`'s direction and so `B`'s height. The two diagonal corners are
  the two of its six equidistant from `A` and `B`.

The vertex budget per terrain instance goes from 54 to 60 for the cut wall;
foliage stays a separate draw.

**Trees on three levels, at one density per area.** The first capture of the
built partition (`--view seam`) showed the forest ending in a straight line at
the 300 m band edge, because trees were eligible on the finest level only: a
band boundary made visible by what stands on it, which is the seam rule broken
by foliage rather than by terrain. Trees are eligible on levels 9, 10 and 11
now, out to the level-9 band (1,200 m), and a cell's chance of a tree is the
finest level's per-material density times `4^(11 - level)`, so a level-10
cell, which covers four finest cells, carries four times the chance and the
forest has the same trees per hectare at every distance it is drawn at. Which
cells carry them still comes off each cell's own stable ID, so the individual
trees do change where a band moves; what does not change is the cover. The
same geometry is drawn on every level, so distant groves are Tenebris-sized
trees standing on coarser ground.

### What is not done, and stated

- **Hysteresis.** A debounce needs a memory of which side a tile was on, and
  the rule is that nothing about a tile persists on the GPU. Anchoring to the
  player already removes the camera-driven case; the residual is a tile on a
  band circle flickering as the player oscillates by centimetres across it.
  Accepted for the first build; a pinned still frame of a boundary is the
  evidence, and a frame-to-frame diff of a slow walk is the test a debounce
  would earn its way in by.
- **Streaming is whole-set, asynchronous.** When the player moves more than a
  quarter of the finest band from the last anchor, every fine level is
  regenerated for the new anchor on the compute pool and the fine region of
  the storage buffer is rewritten when it lands. No incremental edits, no
  per-frame CPU visibility state. The base level is uploaded once.
- **The walker's contact** is the finest resident level: `PlanetContact` is
  built over the level-11 cap and swapped with it, so a player stands on
  2.833 m tiles and 1 m steps. The flight clearance keeps the analytic
  `terrain_radius`. The walker's step height rises from 0.6 m to a cell, since
  a step it cannot climb every 2.8 m is a wall, not terrain.
