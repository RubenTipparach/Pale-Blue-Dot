# Design: horizon-haze

**On hold (2026-09-26).** This is the deeper-air plan as it stood before the
second measurement (proposal, "Measured again"). It is not the next step:
the owner judges the far deck on the `cloud-reach` build first.

## 1. The sky's air

`sky_atmosphere.wgsl`'s `air_density` is
`exp(-altitude / H) * (1 - smoothstep(0.72, 1.0, altitude))`, with `altitude`
as a share of the shell (960 m) and `H` = `SkyParameters.atmosphere.y`, set at
the spawn in `sky.rs` (0.22). The change is that one number, to the value the
owner picks from the stills (0.35, 0.5 or 0.7). The shell, the taper and the
day veil's fade (288-960 m) are unchanged, so the air ends where it does
today and the stars come out at the same height.

## 2. Holding the zenith

The air straight up from the sea, in shell units, is
`D(H) = integral over 0..1 of exp(-x / H) * taper(x) dx`:

| H | m | D(H) | x today | from 421 m, x today |
| ---: | ---: | ---: | ---: | ---: |
| 0.22 | 211 | 0.215 | 1.00 | 1.00 |
| 0.35 | 336 | 0.320 | 1.48 | 2.74 |
| 0.5 | 480 | 0.410 | 1.90 | 4.64 |
| 0.7 | 672 | 0.494 | 2.29 | 6.64 |

`CLEAR_RAYLEIGH` and `CLEAR_MIE` are divided by `D(H) / D(0.22)`. That ratio
is written beside the scale height in `sky.rs`, with a test that computes it,
so the two cannot drift apart. The overcast's fractions (`overcast_scatter`)
apply to the new clear values as before. From the ground, straight up, the
optical depth, and so the colour, is today's. Toward the horizon, and from
inside the layer and above it, the air is deeper than today's by the rest of
the ratio.

## 3. Clouds

Unchanged. `cloud_air` fades a cloud into what is behind it, and what is
behind a far deck from inside the layer is now the brighter sky.

## Verification

- Stills at the owner's ring frame (`--route clouds --frames 1300`, clouds
  on and off), the ground at noon, 1500 m (`--view column --height 1500
  --pitch -32`), orbit, and the deck at dusk (`--time 20.5`), for each of the
  three depths with the zenith held. The ground's near-zenith pixel within a
  few levels of today's.
- `cloud_haze` 1, 2 and 3 on the chosen depth.
- A unit test that the scattering's divisor is `D(H) / D(0.22)` from the
  shader's own density.
- The perf suite is not needed for a changed constant; the sky pass does the
  same work.

## 4. The sea's horizon line (owner, 2026-11-16)

The owner: "what is this black line on the horizon?" Standing in shallow
sea at dusk under cloud, looking out to sea between two jungle islands, a
thin hard near-black band runs the width of the open sea where it meets the
sky. The same band is in the owner's kelp-field shot at noon and the one
from a beach.

### Reproduced

`--fly` with the capture-only eye path (`PBD_EYE_PATH`) over open sea at
`52.053 -118.900`, facing west (yaw 0, pitch 0), the eye 1.7, 10 and 100 m
over the sea's level, at noon (`--time 1` there), dusk (`--time 7`) and
under a full cover (`--weather-at 20000`: cover 1.00, rain 0.63; the lens
drops off), with `--tide low` and `--tide high` beside the noon one. Mac
M4 Max, Metal (the GPU), 2880x1800, `--frames 120`, fast build.
`docs/screenshots/sea-horizon-line/`.

### Which pass draws it

A measurement line in the sky dome (`sky_atmosphere.wgsl`, the dome's
fragment) painted magenta every pixel whose ray, from inside the shell,
meets the sphere the sky treats as solid ground (`solid_radius`, the
untided sheet's radius). The band came back magenta, every pixel of it and
nothing else: **the band is the sky dome, drawn where no sea covers the
ray**.

| Condition | Eye | Band height, px (median, max) | Band colour | Sky 3 px over it | Sea 3 px under it |
| --- | ---: | --- | --- | --- | --- |
| Noon | 1.7 m | 11, 15 | (16, 65, 105) | (122, 184, 247) | (98, 127, 124) |
| Noon | 10 m | 7, 10 | (42, 104, 158) | (124, 186, 248) | (100, 129, 127) |
| Noon | 100 m | 4, 4 | (81, 158, 230) | (128, 191, 249) | (104, 134, 134) |
| Dusk, low tide | 1.7 m | 11, 13 | (63, 27, 2) | (66, 51, 52) | (7, 31, 49) |
| Dusk, low tide | 10 m | 6, 8 | (53, 22, 7) | (62, 53, 55) | (7, 29, 47) |
| Dusk, low tide | 100 m | 2, 3 | (29, 26, 33) | (50, 60, 65) | (11, 38, 56) |
| Cover 1.0 | 1.7 m | 9, 12 | (19, 34, 51) | (80, 97, 121) | (126, 132, 130) |
| Cover 1.0 | 10 m | 4, 7 | (27, 48, 73) | (75, 92, 116) | (127, 134, 132) |
| Cover 1.0 | 100 m | 2, 2 | (41, 74, 110) | (63, 91, 127) | (128, 134, 134) |
| Noon, low tide | 10 m | 8, 10 | (40, 102, 157) | (124, 186, 248) | (100, 129, 127) |
| Noon, high tide | 10 m | 1 in 4 of 183 columns | - | - | - |

sRGB, the mean over the gap's pixels in 183 columns across the open sea.
The band is a third as bright as the sky over it at noon and darker than
the sea under it in every case, and it is tallest where the owner stands:
low, in the shallows.

### The cause

1. **The sea's sheet sits under the sky's ground.** The sky cuts its rays
   at `solid_radius` (`sky.rs:113`), the sea's level less
   `depth_offset_m`: the untided sheet. That was right when it was written
   (it fixed this same line against the sea-level sphere). Since then the
   sea has a tide (`ocean-environments` design 10, `tide_range_m` 3.0):
   `sea_tide` (`sea.wgsl`) moves the sheet from 1 m under to 2 m over it,
   and every trough of the swell is under it too. Wherever the sheet is
   lower than that sphere, its silhouette is under the sphere's tangent,
   and the rows of rays between the two meet the sphere with no sea drawn.
   At high tide the sheet is over the sphere and the gap all but goes (1
   px in 4 columns of 183, the troughs).
2. **What the dome draws there is near black.** A ray that meets the solid
   sphere is cut at the hit (`sky_atmosphere.wgsl:185`). From 1.7 m the hit
   is about 130 m out, from 10 m about 310 m: a sliver of air, almost no
   opacity, so what shows is the black behind the dome, lifted a little by
   the thin air in front of it. The gap is a few pixels tall (the angle
   between the two horizons is the sheet's drop over the distance to the
   horizon, so it grows as the eye comes down), and dark.

Not the cause: the far water LOD (the gap sits over the sheet's own far
edge in every column, and closes at high tide with the same rings), the
sea's fog (the sheet under the band is fogged to its haze colour), a depth
precision gap (the sky only draws at the far plane), or the clouds (with
`PBD_PASS_OFF=clouds` the band is still there, (59, 125, 182) under a sky
of (121, 186, 248) at noon from 10 m: the clouds pass darkens it a little
more, it does not draw it).

### The fix

The dome stops cutting the rays a sea or a ground was meant to cover. From
inside the shell, a ray that meets the solid sphere takes the air of the
horizon instead: the ray in the same compass direction, a hair (0.5 % of
the tangent's dip) over the sphere's tangent (`horizon_ray`). The gap is
then the colour of the sky just over the horizon, the lit air, at every
height, time and cover; the sea meets the sky in the sky's haze. Where the
sea or the ground covers the ray, which is everywhere else, the dome is
under it and nothing changes.

- Geometry is not moved. The tide, the swell and the flat caps all take
  the sheet under any one sphere somewhere; lowering `solid_radius` by
  the tide's metre would move the line rather than end it (a big swell, a
  lower tide setting), and would change what the sky reads as ground
  from orbit.
- From outside the shell nothing changes: the column over the ground
  (`over_ground`) still needs the hit.
- The sun disc, drawn from inside on the same ray, follows the bent ray:
  a sun on the horizon is not cut off at the sheet's silhouette by a dark
  row.
- Cost: one ray-sphere test the fragment already made, and on the gap's
  few rows a normalise. Nothing a player sees changes but the band.
- Under cloud and in the rain's haze this is not enough: see below.

### Under cloud: the clouds pass stops at the same sphere

With the dome fixed, the gap under clear sky and at dusk came out the
sky's colour to within a few levels, but under full cover it was still a
band: (60, 101, 149) under a sky of (93, 123, 163) from 1.7 m. The clouds
pass (`clouds_over`, `water.wgsl`) lays the cloud over a sky pixel out to
`cloud_far`, which stops a ray at the sea's sphere; a gap ray meets it a
few hundred metres out, short of the cloud layer, so the gap got no cloud
and stayed the clear dome's blue under a greyed sky. The coordinator's
fishing view (`--fish --tide low --time 11`, 24.000 122.460 E, eye 2.6 m,
looking east at about 11:00) shows the same: a 9 px band (median, 13 max),
(25, 43, 63) under a sky of (89, 114, 144) before; (68, 115, 166) under
(92, 120, 155) with the dome fixed alone.

The rain's volume (the rain fog, `rain`, `rain_half`, `rain_compose`)
does the same: it marches a sky ray to the sea's sphere, so the gap got a
few hundred metres of haze where the sky over it got the volume's whole
range. With the dome and the clouds fixed, the fishing view's gap was still
(70, 119, 172) under (93, 126, 169); with `PBD_PASS_OFF=rain` the two
matched.

The sheet is not in the scene's depth, which is why every one of these
passes stops a sky ray at the sea's sphere: that is how they know the
sheet is there. The sheet has its own depth target (`water_depth`, which
the splash reads), so a pixel that is sky in both depths is the gap
(`sea_gap`):

- **The clouds pass** draws a gap pixel as the pixel two pixels over the
  sea sphere's horizon in the same compass direction (`sea_gap_uv`), its
  cloud and its height fog included. The march and its history are not
  touched: only the full-resolution composite reads another pixel for
  these few rows.
- **The rain** does not stop a gap ray at the sea's sphere (`rain_far`,
  `rain`), and its full-size compose weighs the small texels by the sheet's
  distance where the sheet drew (`rain_depth_m`), so the gap's long rain
  does not bleed down onto the sea.
- The sheet's depth is bound to the clouds' composite and the rain
  (binding 20, as the splash's), and kept past the cap pass while either
  draws (it was discarded unless the splash drew). That store is the one
  new cost: a 32-bit depth target written out once a frame on a tiled GPU,
  not measured here (other agents were building; no perf sitting).

### After

Every gap is now within a few levels of the sky over it, and the sea under
it is unchanged (`docs/screenshots/sea-horizon-line/README.md` has the
table). From 1.7 m at noon the gap went from (16, 65, 105) to
(122, 185, 247) under a sky of (122, 184, 247); in the fishing view from
(25, 43, 63) to (95, 124, 161) under (92, 120, 155); at dusk from
(63, 27, 2) to (64, 51, 52) under (66, 51, 52).

### Verification

- The same stills after the fix, the gap measured off the same magenta
  frames: its colour against the sky over it.
- A shader test that the dome bends the ground-meeting rays from inside
  to the horizon rather than cutting them.
- Fast build, `cargo test -p pbd-app` and `-p pbd-core`, fmt, clippy,
  `openspec validate --all`, a launch to READY with no `LATE_PIPELINE`.
- The perf suite is not run (other agents were building on the machine);
  the change adds no pass and no sample.
