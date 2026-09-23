# Design: weather overlays

## Data

`atmospheric-circulation` resamples its cells into cube maps for the clouds.
The overlays add one more cube, 6 x 64 x 64 RGBA16F:
- R holds the selected scalar;
- GBA hold the selected vector, in the body frame, in m/s. It is zero for the
  scalar-only overlays.

It is filled by the same core interpolant as the cloud maps, at the same rate,
and **only while an overlay is showing**, so it costs nothing when off.

The core owns every field an overlay names:
- **relative humidity** is `vapour / q_sat(air_k)`;
- **sunlight at the ground** is stage 1's insolation after cloud;
- **temperature** is `ground_k` plus a base of 15 deg C;
- **precipitation** is `rain_rate` in mm/h, negative for snow.

Each is a public function of the state, so the overlay and the simulation
cannot disagree on a formula.

## Drawing

A fullscreen pass in the water composite chain, after the rain and before the
lens, so drops stay on top.
- For each pixel it finds the planet point: the scene depth reconstructed to a
  body-local position, or the ray's hit on the sea sphere where the depth is
  sky. It samples the overlay cube in that point's direction.
- **Colour**: a ramp per overlay (blue, cyan, green, yellow, red for speeds;
  white to deep blue for rain, with violet for snow; blue to red for
  temperature; black to yellow for sunlight). It is blended at
  `overlay_opacity` over the scene, and faded out where the sun is down only
  for Sunlight, which is zero there anyway.
- **Streamlines** by line-integral convolution in the shader:
  - from the point, step 12 times forward and 12 back along the normalised
    vector, moving the direction by `step_m / radius` each time;
  - sum a hashed noise of each step's cell, weighted by a phase that scrolls
    with time times speed.

  The result is streaks aligned with the flow that crawl along it. A calm
  point gets no streaks, because the vector's length gates them. This is the
  standard way to draw a dense flow field without particles, and it costs 24
  texture reads a pixel, only while the overlay is on.
- The overlay is in the body-local frame like the water, atmosphere and
  terrain, so an offset planet draws the same.

## Controls and legend

- `OverlayMode` is a resource, `Off` by default.
- **M** steps it, and the pause menu has a row per overlay.
- The legend is a Bevy UI panel: the name, a colour bar (a 256 x 1 image built
  from the same ramp table the shader's is written from; a test holds them
  together), the range with units, `Day 12 - northern summer`, and `[M] next`.

## Tests

- The overlay map's resample writes each overlay's field, checked against the
  core's function at the texel's direction.
- The shader's ramp table and the legend's image agree, read from the WGSL.
- The mode cycles through every overlay and back to Off.
- Captures from orbit, one per overlay, at a fixed seed and step. Measured:
  - streamline pixels align with the sampled flow (the mean angle between a
    streak's orientation and the wind at its point is under 20 deg);
  - calm areas carry no streaks.

## What was built, against this design

- **The streamlines are comets, not a convolution.** Each pixel walks twelve
  steps UPSTREAM along the flow; a lattice cell the walk passes through may be a
  seed (8% of them), and a seed sends a head downstream whose position scrolls
  with time at a rate proportional to the local speed. The pixel is lit when a
  head has just passed it, fading over a tail of a third of the streak. That is
  12 texture reads a pixel rather than 24, and it gives streaks with a bright
  head moving the way the flow does, which reads as direction where a symmetric
  convolution only reads as orientation. A seed cell is 0.6 of a step, which is
  how wide a streak is.
- **Cloud and rain fade to nothing at zero**, like a radar map: clear sky and dry
  ground show the scene. The other overlays cover the planet.
- **Ranges, from the climate report**: surface wind 0-8 m/s (band means are
  0-3.5 and the storms faster), jet 0-45, currents 0-0.08 (fastest measured
  0.06), rain -10..10 mm/h with snow negative, sunlight 0-1000 W/m2,
  temperature -30..40 C.
- **The map is hidden, not wrong, while it is rebuilt.** `WeatherMapsNow`
  carries which overlay the GPU map holds; the pass draws only when that is the
  one asked for, so switching overlays never paints one field in another's ramp.
- **The legend** is a Bevy UI panel top right: the name, the colour bar (a
  256 x 1 image from `overlay::RAMPS`), both ends in their unit (the rain's say
  which end is snow), the day and season, and `[M] NEXT`.

What the overlays already SHOW about the simulation, from the first captures:
the jet is at or near its 45 m/s cap across nearly every latitude outside the
tropics, a slab rather than a ribbon. That is a finding for
`atmospheric-circulation`, not an overlay defect.

**Data mode (owner's direction after the first captures):** while an overlay
shows, the clouds pass is skipped and the ground's cloud shadows are off, and
the planet under the map is drawn in greyscale (its luminance), so the only
colour on it is the data's. Relief and coastlines stay readable by brightness.
The cloud overlay is where cloud is shown in this mode. Both follow the one
fact `WeatherMapsNow::overlay_kind`, so the map, the missing clouds and the
missing shadows cannot disagree.
