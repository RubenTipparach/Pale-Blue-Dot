# Gather by hand: F on a stone or a shrub

## Why

The owner, on the starter missions (2026-11-16): *"I cant gather rocks or
fiber directly by pressing F, I get that you have to dig dirt to get grass,
but if I see like a shrub or something I should be able press F to collect
it. same with a rock"*.

Today the first mission, "Gather by hand", is played only by digging:
- fibre turns up in one dug grass cell in three;
- loose stone turns up in one dug earth cell in three (`pbd_core::gather`).

The pebbles, bushes, dead shrubs and ferns on the ground are only drawn. The
shader rolls them from each cell's id, and nothing on the CPU knows where
they are. A player sees a stone at their feet and cannot pick it up.

## What changes

- **F picks up** the pebble, bush, dead shrub or fern the eye ray meets
  within reach:
  - a pebble gives one loose stone;
  - a bush, a dead shrub or a fern gives one or two fibre.
- **One placement rule.** Where each piece sits is ported from
  `planet_surface.wgsl` to `pbd_core::scatter`, the same rolls and salts. A
  GPU test pins the port to the shader's own output, so the two cannot drift.
- **What was taken is saved.** A `clutter` record per cell holds the pieces
  taken and when. The piece grows back after a few days, as fruit does.
- **The draw leaves taken pieces out.** It reads a short list near the
  camera, as felled trees already are (`trees_taken`), and a piece dissolves
  out over the fade time rather than vanishing in one frame (priority 1, no
  pop-in).
- **The missions count it.** A pick-up counts `gather`, so "Gather by hand"
  can be played with F alone. Its help line says so.
- **The bot** gathers with F in the trunk showcase.

Digging still turns up fibre and stone as it does now.

## Not changing

- Grass tufts and flowers stay decoration. Grass gives fibre by digging.
- No generator version moves. Clutter is derived and never saved; only what
  a player took is.
