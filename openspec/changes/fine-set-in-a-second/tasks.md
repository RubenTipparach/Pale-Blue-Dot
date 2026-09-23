# Tasks

## 1. Measure
- [x] `streaming_cost` on the owner's desktop (i7-9700F): 16,089 ms whole,
      97% in `record`, tier 95 ms. In the proposal.

## 2. Floors only where the shader reads them
- [x] Lay all four bands first, settle `complete`, then record; `record`
      computes `fine_floor` only for a side whose neighbour is within
      `complete[k+1]` plus two tiles, never on the finest level.
- [x] Test `a_floor_is_computed_wherever_the_shader_reads_one`: the shader's
      own test, reflected neighbour and all, against every coarse side.

## 3. Every core
- [x] Lattices laid per level on scoped threads; records built in chunks on
      scoped threads with a memo each, joined in order.
- [x] Test: the parallel build equals a one-thread build byte for byte.
- [x] Re-run `streaming_cost`; record the numbers in the design.

## 4. A dig never waits
- [x] `sample_at` answers from the record where it has no column: solid
      below the cap.
- [x] `ColumnTier::adopt`: generate one solid rim column with its edits,
      link it both ways to its resident neighbours, stamp its record, extend
      the light. `apply_edit` adopts before it edits.
- [x] Test: a dig at a cell outside the tier takes the layer, and the column
      is resident afterwards with its neighbours pointing at it.

## 5. Prove it on the desktop
- [x] Run the release build on the owner's machine: startup time, the
      rebuild landing time from the log, a walk with digging; no
      `edit BLOCKED` line. Numbers in the design.
- [x] The base level on every core (startup 17.0 s to 4.4 s).
- [x] The two requirements pinned by tests moved into
      `openspec/specs/planet/rendering/spec.md`.
