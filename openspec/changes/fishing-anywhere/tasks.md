# Tasks

- [x] 1.1 `PlanetWater::surface` reads the column's fresh water (design 1). Verify: a test that a float over a river column floats at the river's top, and over the sea at the sheet.
- [x] 1.2 Schools spawn in fresh water at its top as `River` (design 2). Verify: an app test with a column holding water above the sea.
- [x] 1.3 `School::scale` and the body-size rule (design 3). Verify: a core test that `length_cm` scales; an app test that a small pool's scale is under a big one's.
- [x] 1.4 Checks; a capture of a cast landing in a river.
- [x] 1.5 `hook_jingle` on `Event::Hooked`; `catch_chime` stays on the catch (design 4). Verify: the cue-book test (every cue the code names is in `cues.ron`).

## 2. Swamps, lakes and rivers are fresh (design 6)

- [x] 2.1 `planet_gen::fresh_water_at`: a river's water or a swamp pool, not in a town. Verify: a core test that every swamp pool and river cell is fresh, stands over the catch's 0.6 m line, and open sea is not (495 pools, 897 river cells, none under the line).
- [x] 2.2 The bot's `fresh_shore` takes water one layer deep (0.9 m, was 1.2 m).
- [ ] 2.3 Verify: the bot plays "Sea fishing"'s fresh half on `tree-run-7`, with the frame log on.
