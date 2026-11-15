# Proposal: clouds lit by the dusk the sky has, with no stars through them (dusk-clouds)

## Why

The owner, 2026-11-14, from the ground at dusk turning to night: "clouds seem
a bit white for dusk". The sky overhead is black with stars, a faint orange
glow sits low on the sun's side, and a big cumulus mass across the view is a
bright near-white grey, brighter than anything else in the frame, as if lit
by day. And stars show through that mass.

Reproduced at the default spawn, `--time 21.3` (the sun 4 degrees under),
`--weather-at 7200` (`docs/screenshots/dusk-clouds/`).

## What was found

### The clouds have their own sun and sky, which outlast the ground's

The ground and the dome are lit by `pbd_core::sky_light` (`lighting-model`):
the sun after the air (`sun_air`), gone at the horizon, and the sky's light,
whose twilight the dome now ends by `smoothstep(-0.12, 0.0, e)`
(`dusk-sky`). The puffs (`cloud_puff_fragment`) and the march
(`cloud_light`) still light a cloud by the older rule:

- the sun at `smoothstep(-0.10, 0.15, e)` in a fixed orange-to-white;
- the sky as `mix((0.03, 0.04, 0.07), (0.55, 0.68, 0.92), day)` times
  `cloud_ambient_sky`, a fixed day blue that also stands in for the sunlight
  scattered through the cloud;
- the ground's bounce by the same `day`, and a night floor of 0.045.

Against the sky's own light (the mean the ground sees from the upper
hemisphere, measured off `sky_radiance`, design 1):

| Sun | Ground's sky light (clear) | Cloud's sky term today |
| ---: | --- | --- |
| 30 | (0.077, 0.145, 0.225) | (0.40, 0.49, 0.65) |
| 0 | (0.056, 0.098, 0.140) | (0.18, 0.22, 0.29) |
| -4 | (0.033, 0.052, 0.078) | (0.08, 0.09, 0.11) |
| -8 | (0.012, 0.017, 0.033) | (0.065, 0.071, 0.091) |

By day the cloud's term is three to five times the sky's, because it carries
the sun's light scattered through the cloud too. At dusk it keeps that day
blue's share while the sky's own light falls: at the sun 8 degrees under the
cloud is lit five times what the ground is, and the eye's exposure, which
opens up for the dark ground, carries it to near white. The night floor
alone (0.045) is four times the ground's (`AMBIENT_FLOOR`, 0.006 to 0.011).

### The stars are behind the cloud's haze, not behind the cloud

The stars are quads in the opaque pass (`steady-stars`); the cloud composite
lays the cloud over the scene after them as `scene (1 - aT) + light`, with
`T` the air in front of the cloud (`puff-haze`). From the ground a cloud a
kilometre off keeps `T` = 0.42 of its coverage, so 58 % of every star behind
it shows through it. Fading a cloud into the sky behind it stands for the
air's own light (`horizon-haze`); the stars are not that light.

## What changes

- **The sun on a cloud is the dome's.** `sun_air` at the cloud's own sun
  elevation, raised for its height as the dome raises its upper sky (`+0.22`
  at the shell's top, 960 m, in proportion below), and gone by the dome's
  `smoothstep(-0.12, 0.0, e)`: at sunset the tops are orange and the bases
  already out of the sun; by 7 degrees under none of the cloud is sunlit.
  Normalised by the noon sun (`SUN_NOW`), so a cloud at noon is as white as
  it is today.
- **The sky on a cloud is the sky's.** The sky term is the sky's own mean
  light at the cloud's sun elevation and cover (`pbd_core::sky_light::
  cloud_sky`, a fit to `sky_radiance` within a quarter everywhere, design
  1), rising to today's day blue only as the sun is up
  (`smoothstep(-0.02, 0.15, e)`), since that share is the sunlight inside
  the cloud. The ground's bounce follows the same share. The night floor
  goes to 0.008, the ground's.
- **The march and the near fog light a cloud the same way** (`cloud_light`
  and `cloud_sun_sky` in `clouds.wgsl`), so flying into a cloud at dusk
  keeps its colour.
- **The cloud hides the stars.** Where the sky is behind the puffs, the
  composite hides the scene by the cloud's own coverage and lays the air's
  light from the sky with the stars taken out (the dimmest of the pixel and
  four neighbours three pixels off, sky pixels only: a star is a point, the
  sky's glow is smooth). Where a surface is behind, it is as today.

## Not in this change

- The dome, the ground's light and the stars themselves.
- The cloud's shape and opacity: `space-clouds`.
- Moonlight on the clouds as a directional light: the sky's night term
  already carries the moon's share the ground gets.
- Frame time: not measured. The composite gains four scene taps and depth
  loads on pixels with cloud in front of the sky.
- Recommendation taken (ask only with screenshots): the values above; the
  before and after stills go to the owner.
