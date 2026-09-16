# Proposal: bring the preview's scale and its shader back toward Tenebris

## Why

Two findings, measured rather than felt, both in
`docs/tenebris-comparison.md`.

**The world is 6.7x too big for its avatar.** The walker is Tenebris's at 1:1 -
same 1.6 m eye, same 8 and 14 m/s, same 12 m/s jump - and the ground around it
is not. Measured off the real `dual_sphere(8)`: tiles are 18.883 m across
against Tenebris's 2.832 m, and the elevation step is 6 m against 1 m. Read in
eye heights, a hexagon is 1.8 people wide there and 11.8 people wide here, and a
terrain step goes from something you walk up (0.63 of eye height) to a wall over
twice your height (3.75). The same jump clears 7.3 blocks in Tenebris and 1.3
steps here, so terrain Tenebris lets you hop over is a cliff.

**The faithful shader is the one that is not running.** `hex_terrain.wgsl` is a
term-for-term Tenebris port with every knob a uniform, and it is bound to no
pipeline. The live `planet_surface.wgsl` reimplements a subset with every knob a
literal, and it drops the night-side rim floor, the ambient floor, torch light,
underwater absorption and the Bayer cutout. Two of those show in a still frame:
the night limb falls to black, and lighting is flat across a whole 18.9 m tile.

## What changes

Two things that are cheap and do not touch topology or uploads, and one that
does:

1. Restore the night-side rim floor in the live shader, matching Tenebris's
   configured `distant_rim_floor` and our own unbound port.
2. Lift the live shader's hard-coded rim, fog, terminator and light-tint
   numbers into the params uniform it already fills, so a second planet becomes
   possible without editing WGSL.
3. Decide the preview body's scale deliberately. Tile width is
   `1.209 * R / 2^L`, so at level 8 a 600 m radius lands Tenebris's 2.83 m
   tiles and 250 m lands this project's own 1.18 m target.

## Open decision

Option 3 is the owner's call and is not assumed here. Shrinking the radius moves
nine other tuned numbers with it - the 4,800 m atmosphere shell, the 4,600 m
cloud layer, the 2,300 m foliage range, the 3,200 m draw-budget switch, the
terrain amplitude - and invalidates every capture in the comparison document.
The alternative, scaling the avatar to the world (eye near 10.7 m, walk and
sprint near 53 and 93 m/s, jump near 28 m/s), is equally cheap and abandons the
premise of a person standing on a planet. The third path is the streamed
near-player grid, which is `voxel-engine-foundation` and is the whole remaining
engine.

## Non-goals

- Binding `water.wgsl`. It needs scene colour and depth inputs and its own
  pipeline; `docs/shader-port.md` already carries it as outstanding.
- Interpolating skylight per corner. It is the right fix for the flat-tile
  lighting and it is upload-side work, not a shader edit.
- Any look change landing without a before-and-after capture the owner has seen.
  Green tests do not settle a look.
