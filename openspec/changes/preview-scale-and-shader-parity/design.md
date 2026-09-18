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

## One water shader, viewed from above and below

`water.wgsl` is the visual reference because it is the existing faithful port
of the previous Tenebris water shader. The full inventory, with captures of the live
branch at 1.6, 10, 50, 200 and 1000 m, is in `docs/tenebris-comparison.md`
under "Water: five systems in Tenebris, one branch here".

**What binding the port does and does not restore.** Tenebris's water is five
systems: the cap pass, the composite pass (underwater fog, distortion, lens
droplets, emerge drips), terrain wetness in `hex.fs`, precipitation, and the
CPU flow simulation. `water.wgsl` ports the cap pass and, measured against
`water.fs.glsl` line by line, drops five terms from it: rain ripples, flow
advection, the waterfall scroll, the two foam weights and the specular sun
tint. The last two are one-line omissions and SHALL be corrected in the port
before it is bound, since the port is the reference. The first three need a
rain intensity and a flow field, which this project does not have, and the
composite, wetness, precipitation and flow systems have no counterpart here at
all. Binding the port is therefore the first step of the water work and not
the whole of it; the remaining systems are each their own change and are not
yet written up.

**Two terms the port now carries that Tenebris does not, both on the owner's
call after seeing the first captures ("water super glitchy where it overlaps",
"way too shiny").** The sheet self-sorts against a private single-sample depth
buffer, because a wavy sheet with no depth between its own fragments lets a far
trough draw over a near crest in whatever order the cells come; Tenebris has
the same self-sort through its swapchain depth. And the wave height and
gradient are scaled by `1 / (1 + footprint * detail_fade)`, where footprint is
the per-pixel change of the noise coordinate, so once the fbm's features fall
under a pixel they fade instead of aliasing into white sparkle; `detail_fade`
0 is Tenebris exactly. The shine itself was data: `sky_horizon_color`,
`sky_zenith_color`, `sky_horizon_strength` and `specular_intensity` are
re-authored for this tone-mapped pipeline in `water.ron`, and the Tenebris
values are in the file's history. The live water branch in
`planet_surface.wgsl` is not a fallback reference: it is a second,
feature-incomplete implementation and is removed once the dedicated water pass
is live.

The dedicated pass receives the same water surface, camera and body-local frame
on both sides of the interface. Scene colour and scene depth are explicit render
inputs so the shader can recover the prior refraction and optical path-length
absorption instead of approximating them. Whether the camera is above or below
the surface selects a branch inside this one shader and one parameter set; it
does not select a separate material, pipeline look, wave model or palette.

"Exactly the same as before" means visual parity with the committed faithful
port and its Tenebris source, not byte-identical output from a different graphics
API. Acceptance therefore uses fixed-camera, fixed-time captures above and below
the surface and checks the visible terms separately: wave displacement and
normals, Fresnel reflection, refraction, foam, depth/path-length absorption and
the underwater view. Crossing the surface must not introduce a discontinuity in
wave phase, water colour or geometry. Parser validation alone cannot establish
this; the real render graph and both camera positions must be exercised.
