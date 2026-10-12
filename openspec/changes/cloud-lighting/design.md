# Design: cloud lighting

## 1. What the march does now, measured

`tools/cloud_light.py` transcribes the lighting arithmetic in
`shaders/clouds.wgsl` (the density profile, `cloud_shadow` and the `lit` term of
`cloud_march`). It runs that over a flat slab whose noise is a constant
`shape`, viewed straight down at the top and straight up at the base, with the
shipped values: slab 260 m, thresholds 0.61 / 0.2016, extinction
0.0115 / 0.03, base darkness 0.34 / 0.12, night floor 0.045.

The shadow term at noon, sun 60 deg, fair cover, shape 0.9:

| Height in slab | Density | Shadow now | Shadow at the view's extinction |
| ---: | ---: | ---: | ---: |
| 180 m (lit top) | 0.85 | 1.000 | 1.000 |
| 130 m | 0.96 | 0.019 | 0.301 |
| 80 m | 0.96 | 0.001 | 0.125 |

- The first light sample is 41 m in; the next three are 83 m apart. A sample
  either sees clear sky toward the sun or sees about a full optical depth of
  4. Nothing sits between those two outcomes.
- The body of any cloud is therefore lit at `cloud_base_dark` times the sun.
  The view march spends most of its opacity in that body, because the tapered
  top is thin. So the top the eye sees averages to grey: **0.44 against 0.82**
  for a fully lit sample.

**Captured**, noon, slider at half:
- **From 900 m above the shore** (`docs/screenshots/cloud-lighting-before-above-half.png`), the deck's
  sunlit top reads as a grey band with darker smudges. Its unsaturated pixels
  have luminance median **0.57**, p90 0.65 and brightest **0.69** of display
  white *(measured)*. No part of a sunlit top reaches white.
- **From below** (`cloud-lighting-before-up-half.png`), the sky is an even grey
  sheet with no bases to tell apart.

## 2. The model

Every term below is inside `pbd::clouds`. Both the sky shell and the sea cap
import it, so there is one implementation.

**Light march.** Six samples from the sample point toward the sun, with step
lengths in the ratio 1, 1.5, 2.25 ... so that together they cover the slab's
thickness along the sun ray. The steps stop at the slab's outer shell. The
result is `tau_sun`, optical depth measured with **the same `extinction`** the
view march uses: `mix(cloud_extinction, cloud_storm_extinction, cover)`.

**Multiple scattering** (Hillaire 2016, after Wrenninge 2013). For octave
`i = 0..N`:

```text
L_i = b^i * sigma_s * phase(g * c^i, cos_theta) * exp(-a^i * tau_sun)
```

and the light scattered toward the eye is the sum of the octaves. `a < 1`
lets later octaves' light reach deeper, `b < 1` weights them less, and `c < 1`
makes them less forward-peaked. N = 3; the starting values are Hillaire's
(a 0.5, b 0.5, c 0.5), tuned on the measurements below.

**Phase.** Two Henyey-Greenstein lobes, forward `g_f` 0.8 and back `g_b` -0.3,
blended by `phase_blend` 0.5. Looking toward the sun, cloud edges brighten (a
silver lining); looking away, the lit faces are what reads.

**Ambient.** Two terms, both dimmed by the cloud standing between the sample
and where the light comes from:

- **Sky light from above**: `ambient_sky * sky_colour * exp(-tau_up)`, where
  `tau_up` is optical depth straight up, from three samples to the slab top.
- **Ground bounce from below**: `ambient_ground * ground_albedo * sun_on_ground
  * exp(-tau_down)`, with `tau_down` taken the same way toward the base.

A sample at the base of a tall storm has hundreds of metres of dense cloud over
it, so `tau_up` is large and the sky cannot light it. A sample at the base of
a thin cumulus has little cloud over it, so the sky still does. **That is the
"dark on the bottom depending on density" the owner asked for, derived from the
field rather than set by a knob.** `sky_colour` is the zenith colour the dome
already computes for the sun elevation. The night floor stays, as the ambient
floor when the sun is down.

**What goes.**
- The `0.05` literal.
- `cloud_base_dark` and `cloud_storm_dark` (with their `CloudLayer` lanes
  `slab.w` and `storm.y`). Darkness is now an output.
- The `day*0.90` factor, which lit the whole cloud by the zenith cosine of its
  column. Sunlight now enters through `tau_sun` and the phase function.

**What is added to `weather.ron`**, validated with units:

| Knob | Unit | Default |
| --- | --- | ---: |
| `cloud_scatter_octaves` | count, 1..4 | 3 |
| `cloud_scatter_extinction_falloff` (a) | fraction | 0.5 |
| `cloud_scatter_energy_falloff` (b) | fraction | 0.5 |
| `cloud_scatter_phase_falloff` (c) | fraction | 0.5 |
| `cloud_phase_forward` | HG g, 0..1 | 0.8 |
| `cloud_phase_back` | HG g, -1..0 | -0.3 |
| `cloud_phase_blend` | fraction | 0.5 |
| `cloud_ambient_sky` | multiplier | tuned |
| `cloud_ambient_ground` | multiplier | tuned |
| `cloud_light_steps` | count, 2..8 | 6 |

## 3. Cost

Density evaluations per view sample:
- now: 1 + 4 = 5;
- after: 1 + 6 (sun) + 3 (up) + 2 (down) = 12. All octaves reuse the same
  `tau_sun`.

A view sample that `cloud_density` finds empty still returns before any light
march, as it does today. The frame cost is measured on llvmpipe as an A/B
(sky-up, horizon, orbit), which is a relative number only. If the up and down
marches are too dear, `tau_up` becomes one read of the column's cloud water from
`atmospheric-circulation`'s weather map, which is a single texture fetch. That is
the intended end state; the three-sample march is what works before that map
exists.

## 4. How it is checked

- **The transcription.** `tools/cloud_light.py` is updated with the new
  arithmetic in the same commit. The claims in the proposal (top 85%+,
  monotonic base, storm 15% or less, cumulus 35-60%) are checked on its output,
  and the before and after tables go in this document.
- **One extinction, pinned.** A test reads `clouds.wgsl` and fails if the light
  march uses a numeric extinction rather than the layer's. This is the same way
  `rain_light` is held to the terrain's day curve.
- **The shared layout.** `CloudLayer` changes shape, so the existing size test,
  which checks the Rust struct against naga's layout, moves with it.
- **Captures.** Taken at noon and at sun 20 deg, under fair, half and storm
  cover:
  - from below, looking up;
  - from the side, at 250 m;
  - from above, at 900 m.

  They are measured the way `overcast-and-rain` measured its bands: the mean
  luminance of cloud pixels over their tops against their bases.
- The visual check in the running game is the owner's. Until then, the new
  requirement stays in this change.

## 5. What was built, against this plan

Built in `assets/shaders/clouds.wgsl` (one after-scene clouds pass in the water
chain, so the sky, the sea and the land all get the same clouds). Where it
differs from the sections above:

- **Octaves and light steps are fixed in the shader**, at 3 and 6, not knobs.
  Nothing in tuning called for another count, and a count is a loop bound the
  shader would otherwise have to clamp. `cloud_scatter_octaves` and
  `cloud_light_steps` were not added.
- **No downward march.** The ground bounce is `ground * day * (1 - height)`: it
  lights the base and fades toward the top. A `tau_down` march cost three more
  density reads for a term that only ever reaches the bottom few samples.
- **The knobs that exist** (`weather.ron`, validated): `cloud_sun`,
  `cloud_ambient_sky`, `cloud_ambient_ground`, `cloud_phase_forward`,
  `cloud_phase_back`, `cloud_phase_blend`, and the three scatter falloffs
  (extinction, energy, phase). `cloud_base_dark`, `cloud_storm_dark` and the
  `0.05` literal are gone.
- **Where the cloud is comes from the atmosphere's weather map**, not the old
  deterministic field. That map is 64 texels a cube face (118 m), and a cloud's
  edge is a steep threshold on its cover. **Read bilinearly, every cloud from
  orbit was outlined in texel-sized stairs.** A checkerboard written into the map
  showed the sampler really was filtering: the stairs are what a threshold makes
  of bilinear's slope jumping at every texel line. The map is read with a cubic
  B-spline instead (`cloud_map_smooth`, four bilinear taps, placed on the face the
  direction points through; a tap past a face's edge resolves on the next face).
  The ground's cloud shadows import the same function, so a shadow and its
  cloud cannot disagree.

- **The stairs had a second, larger cause, and it was not the map.** With the
  B-spline in, the orbit capture still showed screen-aligned blocks about seven
  pixels across, inside the clouds as well as at their edges. Every after-scene
  pass takes its ray's direction from a point reconstructed at clip depth 0.5
  (`view_ray` in `water.wgsl`), which under Bevy's reverse-Z is twenty
  centimetres ahead, minus the camera. Both are f32 body-local metres; from
  orbit the camera is some 15 km out, where f32 rounds to about a millimetre, so
  each ray was turned by up to 0.005 rad: seven pixels at this field of view.
  The point is now taken a kilometre out (`VIEW_RAY_DEPTH`), which puts the same
  millimetre at a microradian. It was the whole of the blockiness; the sea's
  sheet and the rain use the same ray and were quietly carrying the same error.

Still open: the transcription (`tools/cloud_light.py`) has not been updated to
this model, and the captures below have not been measured against the
proposal's four numbers. Until they are, this is implemented, not validated.
