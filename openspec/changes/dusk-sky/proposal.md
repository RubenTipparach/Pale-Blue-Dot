# Proposal: one twilight for the sky and the ground (dusk-sky)

## Why

The owner (2026-11-13), on a night shot at 02:00 with the sun 30.5 degrees
under the horizon: "notice how parts of the sky is still bright? we need a fix
to create that dusk lighting", "the lighting on the ground should match the
lighting from the sky", and "im not even sure this atmosphere model works for
this planet of this size".

They are right about the size (design 1). The dome (`sky_atmosphere.wgsl`)
scatters sunlight through this planet's own geometry: a shell to 1.2 R (960 m
over a 4,800 m planet) with a 211 m scale height, about 33 times Earth's for
the planet's size. Air h metres up stays in sunlight until the sun is
`acos(R / (R + h))` under the horizon: 16.7 degrees at one scale height, 33.6
at the shell's top, where on Earth the same air is gone by 18. The ground is
lit by a different model (`pbd_core::sky_light`), Earth-timed: its sky light
is gone by about 7 degrees and its twilight by 11.5. So for hours the dome
glows over a ground at night.

The before sweep (`docs/screenshots/dusk-sky/before-sweep.jpg`) shows it,
and more: an orange slab under a dark wedge at -11 degrees, the planet's
shadow edge as a hard diagonal across the sky at -17, and a lit teal corner
of high air down to -30.5.

## What changes

- Near the ground the dome's sunlight follows the ground's twilight: it fades
  with the sun's elevation at the camera over the same range the ground's sky
  light does (gone by 7 degrees under), and its dusk glow on the horizon
  fades over the ground's twilight band (gone by 11.5). The per-sample planet
  shadow, which is what draws the hard edge across a small planet's sky, is
  not used near the ground.
- From high up the dome is as it is today (the shadow geometry, the
  terminator from orbit); between the two it eases with the camera's
  altitude, as the daylight veil already does.
- No change to the ground's model, the day sky, the sun disc or the clouds.

## Not in this change

- Thinning the atmosphere (a scale height of about 60 m would end twilight
  near 18 degrees by geometry alone) changes the day sky and the limb from
  orbit, priorities 4 and 5; written up as the alternative, not taken.
- Recommendation taken (ask only with screenshots): one Earth-timed twilight
  near the ground. The before and after sweeps go to the owner.
