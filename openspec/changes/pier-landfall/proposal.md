# Proposal: a pier's planks lie on the beach, never in it (pier-landfall)

## Why

The owner (2026-11-14, a screenshot of a harbour at night in rain): "this
board thing seems to be clipping in the terrain". Where a pier leaves the
quay and crosses the beach, its planks are drawn inside the sand: the sand's
top is drawn over the planks, so the pier's landward end looks half buried.

Measured (design, "The finding"): on every harbour of the shipped seed, the
first two or three stretches of each pier that starts on the quay (the main,
west, east and hut piers) have the ground under them at exactly the deck's
height, 1 m over the sea. A pier's planks are cut 0.12 m deep *under* its
deck height, so there they lie wholly inside the top layer of the beach, and
their top face and the sand's are one plane. A few stretches further out sit
over natural ground that reaches the deck as well, and at Marenstrand one
stretch of the main pier crosses a bank a metre over it.

## What changes

- A pier stretch whose ground reaches its deck lays its planks **on** the
  ground: their underside on the highest ground under the stretch, their top
  0.12 m over it. Every other stretch is as it was, 1 m over the sea.
- What stands on a pier (its dressing) stands on the same planks.
- Drawing and collision of a template piece only. No stored record, no
  ground, no generator and no layout version changes: a harbour already in a
  save is the same town on the same ground, and only its pier's planks move
  up where they were inside the sand.

The green speckle on the water round the piles, in the same shot, is a
different cause and is reported in the design, not fixed here.
