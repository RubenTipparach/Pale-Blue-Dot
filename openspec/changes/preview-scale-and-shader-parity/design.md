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

## ONE `daylight` drives the sun AND the sky, and the sun sets 7 degrees late

The owner: a directional light dims as it crosses the horizon, and it cannot
light anything below zero degrees. Measured against the shipped shader, both
halves of that are right and the same line causes both.

`planet_surface.wgsl` computes one number and multiplies everything by it:

```wgsl
let sun_elevation = dot(radial,sun);
let daylight = smoothstep(-0.13,0.20,sun_elevation);
let direct = max(dot(n,sun),0.0)*daylight;
```

`sun_elevation` is the SINE of the sun's altitude, so those edges are angles:

| sun's altitude | our `daylight` |
| ---: | ---: |
| +10 deg | 0.982 |
| +5 deg | 0.729 |
| +2 deg | 0.500 |
| **0 deg** | **0.343** |
| -2 deg | 0.201 |
| -4 deg | 0.088 |
| **-7.5 deg** | **0.000** |

So direct sunlight is at a third of full strength with the sun ON the horizon,
and the ground is still being lit by a source seven and a half degrees under
it. Anything that reads `direct` - the Lambert term, the specular, the rain
glint, and the shadows this will grow - is lit by a sun that has set.

**The standard says where it ends.** Sunset is defined at a geometric altitude
of **-0.833 deg**: 34 arcminutes of refraction plus the sun's own 16 arcminute
disc radius. Past that no direct light reaches a surface at sea level. What
continues is SCATTERED light, and it has its own named bands - civil twilight
to **-6 deg**, nautical to -12, astronomical to -18.

**So the one number is two.** They are different physics and they want
different curves:

- **`sunlight`**, for the direct term, the specular and the glint: full while
  the disc is up and nought by -0.833 deg, `smoothstep(-0.0145, 0.02, s)` on
  the sine. Narrow on purpose - a surface already dims at grazing incidence
  through `dot(n,sun)`, which is the geometry doing the work, and the present
  ramp double-dims it. That is why the current model is BOTH too dark at noon
  on a low sun and too bright after dark.
- **`twilight`**, for the ambient, the distance fog and the rim: a long
  falloff to about -6 deg, `smoothstep(-0.105, 0.05, s)`, which is the glow
  that should be the only thing on the ground once the sun is down.

The one-line split is the whole change, and it is deliberately not taken here:
this document is the write-up and the edit is a separate request. What it will
need with it is a capture at a few sun altitudes either side of zero, because
the thing to check is that the last direct highlight dies at the horizon and
the ground stays visibly blue rather than going black.

Sources: [USNO, Rise, Set, and Twilight Definitions](https://aa.usno.navy.mil/faq/RST_defs)
and [Sunset](https://en.wikipedia.org/wiki/Sunset).

## A face's tile is the MATERIAL's, and ours is the altitude's

Measured against the reference and against the shipped art, not felt. Tenebris
answers "what picture goes on this face" with one function,
`blocks::face_tile(block, cap)`, and the answer is the standard Minecraft
convention its own doc comment names:

| block | top | side | bottom |
| --- | --- | --- | --- |
| Grass | `grass.png` | **`dirt_grass.png`** | `dirt.png` |
| Snow | `snow.png` | **`dirt_snow.png`** | `dirt.png` |
| Farmland | `farmland.png` | `dirt.png` | `dirt.png` |
| everything else | its own tile | its own tile | its own tile |

and its generator stacks the column so there is something for those tiles to
describe: the top cell is the biome's sod, `altitude > surface - 4.0` is dirt
(sand under a desert), and everything below that is stone.

**Our column already stacks exactly that way.** `column::generate_solid` gives
the top metre the biome material, the next three metres `Soil` (`Sand` under
sand, `Stone` under rock and snow), and stone below. The stack is right. What
is drawn on its faces is not, in three separate places:

- **`render_code` collapses soil into grass.** `Material::Soil`, `Grass` and
  `DryGrass` all become code 2, and code 2 is the fragment shader's default:
  base `vec3(0.12,0.32,0.075)` on tile `(0,0)`, which is the sward. So a dirt
  layer under the sod, and a cave wall cut through it, come out GREEN. The same
  line collapses `Material::Dirt` into `Sand` (code 1, the sand tile), so the
  one material actually named dirt is drawn as beach.
- **A terrace wall ignores the material entirely.** `kind==1u` blends
  `vec3(0.30,0.21,0.13)` to `vec3(0.34,0.36,0.37)` over `smoothstep(60.,180.)`
  of ABSOLUTE PLANET HEIGHT and picks tile `(2,0)` or `(3,0)` the same way. So
  a one-metre step in a meadow at 200 m draws stone to its foot, an identical
  step at 40 m draws dirt to its foot, and neither asks what is actually in
  those cells. There is no grass-to-dirt side anywhere on the body.
- **The art for it is shipped and never sampled.** Each `assets/tilesets/*.png`
  is a 4x4 grid of 313 px tiles, and `fields.png` holds the whole Tenebris set:
  `(0,0)` sward, `(1,0)` **the grass-to-dirt transition**, `(2,0)` dirt, `(3,0)`
  stone, with cobble, mossy stone, coarse dirt, dark soil, ore, clay, bark, a
  log end, leaves and planks in the rest of it. `(1,0)` is sampled by nothing:
  the only `vec2(1.,0.)` in the shader are UV corners of a side quad.

**So the picture the owner is describing is already paid for.** The stack knows
what it is made of, the atlas has the four pictures, and what is missing is the
one function between them that Tenebris has and we do not: tile chosen from the
material AND the face, with the top cell's side getting the transition.

The shape of it, for when this is implemented:

- `pbd_core::terrain` grows a face rule beside `Material`, the one authority:
  `(material, Face::{Top,Side,Bottom}) -> tile code`, with the grass, dry
  grass, jungle grass and snow families taking the transition on their sides
  and dirt underneath. A test pins the four rows of Tenebris's table.
- `render_code` stops collapsing `Soil` into the grass code and `Dirt` into the
  sand code. They are different pictures, and the codes are what carry that to
  the GPU.
- The terrace wall's altitude blend goes. A wall runs from this cap down to the
  neighbour's, so what it crosses is THIS column's layers: the transition for
  the first metre, then soil to the bottom of the soil, then stone. The wall
  already knows its own height along the face (`side_uv` runs 0 to 1 across
  it), so the rule is a depth comparison rather than new data.
- The column pass needs nothing new: a run already carries a cap material and a
  body material, which is exactly top-and-side, and its flank is already drawn
  run by run.

Measured shares, so the work has a before: at the default spawn a wall face is
drawn stone-by-altitude or grass-by-collapse on every one of the tier's 3,105
columns, and the transition tile is drawn 0 times.

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
