# Tasks

## 1. Measure
- [x] `streaming_cost` on the owner's desktop (i7-9700F): 16,089 ms whole,
      97% in `record`, tier 95 ms. In the proposal.

## 2. Floors only where the shader reads them
- [ ] Lay all four bands first, settle `complete`, then record; `record`
      computes `fine_floor` only for a side whose neighbour is within
      `complete[k+1]` plus two tiles, never on the finest level.
- [ ] Test `a_floor_is_computed_wherever_the_shader_reads_one`: the shader's
      own test, reflected neighbour and all, against every coarse side.

## 3. Every core
- [ ] Lattices laid per level on scoped threads; records built in chunks on
      scoped threads with a memo each, joined in order.
- [ ] Test: the parallel build equals a one-thread build byte for byte.
- [ ] Re-run `streaming_cost`; record the numbers in the design.

## 4. A dig never waits
- [ ] `sample_at` answers from the record where it has no column: solid
      below the cap.
- [ ] `ColumnTier::adopt`: generate one solid rim column with its edits,
      link it both ways to its resident neighbours, stamp its record, extend
      the light. `apply_edit` adopts before it edits.
- [ ] Test: a dig at a cell outside the tier takes the layer, and the column
      is resident afterwards with its neighbours pointing at it.

## 5. Prove it on the desktop
- [ ] Run the release build on the owner's machine: startup time, the
      rebuild landing time from the log, a walk with digging; no
      `edit BLOCKED` line.
