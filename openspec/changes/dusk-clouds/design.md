# Design: dusk-clouds

## 1. The sky's light on a cloud

What the ground gets from the sky is the cosine-weighted mean of
`sky_radiance` over the upper hemisphere (the DC of `sky_sh` without the
ground below). Measured off the Rust model (a port in python, 24 x 48
directions), clear and overcast:

| Sun | Clear | Overcast |
| ---: | --- | --- |
| night (-20) | (0.0092, 0.0128, 0.0235) | (0.0335, 0.0365, 0.0457) |
| -8 | (0.012, 0.017, 0.033) | (0.037, 0.040, 0.050) |
| -4 | (0.033, 0.052, 0.078) | (0.057, 0.061, 0.069) |
| 0 | (0.0563, 0.0979, 0.1404) | (0.0970, 0.1019, 0.1071) |
| 30 | (0.077, 0.145, 0.225) | (0.155, 0.163, 0.172) |
| 90 | (0.0833, 0.1619, 0.2694) | (0.1674, 0.1764, 0.1878) |

`pbd_core::sky_light::cloud_sky(e, cover)` is a closed form of it that a
shader can afford per fragment: with `N`, `H`, `Z` the night, horizon and
zenith rows above for each cover,

`A = N + (H + (Z - H) smoothstep(0, 0.5, e) - N) smoothstep(-0.16, 0.02, e)`,

the two covers mixed linearly as `sky_sh_pair` mixes them. The fit (searched
over the two smoothstep ranges) is within 23 % of the measured mean at every
elevation and both covers; a test holds it within a quarter against the
model itself, so the two cannot drift apart unseen.

The cloud's sky term becomes

`sky = cloud_ambient_sky * mix(A(e, cover), DAY_SKY, smoothstep(-0.02, 0.15, e))`

with `DAY_SKY = (0.55, 0.68, 0.92)`, today's day blue. By day (the sun above
8.6 degrees) it is today's term exactly; the share above the sky's own light
is the sunlight scattered inside the cloud, and it goes with the sun. The
ground's bounce takes the same share in place of `day`. The night floor
(`cloud_night_floor`) goes from 0.045 to 0.008: the sky's night row already
holds the moon's light and an overcast deck's glow.

## 2. The sun on a cloud

`cloud_sun(e, h)` with `h` metres above the sea:

`lift = 0.22 clamp(h / 960, 0, 1)`,
`S = sun_air(e + lift) / SUN_NOW * smoothstep(-0.12, 0, e) / max(smoothstep(-0.02, 0.01, e + lift), 1e-3)`.

It is `sun_high`'s form (the dome's upper-sky light: the sun from 0.22
higher, faded by `smoothstep(-0.12, 0, e)`), with the lift in proportion to
the height in the dome's shell. Normalised by `SUN_NOW`, the sun at 60
degrees, so the noon cloud is white as today.

| Sun | Base (300 m) | Top (750 m) | Today (any height) |
| ---: | --- | --- | --- |
| 30 | (0.82, 0.79, 0.73) | (0.85, 0.83, 0.79) | (0.88, 0.85, 0.81) |
| 0 | (0.25, 0.13, 0.03) | (0.54, 0.41, 0.22) | (0.31, 0.19, 0.11) |
| -2 | (0.09, 0.03, 0.00) | (0.37, 0.26, 0.12) | (0.15, 0.09, 0.05) |
| -4 | (0.01, 0.00, 0.00) | (0.14, 0.09, 0.03) | (0.04, 0.02, 0.01) |
| -6 | 0 | (0.01, 0.01, 0.00) | 0 |

(all times `cloud_sun * 1.6`, the puff's factor.) The tops keep the sun's
last red light longer than today and the bases lose it sooner; the sky term
(section 1) is what falls most.

The march's `cloud_sun_sky` skipped the sun march below `e = -0.10`; it now
skips it where `cloud_sun` is nought.

## 3. The stars

`clouds` (the composite), with the puffs drawing the clouds and the pixel's
ray going on into the sky (`far` at the far cap):

- the upsample returns the puffs' own coverage `a` (the history's alpha)
  beside the hazed coverage `aT` (the puff cover target);
- the air's light is the scene at the pixel and four neighbours three pixels
  off along the axes, the dimmest of those whose depth says sky: a star is
  at most about three pixels across (`steady-stars`: 1.25 px half-width for
  most), the dome's glow is smooth over six;
- `behind = scene (1 - a) + light + air (a - aT)`.

With no haze (`T` = 1) it is today's `scene (1 - a) + light`. With the sky
behind and no star, `air` is the scene and it is today's again. Where a star
is, the cloud hides it by its own coverage. A pixel with a surface behind is
unchanged. The march renderer is unchanged.

## 4. Tests

- `sky_light::tests::the_cloud_sky_follows_the_sky`: `cloud_sky` within 25 %
  of the measured hemisphere mean, every 2 degrees, both covers.
- `sky_light::tests::the_cloud_sun_is_the_domes_twilight`: a top is sunlit
  after the base; none of it past 7 degrees under; the noon cloud within 10 %
  of white.
- `shader_tests`: the shaders' `cloud_sky` and `cloud_sun` constants are the
  core's; the puffs and `cloud_light` call them; the composite takes the
  dimmest sky neighbour and the own coverage under the puffs.
