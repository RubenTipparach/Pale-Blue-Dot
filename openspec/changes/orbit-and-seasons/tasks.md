# Tasks

- [ ] `Clock { seconds: f64 }`; `fraction()`, `day()`, `year_fraction()`;
      callers moved to them.
- [ ] Sun from the orbit; the sidereal sky rotation; `YEAR_DAYS`; tests from
      the design.
- [ ] `world_seconds` in `world.ron`, written with the pose, restored on load.
- [ ] `--day N` capture flag.
- [ ] Captures of day 0 and day 50 at noon; day 0 matches the old frame.
- [ ] fmt, clippy, tests, `openspec validate --all`.
