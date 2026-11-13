# Design: stars that do not flicker

## Measured from the code

`scene.rs`: half-width `20 + hash^8 * 150` m at 180,000 m. With a 60 degree
vertical field of view, one pixel of 1,800 is 6.1e-4 rad, 110 m at the star
sphere; at 85 % render scale 129 m. So:

| Star | Half-width | Across, pixels (1,800 high) | at 85 % |
| --- | ---: | ---: | ---: |
| most (hash^8 near 0) | 20 m | 0.36 | 0.31 |
| median of the top 5 % | about 60 m | 1.1 | 0.93 |
| largest | 170 m | 3.1 | 2.6 |

## The material

`StarMaterial` (no uniforms; `AlphaMode::Opaque`, no prepass, no shadows).
Mesh: six vertices a star, all at its centre; `UV_0` the corner in -1..1
times the star's pixel half-width; `COLOR` its tint times its intensity.
Vertex: the centre to clip space through the mesh's transform (so
`turn_stars` still turns them), then `clip.xy += corner * 2 / viewport * clip.w`.
Fragment: `colour * (1 - smoothstep(0, 1, r))^2`, r the corner's length over
the half-width.

Half-width in pixels: `max(1.25, metres / 110)`. Intensity: the true
footprint over the drawn one, `(2 * metres / 110)^2 / (area of the falloff)`,
at most 1. Pixels are fixed, so a star keeps its size and light at any
resolution, as a point source does.

## As built

The cone falloff above stopped stars vanishing (12.5 % to 0.1 % a frame) but,
2.5 px across, summed to different light at each offset between pixel
centres: each star's light changed by a median 11 % a frame. The star is a
Gaussian instead, width `max(0.8, 0.66 * metres / 110)` px, the quad three
widths out, intensity the true footprint over `2 pi sigma^2`, at most 1.
Measured in `docs/screenshots/steady-stars/README.md`: nothing vanishes and
no star's light changes by more than 25 % from one frame to the next.
