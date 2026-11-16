# Proposal

## Why

The owner, 2026-11-16, on a mountain range seen from the air (one dark slate
texture over the whole flank, crossed by regular rows of tan stripes, with no
grass or snow on its flats): "mountains should have dirt/grass or snow up
where it is flat, or portions wherever possible, it should also consist of
some various rock patterns so its not all the same rock on a mountain".

Measured in version 10 (`design.md`, "Measured"):
- a Mountains cell's top is Stone below 200 m and Snow above, and nothing
  else, because `top_material` reads altitude and latitude but never slope;
- every rock (Stone, desert Rock, Ore) draws as one render code, so a
  mountain shows exactly one rock picture (slate) of the sixteen its sheet
  holds.

## What Changes

- **Generator version 11** (`TENEBRIS_V11`, beside V4-V10, which stay cell
  for cell): a Mountains cell that is flat (no neighbour more than a metre
  off) takes alpine grass or dirt below a ragged snow line and snow above
  it; steep cells stay rock; the snow line comes down onto the flats.
- **Mountain grass and dirt drawn as alpine soil**, not as the mountains
  sheet's scree at (0,0).
- **Rock variety at the draw**: the rock picture chosen per cell from the
  sheet's rocks (slate, granite, fractured slate, dark shale, lichen stone)
  by bands of altitude and a coarse noise, so a flank shows strata and
  patches. Draw-time only, so it holds for every version's worlds.
- **The tan rows**: the far fade of a riser under snow or grass takes the
  mountain's own mean, not the lowland ground's tan.

## Impact

- `pbd-core`: `planet_gen.rs` (V11, a slope test in `top_material`),
  `terrain.rs` (`GENERATOR_VERSION = 11`), a digest test of V10 taken first.
- `pbd-app`: `planet_surface.wgsl` (rock tile per cell, alpine grass and
  soil, risers' far colour).
- Saves: no change to any existing world (CLAUDE.md, "Saved games survive
  every change"); new worlds are made on version 11.
- Look changes go to the owner with before-and-after captures.
