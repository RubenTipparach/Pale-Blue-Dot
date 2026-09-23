# Tasks

- [x] `Clock { seconds: f64 }`; `fraction()`, `day()`, `year_fraction()`;
      callers moved to them.
- [x] Sun from the orbit; the sidereal sky rotation; `YEAR_DAYS`. Tests:
      solstices and equinoxes, noon on the meridian all year, a year of extra
      turning summing to one turn, day 0 at noon being `SUN_FIXED`.
- [x] `world_seconds` in `world.ron`, written with the pose, restored on load
      (`a_world_comes_back...` checks it; an older file opens at day 0).
- [x] `--day N` capture flag; the log reports the sun's latitude.
- [x] Captures at noon, day 0 and day 50, side by side
      (`docs/screenshots/seasons-day0-day50-noon.png`). At day 50 the north
      pole has turned into its winter night edge. The sun's latitude is
      logged as -0.7 deg on day 25 at noon (half a day past the equinox) and
      -23.5 deg on day 50.
- [ ] Owner's in-game check.
