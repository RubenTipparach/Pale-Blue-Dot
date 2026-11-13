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

The physically right picture is each puff faded by the air in front of IT,
then laid over the puffs behind:

```text
A' = T(d_A) * (c_A, a_A)          B' = T(d_B) * (c_B, a_B)
seen = A' + (1 - T(d_A) a_A) * B'
```

As `a_A` goes to 0 at the near puff's edge, `seen` goes to `B'`, which is what
the far puff shows alone: no step, whatever threshold the discard uses.

- `cloud_puff_vertex` measures the air to each corner of the billboard,
  `cloud_air(eye, to/|to|, |to|)` with `to` the corner from the eye, and
  passes it to the fragment as `air`. A billboard faces the eye, so its
  corners lie at almost the same distance and the air across it is smooth;
  six vertices a puff carry it, not every fragment.
- `cloud_puff_fragment` keeps its discard test on the unhazed coverage `a`
  (so the distances target is written exactly where it is today) and writes
  `(lit*a*air, a*air)`.
- `clouds` (the composite) multiplies by `cloud_air` only when the history is
  the march's (`view.cloud_puff_fade.w < 0.5`). When the puffs draw the
  clouds, everything in the history is puffs, already hazed: with the puffs
  configured, the march returns before marching, so even a frame where the
  map overlay stops the puffs and the resolve runs holds only puffs or none.

What does not change:

- **The distances target and its tests.** Still `Min`, still written where
  `a >= 0.004`. The upsample's `cloud_texel_sees` and the soft fade against
  the ground read it as before, so no cloud is carried across a terrain
  silhouette (priority 2) any more or less than today.
- **When a puff appears.** The opacity ramps (`near`, `soft`, `outside`, the
  cover fade on the CPU) are untouched, so there is no new pop-in
  (priority 1).
- **A puff alone.** Where only one puff covers a pixel, its haze was its own
  distance before and is now: the frames differ only where puffs overlap
  (the difference image in `docs/screenshots/puff-haze/`).
- **The march renderer.** Its composite path is the same code as before.

## 4. Alternatives

- **Coverage-weighted distance** (blend the distances "over" by coverage, so
  the haze distance slides from the far puff's to the near one's as the near
  one thickens). It removes the hard step but still hazes the far puff to a
  mean distance, and it changes the distance every silhouette test reads,
  which is what keeps the clouds off the terrain. Rejected.
- **Haze per fragment** in the puff pass: exact, but five `exp`s for every
  puff fragment, overdraw included. The billboard's air is smooth enough to
  interpolate from its corners.
- **Lower `cloud_haze`**: hides the step and loses the horizon haze
  (priority 4). Rejected.

## 5. Verification

- The same two shots, same args, same frame, before and after; crops and a
  difference image in `docs/screenshots/puff-haze/`.
- `shader_tests.rs`: the puff vertex measures the air, the puff fragment
  multiplies its output by it, and the composite hazes only the march.
- The game draws: no shader error in the capture's log.
- Frame time is not measured here (`perf_suite.py` is the owner's, on real
  hardware). The composite no longer integrates the haze for each
  full-resolution pixel under the puffs; the puffs integrate it six times
  each.
