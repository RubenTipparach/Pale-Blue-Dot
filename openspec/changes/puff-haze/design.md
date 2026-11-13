# Design: puff-haze

## 1. The outline, shown

Default spawn, 24.117 -2.964 (map 24.117S 2.964W), 65.6 m elevation, 1.6 m
above ground, `--time 12` (sun 61.4 degrees up), `--weather-at 1800` (cover
0.55, rain 0.01), 35 degrees up. Full launch lines in
`docs/screenshots/puff-haze/README.md`.

| | before | `cloud_haze: 0` (diagnostic) |
| --- | --- | --- |
| north, yaw 90 | `north-crop-before.png` | `north-crop-haze-off.png` |
| south, yaw 270 | `south-crop-before.png` | `south-crop-haze-off.png` |

`north-crop-before.png` is a 700 x 500 crop at full resolution (2880 x 1800,
crop at 1950,850). A front puff's silhouette runs as a curve from the top
right to the bottom middle. Brightness (mean of RGB) along row 350 of the
crop, every third pixel, crossing the line:

```text
before     174 174 176 176 176 177 177 177 178 178 178 178 179 179 179 | 172 165 164 165 165 164 165
haze off   216 216 217 217 217 218 218 218 219 219 218 218 219 219 219 | 220 219 219 220 220 219 220
```

Inside the front puff the far puff brightens toward the line (the fringe),
then drops 14 levels in about 6 pixels, which is one march texel at the 0.4
render scale, bilinearly upsampled. With the haze off the same row is flat.
`south-crop-before.png` shows the same thing several times over, with the
step as a staircase where a silhouette runs nearly level across the texels.

## 2. Why

The puff pass (`planet_water.rs`, `Pass::CloudPuffs`) has two targets:

- **Cloud**, `PREMULTIPLIED_ALPHA_BLENDING`: each puff writes
  `(lit*a, a)` and is laid "over" what the farther puffs left. This is right.
- **Distances**, blend `Min`, cleared to the far cap: each puff writes
  `(its distance, the scene's stop)`, and the texel keeps the nearest. A puff
  writes it wherever `a >= 0.004`, so the front puff's whole footprint, down
  to its almost invisible fringe, holds the FRONT puff's distance.

The composite (`clouds` in `water.wgsl`) upsamples the cloud and takes one
haze distance per pixel from those texels (`cloud_upsampled`'s `distance`),
then fades the whole cloud by the air in front of it:

```wgsl
let cloud = upsampled*cloud_air(eye, ray.direction, min(select(span.x, at, at > 0.0), far));
```

So in the front puff's footprint the far puff behind it is hazed to the front
puff's distance, and outside it to its own. `cloud_air` mirrored on the CPU
with the shipped constants (density 0.00036 per metre at the sea, scale
height 1050 m, `cloud_haze` 3, radius 4800 m) from the spawn's 65.6 m:

```text
elevation   400 m   600 m   800 m   1200 m   1600 m   2400 m   3200 m
15 deg      0.68    0.57    0.48    0.36     0.28     0.20     0.16
25 deg      0.69    0.58    0.50    0.39     0.32     0.24     0.21
35 deg      0.70    0.60    0.52    0.42     0.35     0.28     0.25
```

A near puff 800 m out over a far one 1600 m out, 25 degrees up: the far puff
is drawn at 0.50 of itself just inside the near puff's footprint and 0.32 just
outside, about one and a half times as strong inside. That is the step. The
fringe is the near puff's thin edge: it adds little of its own, and the far
puff shows through it at the near puff's lighter haze.

Ruled out: the "over" blend is premultiplied on both sides (no dark or bright
halo from a straight-alpha mix); the erosion's smoothstep is soft; and the
upsample's silhouette test keeps every texel over the sky (`far` is the
1e9 sky stop there). With the haze off the frame has no line anywhere.

## 3. The fix: each puff hazed to its own distance

### 3a. What the blend must be

The composite's haze is "fade the cloud into what is behind it": a pixel is
`L = S (1 - alpha) + C`, with `S` the scene behind (the sky, the ground) and
`(C, alpha)` the cloud's premultiplied light and coverage, both scaled by the
air `T`. That is the air's own light taken as `S`. With two puffs, near `A`
in front of far `B`, each faded by the air in front of it:

```text
C     = T_A a_A lit_A + (1 - a_A) T_B a_B lit_B
alpha = T_A a_A       + (1 - a_A) T_B a_B
```

The far puff is hidden by the near puff's own coverage `a_A`, not by its
hazed coverage `T_A a_A`: the air in front of `A` adds its light over `A`, it
does not make `A` transparent to `B`. Drawn far to near, every puff adds
`(T a lit, T a)` and scales what is already there by `(1 - a)`.

- As `a_A` goes to 0 at the near puff's edge, `(C, alpha)` goes to `B`'s own:
  no step, whatever threshold the discard uses.
- As `a_A` goes to 1 inside the near puff, it goes to `A`'s own, exactly what
  the old one-distance haze drew there. A puff alone is drawn as before.

A CPU mirror of the three blends (sky 0.25, both puffs 0.9 lit, `T_A` 0.50,
`T_B` 0.32, `B` opaque), the pixel's value as the near puff's coverage rises:

```text
a_A          0.000  0.003  0.005  0.300  0.700  1.000
physical     0.458  0.458  0.459  0.493  0.540  0.575
plain over   0.458  0.459  0.459  0.524  0.613  0.679
old          0.458  0.458  0.575  0.575  0.575  0.575
```

### 3b. A first try, measured: hazed puffs laid plainly "over"

The first build wrote `(T a lit, T a)` through the existing premultiplied
"over", so each puff scaled what was behind it by `1 - T a`. The outline went
(the row in section 1 read a flat 161-163), but the difference image against
the before showed every stack of puffs up to 42 levels brighter inside, and
24-37 % of the frame changed by more than 2 levels: a hazed near puff let the
far puff through where it should let the air's light through (the "plain
over" row above). A deep stack near the horizon would add up toward an opaque
white deck however thick the air, against priority 4. Kept as
`docs/screenshots/puff-haze/try1-*`; not shipped.

### 3c. Three targets

Fixed-function blending takes a target's blend factor from that target's own
alpha output, so one target cannot add `T a` and scale by `1 - a`. The hazed
coverage goes into a target of its own:

| target | format | writes | blend | holds |
| --- | --- | --- | --- | --- |
| history | `Rgba16Float` | `(T a lit, a)` | premultiplied over | rgb `C`; alpha the plain coverage |
| distances | `Rg16Float` | `(dist, far)` | `Min` | unchanged |
| puff cover (new) | `R16Float`, the history's size | `(T a, 0, 0, a)` | premultiplied over | r `alpha` |

- `cloud_puff_vertex` measures the air to each corner of the billboard,
  `cloud_air(eye, to/|to|, |to|)` with `to` the corner from the eye, and
  passes it to the fragment as `air`. A billboard faces the eye, so its
  corners lie at almost the same distance and the air across it is smooth;
  six vertices a puff carry it, not every fragment.
- `cloud_puff_fragment` keeps its discard test on the unhazed coverage `a`
  (so the distances target is written exactly where it is today).
- The composite (`clouds`, through `cloud_upsampled`) takes the puff cover's
  `alpha` in place of the history's alpha at the same texels and weights, and
  multiplies by `cloud_air` only when the history is the march's
  (`view.cloud_puff_fade.w < 0.5`). The composite runs only when the puffs
  drew this frame (both need no map overlay and clouds on), so it never reads
  a stale cover.

What does not change:

- **The distances target and its tests.** Still `Min`, still written where
  `a >= 0.004`. The upsample's `cloud_texel_sees` and the soft fade against
  the ground read it as before, so no cloud is carried across a terrain
  silhouette (priority 2) any more or less than today.
- **When a puff appears.** The opacity ramps (`near`, `soft`, `outside`, the
  cover fade on the CPU) are untouched, so there is no new pop-in
  (priority 1).
- **A puff alone, and inside an opaque near puff.** Drawn as before (3a); the
  frames differ where a puff is partly transparent over another.
- **The march renderer.** Its composite path is the same code as before; the
  new target is bound but not read.

## 4. Alternatives

- **Coverage-weighted distance** (blend the distances "over" by coverage, so
  the haze distance slides from the far puff's to the near one's as the near
  one thickens). It removes the hard step but still hazes the far puff to a
  mean distance, it needs an extra "over" channel all the same, and it
  changes the distance every silhouette test reads, which is what keeps the
  clouds off the terrain. Rejected.
- **Fog each puff toward the scene's colour** (the usual particle fog): the
  same result with two targets, but the puff pass would read the scene at the
  0.4 scale, and a hazed cloud in front of a hill would carry the hill's
  silhouette at the coarse texel. Rejected.
- **Hazed puffs laid plainly "over"**: 3b. Rejected.
- **Haze per fragment** in the puff pass: exact, but five `exp`s for every
  puff fragment, overdraw included. The billboard's air is smooth enough to
  interpolate from its corners.
- **Lower `cloud_haze`**: hides the step and loses the horizon haze
  (priority 4). Rejected.

## 5. Verification

- The same two shots, same args, same frame, before and after; crops and a
  difference image in `docs/screenshots/puff-haze/`.
- `shader_tests.rs`: the puff vertex measures the air, the puff fragment
  writes the light and the cover as 3a, and the composite takes the cover and
  hazes only the march; the CPU mirror of 3a.
- The game draws: no shader or pipeline error in the capture's log.
- Frame time is not measured here (`perf_suite.py` is the owner's, on real
  hardware). The composite no longer integrates the haze for each
  full-resolution pixel under the puffs; the puffs integrate it six times
  each, and write one more 16-bit channel at the 0.4 scale.
