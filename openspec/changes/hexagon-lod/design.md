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
