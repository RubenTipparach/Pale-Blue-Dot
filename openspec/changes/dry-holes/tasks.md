# Tasks

## 1. Write-up

- [x] 1.1 Measure what fills a dry dug hole (design 1, 2): `dry_hole_report`
  at a beach, a shore and two river banks; captures before. Verify: the
  report's table in the design; `docs/screenshots/dry-holes/before-*`.
- [x] 1.2 Proposal, design, spec delta. Verify: `openspec validate --all`.

## 2. Dry holes

- [x] 2.1 `TierWater::dug` from the save's edits at the build and on every
  edit; the sea's reach does not run down a dug column's air under the sea's
  level, and a dug column with air at the sea's top layer is not reached
  (design 3). Verify: `a_pit_dug_below_the_sea_beside_a_beach_stays_dry`;
  B22's test unchanged; `dry_hole_report` at `39.725 15.339` dry after the
  dig, the rule and a reload.

## 3. The sea flows

- [x] 3.1 The rule's sea clause, `transform_at` and `Grid::sea` (design
  4.2). Verify: `the_sea_fills_what_it_feeds_and_falls_over_a_lip`;
  `tests/water_flow_presets.rs` unchanged.
- [x] 3.2 `ColumnTier::sea_joined`, kept in `TierWater::joined`, refreshed by
  `FineSet::mark_sea` and at the build's end (design 4.3). Verify:
  `a_bucket_in_a_sealed_pit_under_the_sea_is_not_the_sea`; the river settle
  tests unchanged.
- [x] 3.3 The edit fills only the cell it opened; the rule brings the rest
  (design 4.1). Verify: `digging_through_to_the_sea_lets_it_flow_in`,
  `a_channel_cut_from_the_shore_fills_along_it`,
  `a_breached_pit_fills_from_the_sea_over_its_lip`.
- [x] 3.4 Captures after, beside the before (`docs/screenshots/dry-holes/`,
  with a README). Verify: the README's table.

## 4. Next

- [x] 4.1 Dropped (2026-11-19). It was the tide as flow (design 7): the
  sea's cells near the player rising and falling with the tide. The owner
  had the tide itself removed ("Remove literal tide feature"; `no-tides`),
  so there is no tide left to flow, no intertidal beach for a pit to sit in
  and no tide water for a save to tell from a player's. Nothing is built for
  it.
- [ ] 4.2 Frame time of an edit at the coast on the desktop (the extra
  walk, design 6), on the owner's machine. Not measured in the cloud.

## 5. No sea shading in a dry hole (the owner, 2026-11-19)

- [x] 5.1 Measure why a dry dug pit's floor and lower walls are green
  (design 10): the instrument's captures inside and above the beach pit,
  taken after the dig's ground has finished fading in. Verify: the finding
  in design 10 (the water's compose, not shade).
- [ ] 5.2 The terrain writes "water may stand over this" to the scene's
  alpha; the compose takes a pixel for wet only as far as it says (design
  10). Verify: `a_dry_face_is_not_seen_through_the_sea` and the shader
  tests; the compose instrument shows no pixel of the pit wet.
- [ ] 5.3 Captures before and after, inside and above, and one with the sun
  on the pit's floor (`docs/screenshots/dry-holes/`, README). Verify: the
  pit's floor and walls are the beach's tan sand, shade aside.
