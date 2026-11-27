# Tasks: infrastructure-missions

Phase 4a. Section 1 is this write-up and the mockup; the rest waits for the
owner to look at M33's INFRASTRUCTURE tab and ask for the build (CLAUDE.md,
"Write it up before touching code").

## 1. The write-up and the mockup

- [x] 1.1 This change: proposal, design, tasks and the
      `player/infrastructure-tree` spec delta; `openspec validate --all`
      passes.
- [x] 1.2 `early-power` says its missions are this tree's Power branch
      (design 1, 2, 11, 12); `town-missions-tree` notes that its Roads moved
      here, with the owner's words (design 13).
- [x] 1.3 M33 version 12: the INFRASTRUCTURE tab (`#infrastructure`, `#power`
      redirects), Power as a branch, MISSION and RECIPES, the bot run off
      until Step or Play; the TOWN tab without roads; the info modal; icons by
      `tools/art/infrastructure_icons.py`.
- [x] 1.4 Screenshots in `docs/screenshots/infrastructure-missions/`
      (headless Chromium, software rendering, said so).
- [x] 1.5 The tracker's M33 card and roadmap step 4a, `mockup-work.md`.
- [ ] 1.6 Published (the owner's session) and the owner's review.

## 2. Moving the roads (the game; waits on a request)

Another change is editing `missions.ron` and the missions code now, so none
of this is done in this round.

- [ ] 2.1 `assets/config/infrastructure_missions.ron`: Stoneworks and Roads
      with the moved ids, objectives, counts and coins unchanged, and Town to
      town; `town_missions.ron` without the Roads branch, `wheels` off
      `first-job`, `wheel` unlocked by `first-job`. A `pbd-core` test that
      both load as strict trees with one root each and share no moved id.
- [ ] 2.2 A third `Chain` and the `infrastructure_missions` record, opened by
      `first-job` (design 1). Tests: no record before, the record after, the
      town record untouched.
- [ ] 2.3 The migration (design 5): done and open moved missions and their
      counts and focus carried from the town record on open; coins under the
      old `town_missions:<id>` key never paid twice. Tests: a world with
      `pave` done and `join` half done opens with both as they were and pays
      nothing; a rollback build still reads its town record.
- [ ] 2.4 `Chain::recipe_open` over three chains; the wheel open for a world
      with `first-job` done.
- [ ] 2.5 The Missions tab's INFRASTRUCTURE tab, the HUD focus across three
      trees, the toast on opening. The locale keys moved from
      `town_mission.<id>.*` to `infra_mission.<id>.*`.
- [ ] 2.6 Captures: a migrated world's two trees, before and after.

## 3. The new branches (each after its system)

- [ ] 3.1 `pave` `towns` and `street`: Town to town and A street to the door
      (today).
- [ ] 3.2 `site` and `raise` from the building planner's events: Lay a plot,
      Raise a building (today).
- [ ] 3.3 The mason's bench, dressed stone, mortar and the civil kit's
      presets (`bp-*`, design 4) with their rules; `works`: the Civil works
      branch.
- [ ] 3.4 `ask` and `grow`, with town growth on the grid (M15, M22).
- [ ] 3.5 The Power branch, with `early-power`'s build.

## 4. Proof

- [ ] 4.1 The bot plays the tree to its last live mission (CLAUDE.md,
      "Demos"), for the owner's batch.
