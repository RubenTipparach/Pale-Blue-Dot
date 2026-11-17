# Proposal: far trees lit by their own relief

## Why

The owner, 2026-11-16, on a forested hillside in daylight seen from about
40 m up: "I think trees seem too evenly lit, maybe they need normals or
something? the imposters just dont have any depth at far LODs". In that
shot:

- every far tree is the same flat dark green blob;
- the hillside of canopy is one even tone;
- no crown has a sunlit side, a shaded side or any shading inside it.

Near trees look fine.

Measured (design 1): a mid card or far billboard is lit with **one normal
for the whole tree**, and the crown's shade is **one constant** (0.87). So
every pixel of a far crown takes the same light. With the same trees forced
to their mid cards, the canopy at noon is 21% darker than with their near
models (luma 33.9 against 43.0), and its spread of tones is 40% narrower.

## What changes

- **The bake keeps the near model's relief.** Beside each impostor's colour
  strip, the bake writes a relief strip from the same rasterisation: per
  pixel, the near model's normal (in the view's level frame) and its
  root-to-tip AO. Nothing is guessed: it is the same data the near pass
  lights with.
- **Far trees are lit per pixel by it.** The mid cards and the far billboard
  rebuild each pixel's normal from the relief and take its own AO. A crown
  then has a sunlit side, a shaded side, and darker hollows inside it,
  as the near crown does.
- **One tuning value**, `tree_impostor_relief` in `scatter.ron`, 0..1,
  default 1. Zero gives back today's flat card exactly.

Draw-time only: nothing is saved, and saves are untouched. No new pipeline.

## Not in this change

- A crown shadowing itself in the sun's cascades (a card cannot cast a
  shadow on its own pixels). The baked AO stands in for it (design 4).
- Per-tree tint: the near and far trees already share `cell_variation`.
