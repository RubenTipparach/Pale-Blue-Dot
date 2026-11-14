# Tasks

## 1. Measure

- [x] 1.1 `--new-world <name>` instrument and the `WORLD_SWITCH` log line
      (world, seed, generator, edits, saved pose, walker, spawn direction,
      clock) before and after a switch.
- [x] 1.2 The "before" run: `docs/screenshots/new-world-arrival/before.log`,
      `before-old-make.jpg`, `before-after-switch.jpg`.

## 2. Build

- [ ] 2.1 `walking::new_world_up` shared by `setup_walking` and a switch;
      `NewWorldSpawn` from the launch (decision 3).
- [ ] 2.2 The leaving world written and drained first (decision 4).
- [ ] 2.3 `loading::begin_switch`, the thread opening the save, and the
      in-place install (`planet::install_prepared`, `towns::replace`,
      `loading::Arrived`) (decisions 1, 2, 7).
- [ ] 2.4 The desktop's `arrive`: hotbar, tool, drops, clock (decision 5),
      air, fishery, map raster, walker.
- [ ] 2.5 The arrival banner (decision 6).

## 3. Prove it

- [ ] 3.1 Tests: a new world made while one is open gets its own save, opens
      with no pose and no edits, and the old save's files are unchanged; a
      new world's walker stands at the new-world spawn, not the last world's
      pose.
- [ ] 3.2 The "after" run: the old world, the loading screen during the
      switch, the new world's arrival with its banner, and the log.
- [ ] 3.3 The owner's own launch: NEW WORLD from the saves page.
