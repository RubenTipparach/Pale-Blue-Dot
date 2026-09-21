# Tasks

## 1. Measure
- [x] `streaming_cost::what_the_near_field_costs_to_build`: the rebuild by
      level and tier, one column alone, the worm gather alone. Numbers in the
      design.

## 2. The tier on its own
- [ ] `ColumnTier` keyed by stable cell ID with a free-list slot allocator;
      `column(id)`, `set_layer`, `repack`, `reconcile` through the ID; the
      finest records restamped on a band swap. Tests: a column keeps its slot
      across a band swap; a freed slot is reused; an edit through the ID lands
      on the same column the march found.
- [ ] `column::build` split: the band rebuild no longer builds the tier; the
      tier is a resource of its own with its own upload in `upload_fine`
      (changed slot ranges, light and materials with them).

## 3. Growth
- [ ] Per frame: the resident set against the camera; candidates within
      `reach_m` nearest first; drop past `reach_m * 1.5`; generate under
      `tier_budget_ms`, at least one; rim columns regenerated carved when
      their ring closes. Test: a walk of 200 m in 10 m steps leaves every
      column within reach resident and every column in the tier either
      interior-carved or rim-solid; no column generated twice.
- [ ] The regional worm gather re-gathered on the pool when the anchor has
      moved a fraction of its reach.
- [ ] `tier_budget_ms` in `ColumnSettings`, validated, with its unit.

## 4. An edit never waits
- [ ] The digging march generates a missing column synchronously and adds it.
      Test: an edit at a cell outside the resident tier succeeds and the
      column is resident afterwards.

## 5. Prove it
- [ ] A headless scripted walk (the swim keys' mechanism, W held) at sprint
      for 30 s with a capture every 5 s: no solid rim inside `reach_m`, and
      the tier's resident count and per-frame generation cost logged.
- [ ] The owner's in-game walk: no wall ahead, digging works while walking.
