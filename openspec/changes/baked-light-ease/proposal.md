# Proposal: baked light eases, it never steps

## Why

The owner, 2026-11-13: "several visual popins: clouds, terrain, and baked
lighting", and "we need some way to better transition those". Priority 1 in
`CLAUDE.md` is "No pop-in": nothing appears, vanishes or changes light in one
frame. Clouds and terrain geometry are other changes; this one is the light
that is baked somewhere and then swapped in whole, so a surface's brightness
steps between one frame and the next.

`frame-budget` 3.2 ("Bake off the main thread, and ease the light so it never
steps") is open, and its spec delta already asks that a town re-baked by a
landing "eases to the new light rather than stepping". `lighting-model` L7
says the newest lamp bake "fades in over the one before it". This change
measures every baked-light swap in the game, says which of them step, and
makes each one ease.

## What was found (from the code; measured in the design)

Five bakes reach the screen by a swap:

1. **The sun's bounce (L4, `planet_bounce.rs`).** Re-baked every 2 degrees of
   sun, about every 16 s of play, and shown at once (`fades: false`). At a
   landing round a new anchor, `bounce_gain` drops to zero the frame the set
   lands, and the bounce fades back in over half a second once the new bake
   comes back from the pool.
2. **The lamp bake (L7).** Cross-fades between bakes of one set as designed,
   but at a landing round a new anchor neither half holds the new set's
   slots, so every baked lamp falls back to the flood in one frame and fades
   back to the bake when it lands.
3. **Dusk and dawn (`switch_dusk_lamps`).** The whole field is re-baked with
   the dusk-lit lamps lit or out, and the GPU's light buffer is rewritten in
   one frame: every street lamp's light on the ground, the trees and the
   clutter switches on or off at once.
4. **A town's outside (`bake_town_outsides`).** Baked a slice a frame and
   swapped in whole when done; and a town the tier stops reaching goes back
   to open sky in one frame. At night that is every lamp's light on its
   walls, on or off, in a frame, and at dusk the walls light some 35 frames
   after the ground under them.
5. **The field-lit things (`light_from_the_field`).** A ship's, a fish's, the
   held tool's and an unbaked town's eight corners are read again when the
   field changes, and written to their row that frame.

## What changes

- One easing rule for every baked light: when a bake replaces another, the
  surface shows the old one and blends to the new over `BAKE_EASE_S`
  (1 s, smoothstep), never in one frame. Where the old bake cannot be read
  any more (it was laid round another anchor), the new set brings its own
  bake with it, so there is nothing to step from.
- L4: two halves on the GPU, as L7 has; the bounce for a new set is baked in
  the same pool task as the set and lands with it.
- L7: the lamp bake for a new set lands with it too, and is shown at once
  over a set whose last bake it continues.
- Dusk and dawn: the light field keeps the field before the relight beside
  the new one and blends the two over `DUSK_EASE_S`. An edit (a dig, a lamp
  placed) still lights at once, as the light spec requires.
- Towns: the bake a town carried is kept per vertex beside the new one and
  the mesh blends from it; going back to open sky blends the same way.
- Field-lit things: when the field changes under a mesh that has not moved,
  its corners ease from what they showed.

## What does not change

- No generation, no save format, no tuning of how bright anything is: the
  light a surface ends on is the light it ends on today.
- The bakes stay where they run now (pool tasks, and the town's 2 ms slices
  on the main thread): no bake costs one long frame.
- An edit's light still changes on its own input frame.

## Impact

`planet_bounce.rs`, `planet.rs` (`upload_fine`, `PlanetParams`),
`planet_lod.rs` (the set's task, `switch_dusk_lamps`), `field_light.rs`,
`towns.rs`, `planet_surface.wgsl`, `field_lit.wgsl`. GPU memory: the light
field and the bounce gain a second half (about 8 MiB and 1 MiB at capacity).
GPU time: the second read only during a fade. Frame cost is not measured by
`perf_suite.py` here (the owner runs it); the bake tasks' times are logged.
