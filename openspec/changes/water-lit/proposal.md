# Proposal: the water is lit, not luminous (water-lit)

## Why

The owner (2026-11-13): "water foog is also super unnatural looking and
glowing way too much, check with how water has been upgraded in
crab-breeze". CLAUDE.md priority 6 says the same from 2026-09-25: the water
"looks foggy when it is dark out" and "should be a darker blue where it is
deep".

The before shots (`docs/screenshots/water-lit/before-sheet.jpg`, the owner's
beach at `--at 12.948 8.135`) show a saturated cyan sea that is as bright at
noon under rain, at dusk and at night under a black sky. The cause is in
`water.wgsl`:

- The water's body (`deep_color`, 0.0 / 0.12 / 0.28) is drawn as a colour,
  not lit: only a day/night floor (`night_floor` 0.18) scales it. A bright
  blue is emitted whatever the sky does.
- The reflection is two authored daytime constants (horizon 0.10 / 0.36 /
  0.72, zenith 0.03 / 0.18 / 0.55), saturated blues that ignore the cover,
  plus a separate night colour.
- The water's haze ignores the cover and the rain, which the terrain's does
  not, so the sea and the shore beside it fog to different colours.
- The seabed is tinted toward the same unlit `deep_color`.

Crab-breeze took this same water (it ported it from here, `829f79bf`) and
fixed exactly this (its `water-look` change; read-only survey, design 1):
the body is an albedo times the light that reaches the water, the deep sea
is a near-black navy that the reflected sky colours, the shallows are teal,
and the far sea greys into the horizon.

## What changes

- The water's body is an albedo lit by the light on the open sea: the
  terrain's own sky light (L2's coefficients, by the cover) and the sun after
  the air, as dim as the cloud makes it. It darkens at dusk, under cloud and
  at night with the land. `night_floor` goes.
- Crab-breeze's albedos and absorption: deep 0.004 / 0.018 / 0.040, shallow
  0.020 / 0.090 / 0.075 easing to deep over 14 m of water under the pixel,
  absorption 0.32 / 0.035 / 0.012 per metre.
- The reflection is the sky's own radiance along the reflected ray, from the
  same nine coefficients the wet ground mirrors (`sh_radiance`), so it greys
  under cloud and darkens at night. The horizon floor of the Fresnel goes
  from 0.22 to 0.7 so the far sea carries the sky.
- The water's haze is the terrain's: the same density, thickened by cover
  and rain, toward the same overcast-greyed sky colour.
- The seabed's tint is the lit deep colour, not the bare one.
- Underwater, the murk is the lit body attenuated by the eye's depth.

## Not in this change

- Crab-breeze's GGX glint, glitter path, crest scatter and whitecaps.
- The drops on the lens (their own fix) and the rain volume.
