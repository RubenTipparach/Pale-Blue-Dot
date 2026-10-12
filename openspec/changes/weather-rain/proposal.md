# Proposal: a weather field, and what rain does to water and ground

## Why

Three of the five terms the water port is missing (`docs/tenebris-comparison.md`,
the water section) need one input this project does not have: a rain
intensity. In Tenebris that one number drives four things at once - the rain
ripples on the water cap, the lens droplets, the terrain's wetness (a rippled
wet sheet, impact rings, rivulets down every side face, a darkening, a sky
sheen and a sun glint) and the precipitation itself - and none of the four can
be added without it.

## What changes

A `Weather` resource carrying a rain intensity in `0..1` and a wetness that
follows it on `wet_fade_tau_s`, with a `--rain <0..1>` launch argument and a
key to cycle it. From that one number:

1. **The cap** adds Tenebris's Zavie raindrop gradient into its wave gradient
   (`u_rain`), so drops perturb the normal, refraction and foam.
2. **The terrain** gets the `hex.fs` rain block ported into
   `planet_surface.wgsl`, gated by sky light and by being above the waterline.
3. **The lens** droplets in the composite's lens pass read the same intensity.
4. **Precipitation**: the near shower, a tangent disk of streaks around the
   camera that fall from a column height to the surface or the water surface,
   suppressed underwater and below ground, rebuilt on the CPU each frame as one
   mesh.

Every knob is `weather.ron`, with Tenebris's `weather.yaml` values and units.

## Non-goals

- A per-column cloud and moisture field, and with it snow and the distant storm
  shafts. Tenebris's `core::weather` derives rain from cloud density; this
  project's clouds are noise inside the sky shader, and evaluating that noise
  again on the CPU is the two-copies defect. Rain is global until a shared cloud
  field exists, and the shafts wait for it.
