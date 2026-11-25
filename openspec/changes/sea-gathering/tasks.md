# Tasks

The owner's request of 2026-11-25 ("gather specific aquatic life", and
"collect and sell them, crush them for fertilizer, or use them as
decoration"). Written up first, then built on `claude/phase-2-1`.

## 0. Write-up

- [x] 0.1 Proposal, design, spec delta, tasks; `openspec validate --all`.

## 1. Core (`pbd-core`)

- [x] 1.1 `sea_gather`: the roster (model, good, regrowth, crush yield,
  ornament), the `sea-take` and `ornament` records, `hand_refusal`.
- [x] 1.2 Goods rows, values, grinder and campfire recipes, locale keys.
- [x] 1.3 `farming::Plot::fed` (serde default) and the feeding rule
  (`feed`, one more at harvest); tests.

## 2. App (`pbd-app`)

- [x] 2.1 `fish::places`: a piece id and gatherable kind from `plan`; taken
  pieces left out after the trim; a take rebuilds its tile at its fade;
  the aimed piece exposed.
- [x] 2.2 `desktop::sea_gather`: F gathers a piece or a crawler; protected
  refused; the record and the pack in one line; the toast.
- [x] 2.3 Ornaments: right-click sets one, F takes it up; drawn with the
  reef model, `LitByField` and `LitLikeTerrain`.
- [x] 2.4 Fertilise: F on an unripe crop with fertiliser in the pack.

## 3. Art

- [x] 3.1 `dump_sea_models` (ignored test) and `tools/art/sea_gather_icons.py`;
  icons in `assets/items/goods/`, rows in `PROVENANCE.md`.

Tests: `sea_gather::tests` (3: every piece a valued, crushed good; the
regrowth clocks and record bodies; the hand's refusals),
`farming::tests::fertiliser_speeds_the_rest_and_adds_one`, the recipe-book
counts in `station` and `cooking`; in `pbd-app`,
`fish::places::tests::a_reef_s_pieces_have_steady_ids_and_a_take_leaves_one_out`
and `the_aim_meets_a_piece_by_its_stem`. The roster's ids found a
collision on the first run: a coral grown on a boulder hashed `k ^ 0xb`,
which is also a neighbouring lattice point's key; grown and filler pieces
now take salts of their own.

## 4. Captures (lavapipe, `--capture`, `tools/capture_sea_gathering.sh`)

- [x] 4.1 A reef before and after a take; a kelp bed before and after.
- [x] 4.2 The pack with the new icons.
- [x] 4.3 An ornament on land at night.

- [ ] 4.4 Not yet shot: a crawler taken, a protected creature refused, a
  crop fertilised, an ornament on the sea floor.

## 5. Owed to the owner

- [ ] 5.1 Frame cost measured on real hardware (not done in this session).
