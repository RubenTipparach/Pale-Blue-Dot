# Tasks

## 1. Measure

- [x] 1.1 `--new-world <name>` instrument and the `WORLD_SWITCH` log line
      (world, seed, generator, edits, saved pose, walker, spawn direction,
      clock) before and after a switch.
- [x] 1.2 The "before" run: `docs/screenshots/new-world-arrival/before.log`,
      `before-old-make.jpg`, `before-after-switch.jpg`.

## 2. Build

- [x] 2.1 `walking::new_world_up` shared by `setup_walking` and a switch;
      `NewWorldStart` from the launch (decision 3).
- [x] 2.2 The leaving world written and drained first (decision 4).
- [x] 2.3 `loading::begin_switch`, the thread opening the save, and the
      in-place install (`planet::install_prepared`, `towns::replace`,
      `loading::Arrived`) (decisions 1, 2, 7).
- [x] 2.4 The desktop's `arrive`: hotbar, tool, drops, clock (decision 5),
      air, fishery, map raster, walker.
- [x] 2.5 The arrival banner (decision 6).

## 3. Prove it

- [x] 3.1 Tests: a new world made while one is open gets its own save, opens
      with no pose and no edits, and the old save's files are unchanged; a
      new world's walker stands at the new-world spawn, not the last world's
      pose (`saves::tests::a_new_world_made_while_one_is_open_is_its_own_and_leaves_the_old_alone`,
      `walking::tests::a_new_world_starts_at_the_new_world_spawn_not_the_last_worlds_pose`,
      `desktop::arrival::tests`).
- [x] 3.2 The "after" run: the old world, the loading screen during the
      switch, the new world's arrival with its banner, and the log
      (`docs/screenshots/new-world-arrival/README.md`).
- [ ] 3.3 The owner's own launch: NEW WORLD from the saves page.
