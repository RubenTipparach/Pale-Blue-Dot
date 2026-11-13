# Design: one twilight for the sky and the ground

## 1. Why a small planet's sky stays lit

A point h metres above the ground is in sunlight while the sun's depression
under the horizon is less than `acos(R / (R + h))` (the planet's shadow
cylinder, ignoring refraction). R = 4,800 m, the dome's shell top 1.2 R
(`sky.rs::ATMOSPHERE_RADIUS`), density `exp(-a / 0.22)` with `a` the height as
a share of the 960 m shell (`air_density`), so a scale height of 211 m:

| Height | Density | Lit until the sun is under by |
| ---: | ---: | ---: |
| 211 m (1 scale height) | 37 % | 16.7 deg |
| 422 m | 14 % | 23.3 deg |
| 633 m | 5 % | 28.0 deg |
| 960 m (top) | tapered to 0 | 33.6 deg |

Earth: twilight ends at 18 degrees because the air lit at that depression is
above 300 km. Toward the sun the view ray at the horizon runs 3.2 km through
the shell here, a third of the way round this planet, so the horizon on the
sun's side sees air whose own sun is far higher than the camera's.

## 2. The two models today

- Ground (`pbd_core::sky_light::sky_radiance`, the SH every lit surface reads):
  the sun's light on the sky fades over `e` from 0 to -0.12 (sine of the
  elevation; 0 to -6.9 degrees), a twilight glow over 0 to -0.2 (to -11.5),
  then moon and starlight.
- Dome (`sky_atmosphere.wgsl`): single scattering along 16 view samples, each
  lit unless the planet's shadow covers it (`sun_visibility`, a soft
  penumbra); a dusk tint over `sun_height` -0.28 to 0.2.

## 3. The change

Near the ground (camera below 0.3 of the shell, eased out by the shell's
top, the band the daylight veil already uses):

- each sample's sunlight is the camera's twilight, `smoothstep(-0.12, 0.0,
  e)`, the ground's own range, in place of the per-sample shadow test;
- the dusk tint and the horizon's dusk glow fade over the ground's twilight
  band, `smoothstep(-0.2, -0.0, e)` from below, so the last glow on the sun's
  side goes when the ground's does.

From above the band the dome keeps the shadow test, so the terminator and
the limb from orbit are unchanged.

## 4. Alternatives

- Ground follows the dome: physically this planet's, but a night that stays
  lit to 34 degrees: hours of twilight, the opposite of the owner's ask.
- Thinner air: a scale height near 60 m ends twilight by geometry near 18
  degrees, but changes the day sky's gradient and the limb from orbit.

## 5. Measured

Before and after sweeps, seven captures each at 20 N 40 E (`--at 20 40`; map
20 S 40 E), on foot, facing the sun's bearing 10 degrees up, the sun at 0,
-6, -11, -17, -22, -27 and -30.5 degrees: `docs/screenshots/dusk-sky/`.
