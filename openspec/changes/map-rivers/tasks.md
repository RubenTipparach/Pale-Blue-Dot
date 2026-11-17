# Tasks

## 1. River water on the map

- [x] 1.1 `pbd_core::map`: a texel carries its river cover (`river`, thirds
  of the pixel), from `rivers::floods` at a 3 x 3 grid across the pixel, none
  in a town's footprint; `base_texel` gives the point's answer. Tests: the
  point answer is `floods` on seeded directions; the owner's river at the
  version 10 spawn is covered; dry land and the sea are not.
- [x] 1.2 `world_map`: paint a river's share in `sea_colour` at 2 m over the
  shaded ground; the sea unchanged. Test: a river texel is water-coloured, a
  sea texel's colour does not depend on the river field.
- [x] 1.3 The cache: the river in the two spare bits, `p2` in the name. Test:
  pack and unpack keep the river.
- [x] 1.4 Before and after captures of the map at Holbrook in
  `docs/screenshots/map-rivers/` (lavapipe, no GPU).
