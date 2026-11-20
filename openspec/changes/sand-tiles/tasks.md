# Tasks

The write-up and the mockup are this change's first commit. The game's art and
code wait for the owner's approval of the Sand tab, and a request.

## 0. Write-up and mockup

- [x] 0.1 The write-up: proposal, design, spec deltas, tasks.
- [x] 0.2 Candidate tiles drawn in code: `docs/mockups/sand/sand_tiles.py`
  (`sand_tiles.png`, `sheet.png`), seamless, no blue, no outline.
- [x] 0.3 Mockup: the Sand tab on M13 (`docs/mockups/foliage.html#sand`,
  version 4), Today / New for the beach and the desert, day and night with
  lamps, a compare split and the flat tile panel; screenshots in
  `docs/screenshots/sand-tiles/` (SwiftShader, no GPU).
- [x] 0.4 The owner approves the look for the game (owner, 2026-11-20, in chat:
  "approve sand and mountains, and rivers": the Sand tab, M13 version 4).

## 1. The tiles into the game (after approval)

- [x] 1.1 Write the approved tiles into `assets/tilesets/beach.png` (0,0),
  (1,0), (2,0) and `desert.png` (0,0), (1,0) at the shader's sample grid, and
  rebake `atlas.png` with `tools/build_tileset_atlas.py`; `--check` passes.
  Written by `tools/paint_sheet_tile.py` from `docs/mockups/sand/sand_tiles.png`
  (the approved candidates), which inverts the shader's sample grid and
  reads every texel back.
- [x] 1.2 `biomes.json`: the beach accent reads "pink and white shell".
- [x] 1.3 `TUFT_RAMPS` beach and desert rows from the test's printout.
- [x] 1.4 A test: no blue or teal texel in the beach's (0,0), (1,0), (2,0);
  the desert top's red over blue at least twice the beach's (spec
  "Beach sand and desert sand are drawn apart").
- [ ] 1.5 Capture shots of a beach and a desert, before and after, into
  `docs/screenshots/sand-tiles/`.

## 2. Desert sand as its own block (design 5 (c))

- [x] 2.1 `Material::DesertSand` at the end of the enum, its code at the end of
  the save's material table; an old save round-trips unchanged.
- [x] 2.2 `home_biome(DesertSand) = Desert`; render code 1; pack name
  "desert sand"; `Material::is_sand` for the arms where both act alike (dig
  class, sound, bounce, bait, recipes).
- [x] 2.3 `yield_of` (via `pbd_core::column::dug_sand`): generated sand in a desert cell gives desert sand; placed
  sand gives sand. Tests for the three scenarios of "Sand dug in a desert is
  desert sand".
- [x] 2.4 No generator version: the digest tests of every carried version
  pass unchanged.

Built 2026-11-20 on `claude/phase-2-1`. The requirements moved to
`openspec/specs/planet/rendering` and `openspec/specs/world/editing` with their
tests. Desert sand is save code 29. The bilberry, recipes and other places
that take sand take either through `Material::is_sand` where they match on it
(gather, till, dig class, sound, bounce).
