# Proposal: a floor beside a wall reads one layer of the baked light, never two

## Why

The owner, 2026-11-14, with two screenshots ("not sure whats going on here";
"it looks like zfighting, but I'm not sure if thats it, since this is on
normal gorund"): on plain terrain, a sand floor at the foot of 1 m terrain
walls carries a band of fine dark per-pixel speckle, densest at the wall's
foot and thinning out over about 0.5-1 m; the same fringe lines the lit sand
along the terrace walls of Holbrook's lane at night under its street lamps.

## What was found (design 1-3)

It is not z-fighting. It is one floor reading the baked lamp light (and the
baked sun bounce) from two different layers, pixel by pixel.

- Reproduced in a fixed-step capture on the sandy coast at `--at 30 177.5`
  with a torch in hand: grey-blue speckle on the shaded sand within a metre
  of each wall (design 1).
- Bisected by switches, one at a time, same spot: `PBD_LAMP_BAKE=0` and
  `--no-shadows` remove it; `PBD_BOUNCE=0`, `PBD_SOFT_SHADOWS=0` and
  `PBD_MSAA=1` (no multisampling) leave it exactly as it was. The geometry
  is the same in every run, and one sample per pixel leaves it unchanged, so
  no two surfaces are fighting for depth (design 2).
- `lamp_near` (and `bounce_near`, written the same way) samples the bake
  half a metre off the face, at `alt = |q| - R - BASE - 0.5`, and takes the
  layers `floor(alt)` and `floor(alt) + 1`. For a FLOOR, `q` is the middle of
  the air layer over it and `alt` is a whole number, within the 0.49 mm f32
  step at 4,800 m. So `floor(alt)` flips between the layer under the floor
  and the layer over it from one pixel to the next - the `roof-speckle`
  lesson on a different boundary.
- In the floor's own column both answers agree. Within 0.8 m of a border
  `column_pair` blends in up to half of the NEIGHBOUR's column, and where the
  neighbour is the wall the two answers differ: one pixel reads the wall's
  rock (no lamp, no bounce: black) and halves the light, the next reads the
  air on top of the wall. That is the dark speckle, and the 0.8 m blend is
  the width of its band (design 3).

## What changes

1. **The sampled altitude is snapped to a whole layer when it is within
   `LAYER_SNAP_M` (1 cm) of one**, in one helper used by both `lamp_near` and
   `bounce_near`, so a floor or a ceiling reads its own air layer at every
   pixel. 1 cm is twenty f32 steps at 4,800 m and a hundredth of the layer
   blend, so nothing but a face lying on a layer boundary moves.
2. A test holds both samplers to the helper and the constant, and measures,
   in f32 at the game's radius, that a floor's `alt` does straddle a whole
   number (the flip is real) and always lies within the snap of it.

Not in this change: whether a neighbour that is rock at the sample's layer
should be blended in at all (the band takes the air above the wall, as most
of its pixels already did), and the PCSS filter's soft edge, which shows a
fine regular pattern but no speckle.

## Impact

- `assets/shaders/planet_surface.wgsl` (`lamp_near`, `bounce_near`, a new
  `sample_layer` helper), a test in `shader_tests.rs`.
- Saved games: untouched. The bake is derived; nothing is stored.
- Frame cost: a round and a compare per lamp or bounce sample, not measured
  (no perf run on this change).
