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
