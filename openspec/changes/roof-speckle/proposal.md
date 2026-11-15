# Proposal: a room face on its light's band edge is lit or not, never both

## Why

The owner, 2026-11-14, with two screenshots: "under roof areas has
zfighting still", and "stair way too". Inside a town house by its own
lamplight, the far part of a ceiling under the roof is covered in fine
speckle, dark against the warm light, with curved fringes; a dark stair
face is covered in fine lit (orange) speckle.

## What was found (design 1-3)

It is not two surfaces fighting for depth. It is one surface fighting a
light's band of height.

- A building's room lights (`cities-in-the-world` decision 7a) each light
  only a band of height round them, their storey or a stairwell's run, and
  `field_lit.wgsl`'s `room_lights` drops a light where
  `rise < span.x || rise > span.y`.
- In the Fieldstone house a newel sconce at 3.2 m lights up to exactly
  6.000 m, which is exactly the top storey's ceiling under the roof (18
  triangles), and another at 4.2 m lights down to exactly 3.000 m, which is
  where faces of the newel stair and its stone lie (17 triangles). Measured
  by an instrument over every village building (design 2).
- `rise` is measured per pixel in the planet's frame, from positions about
  4,800 m from the centre, where an f32 steps every 0.49 mm. A face lying on
  the edge flips in and out of the band pixel by pixel: the lit pixels and
  the unlit ones are the speckle, and the f32 steps' interference with the
  pixel grid draws the curved fringes.
- Reproduced in a fixed-step capture at 22:00 from the Fieldstone house's
  upper floor looking at the newel's ceiling (design 3,
  `docs/screenshots/roof-speckle/before-upper-floor-newel-ceiling-2200.jpg`).
- Ruled out by measurement: the roof soffit 1 cm over the top ceiling (the
  only overlapping room/outside faces in reach of a ceiling) changes no pixel
  of the stair-well view when lifted 0.5 m, or lowered onto the ceiling at
  the spots tried.

## What changes

1. **A band's edges are given slack.** `room_lights` keeps a light over
   `span.x - BAND_SLACK .. span.y + BAND_SLACK`, `BAND_SLACK` 5 cm, the
   margin the candles and the hearth already add by hand: a face built on an
   edge is inside its band by 5 cm, far beyond the f32 step and beyond the
   tilt between a light's own up and its building's up across its reach
   (design 4).
2. `pbd_core::settlement::pieces::BAND_SLACK_M` holds the number, a test
   holds the shader to it, and a test lays Holbrook at the game's radius and
   checks that no level room face in a light's reach lies within a
   centimetre of an edge as the shader computes it (`rise`, in f32).

Not in this change: the hard line where a candle's band stops 0.15 m under
the top storey's ceiling (seen in the same shots; a separate look question),
the 1 cm soffit, and the coincident faces listed in design 2, which did not
show.

## Impact

- `assets/shaders/field_lit.wgsl` (`room_lights`), `pbd_core::settlement::
  pieces` (the constant), tests in both crates.
- Saved games: untouched. Room lights are derived each time a town is cut
  from its record; nothing is stored or regenerated.
- Frame cost: two subtractions per room light per pixel, not measured.
