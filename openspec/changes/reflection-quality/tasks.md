# Tasks

## 1. Write-up
- [x] 1.1 Proposal, design and the settings spec delta, with the owner's words.

## 2. The setting
- [x] 2.1 `config::Reflections` and `ReflectQuality`, extracted; the
      `LightingSettings` reflection switches take the level; MEDIUM's reach.
- [x] 2.2 `PlayerSettings::reflections`, the file's version 9, an old-file
      test, the `--reflections` flag, `follow_reflections`.
- [x] 2.3 The PICTURE row (a four-step slider), and RAIN SHADOW moved from
      WEATHER to PICTURE.
- [x] 2.4 Both terrain pipelines specialised every frame; the history read
      only after a capture the frame before.

## 3. Proof
- [x] 3.1 Captures at each level, day and night with lamps, in
      `docs/screenshots/reflections-quality/`.
- [ ] 3.2 The perf sitting: four levels, `storm` and `walk`, two repeats,
      release build.
      Not run (2026-11-16): the Mac never went quiet in 90 minutes of
      polling (the screen locked throughout, Chrome open, other agents'
      builds), so no frame cost of any level is measured. The sitting, on the
      release build, with the fog's A/B (weather-rain 7.4) in it:
      `python3 tools/perf_suite.py --scenarios storm,walk --repeats 2
      --out output/perf/2026-11-16-reflections-quality
      --variant "high||--reflections high" --variant "medium||--reflections medium"
      --variant "low||--reflections low" --variant "off||--reflections off"
      --variant "fog-off||--rain-shadows off;PBD_HEIGHT_FOG=0"
      --variant "fog-on||--rain-shadows off"`, then the report into
      `docs/benchmarks/2026-11-16-reflections-quality/`.
- [x] 3.3 READY with no `LATE_PIPELINE` at each level and after a switch
      (`PBD_CYCLE_REFLECTIONS`, 737 switches from HIGH and 22 from OFF, none
      late; `docs/screenshots/reflections-quality/README.md`).
- [x] 3.4 fmt, clippy, tests, `openspec validate --all`.
