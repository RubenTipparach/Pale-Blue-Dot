# Proposal: dropped items lit like the ground

## Why

The owner, 2026-11-18, on three dug sand blocks floating in the pit they were
dug from: "whats the glowing cubes?", "they shouldnt be glowing like that",
"it almost looks like its using the sun light", and "are we able to generate
light probes or something to get proper lighting?".

## What Changes

- A drop is lit as the terrain is: the terrain's own sun with its sun
  shadows, and the cell light field (sky and lamps) the ground is shaded from,
  so a drop in a pit, a cave or the night is as dark as the ground there.
- No new probes: the field is already a light probe at every cell.

## Impact

- `drops.rs` `draw`: the drop's entity carries `LitLikeTerrain` beside
  `LitByField`, as a town's pieces do.
