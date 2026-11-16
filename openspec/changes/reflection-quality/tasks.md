# Tasks

## 1. Write-up
- [x] 1.1 Proposal, design and the settings spec delta, with the owner's words.

## 2. The setting
- [ ] 2.1 `config::Reflections` and `ReflectQuality`, extracted; the
      `LightingSettings` reflection switches take the level; MEDIUM's reach.
- [ ] 2.2 `PlayerSettings::reflections`, the file's version 8, an old-file
      test, the `--reflections` flag, `follow_reflections`.
- [ ] 2.3 The PICTURE row (a four-step slider), and RAIN SHADOW moved from
      WEATHER to PICTURE.
- [ ] 2.4 Both terrain pipelines specialised every frame; the history read
      only after a capture the frame before.

## 3. Proof
- [ ] 3.1 Captures at each level, day and night with lamps, in
      `docs/screenshots/reflections-quality/`.
- [ ] 3.2 The perf sitting: four levels, `storm` and `walk`, two repeats,
      release build.
- [ ] 3.3 READY with no `LATE_PIPELINE` at each level and after a switch.
- [ ] 3.4 fmt, clippy, tests, `openspec validate --all`.
