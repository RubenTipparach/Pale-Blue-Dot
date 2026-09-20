# Tasks

## 1. The field, in the core
- [x] `pbd_core::light`: the seed, the flood, and the corner rule, with the
      reference's own constants (15, one per step, the 1.00/0.85/0.70 ladder).
- [x] Tests: open ground is full daylight, a tunnel falls off by one a cell, a
      tunnel past the range is black, an open column lights a cave at its own
      height, nothing leaks in from off the region, and the contact ladder is
      a ladder.

## 2. The bake, in the tier
- [x] `ColumnTier::relight` after the columns are finished, so the rim's solid
      ring and every edit are already in them.
- [x] `gpu_light`, four layers to a word, and the startup line reports what
      the bake cost and how many air cells the sky never reached.

## 3. To the GPU
- [x] A read-only storage binding beside the column records, uploaded in the
      same pass.
- [x] Zero is DARK, so a slot with no bake yet is not a lit cave.

## 4. The shader
- [x] `sky_at`, `solid_at` off the run words, `corner_light` with the ladder,
      `air_above` and the wall rule with its floor-contact fallback.
- [x] Applied to the terrain cap, the terrain wall, the column caps and the
      column flanks; one everywhere outside the tier.
- [x] The ambient floor, and the three colour branches collapsed into one.
- [x] The burial stand-in and its two config knobs deleted.
- [x] A test reads the shipped WGSL and holds its constants to the core's.

## 5. Prove it
- [x] Captures: the cave, the mouth and the meadow, with frame means.
- [x] Measured: the bake, the buffer, and the meadow unchanged as the control.
- [ ] The owner's in-game confirmation, which is the only thing that counts
      for feel: whether a cave is dark enough to want a light in, whether the
      crease where a block meets the ground reads as contact, and whether the
      falloff at a mouth is the right length.

## 6. Held, with the reasons in the design
- [ ] Torches and the block channel, on the reference's proximity model rather
      than its abandoned BFS one.
- [ ] An incremental relight, if six milliseconds ever stops being affordable.
- [ ] Light on the clutter, per corner rather than per cell.
