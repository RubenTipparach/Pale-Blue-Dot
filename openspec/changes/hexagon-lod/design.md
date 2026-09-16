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

## Choosing the level, on the GPU

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
