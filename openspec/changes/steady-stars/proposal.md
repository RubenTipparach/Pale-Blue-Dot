# Proposal: stars that do not flicker (steady-stars)

## Why

The owner (2026-11-13): "critical issue: stars are super flickery".

The stars are 1,400 quads on a sphere 180 km out (`desktop/scene.rs`), with
an unlit `StandardMaterial`. Most are 20 m across (`20 + hash^8 * 150` metres
half-width): 0.36 of a pixel at 1,800 px high and a 60 degree field of view,
0.31 at the owner's default 85 % render scale. A quad smaller than a pixel is
drawn only where it happens to cover one of the pixel's samples, so as the sky
turns (`turn_stars`, every frame), the camera moves, and the scaled-down image
is scaled back up, each star winks on and off from frame to frame.

## What changes

- Each star is sized in screen pixels, not metres: a `StarMaterial` whose
  vertex shader puts the quad's corners a fixed number of pixels from the
  star's projected centre (at least 1.25 px half-width, about 2.5 px across,
  more for the few big stars, as now), with a soft round falloff.
- A star's brightness goes into its intensity: the light the old quad would
  have spread over its true footprint, spread over the drawn one, so faint
  stars stay faint and the bright ones stay bright, at any window size or
  render scale.
- Drawn in the opaque pass as now, behind the sky's daylight veil.
