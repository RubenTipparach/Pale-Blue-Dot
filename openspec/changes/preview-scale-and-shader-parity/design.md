# Design

## The scale arithmetic

The dual of a midpoint-subdivided icosahedron puts a cell on every primal
vertex, and two cells adjoin exactly when their vertices share a primal edge.
So the centre-to-centre tile spacing IS the primal edge length scaled to the
body, which is what `planet::tile_width` computes and what
`planet::tile_widths` reports at startup.

Measured on the unit sphere, the mean edge converges to `1.2087 / 2^L`, so

```text
tile width = 1.2087 * R / 2^L
```

Tenebris's own `subdivisions_for_radius` comment writes this constant as 1.05,
which is about 15% low; its clamp to level 7 is what actually decides its tile
size, so the error never showed. Both engines are measured here rather than
taken from that comment.

| | R | L | tile width | vertical step |
| --- | ---: | ---: | ---: | ---: |
| Tenebris | 300 m | 7 | 2.832 m | 1.0 m |
| This preview | 4,000 m | 8 | 18.883 m | 6.0 m |
| This project's target | 4,000 m | 12 | 1.176 m | 1.0 m |

Level 12 is 167,772,162 cells. At the current 128-byte topology record that is
21.5 GB for one globe, before heights, meshes or collision, which is why the
target is a streaming problem and not a constant to raise.

## The shader terms that differ

Against Tenebris's `hex.fs`, the live `planet_surface.wgsl` keeps the
terminator, the Lambert term and the fresnel rim, and drops:

| Term | Tenebris | Live prototype |
| --- | --- | --- |
| Night-side rim floor | `0.25 + 0.75 * day`, from `lod.yaml` | multiplied by `daylight`, so zero |
| Ambient floor | `max(0.05, ...)` | none |
| Torch / block light | `torch.rgb * torch.w * v_torch_light` | absent |
| Light sampling | per vertex, interpolated | per cell, `@interpolate(flat)` |
| Underwater absorption | `exp(-absorption * depth)` | absent |
| Cutout foliage | 4x4 Bayer screen door | absent |
| Per-planet palette | a `lod.yaml` section per tileset | inline literals |

`hex_terrain.wgsl` already has all of them, as uniforms. It is the reference for
anything restored here: change it there first if the two ever need to differ.

## Why the flat tile matters more here than there

`@interpolate(flat)` on skylight costs Tenebris almost nothing across a 2.8 m
tile. Across 18.9 m it makes each hexagon a single brightness with a hard step
at every edge, which is the largest single reason the ground reads as faceted
plates. The fix is a per-corner skylight value rather than a per-cell one, so it
is upload-side work and is deliberately not bundled with the two cheap shader
changes.
