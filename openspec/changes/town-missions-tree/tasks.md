# Tasks: town-missions-tree

Phase 3. Section 1 is this write-up and the mockup; the rest waits for the
owner to look at M33's Town tab and ask for the build.

## 1. The write-up and the mockup

- [x] 1.1 This change: proposal, design, tasks, the `player/town-tree` spec
      delta; `openspec validate --all` passes.
- [x] 1.2 M33 version 5: the Town tab (`mission-tree.html#town`), the town
      tree drawn as M33 draws the starter tree, its bot run off until Step or
      Play, its description in the info modal; the starter tree on its own
      tab (`#starter`). Screenshots at desktop and phone width in
      `docs/screenshots/town-missions-tree/` (Chromium on SwiftShader).
- [x] 1.3 The tracker's M33 entry and `docs/mockups/mockup-work.md`.
- [x] 1.4 Published to M33's link (the owner's session).
- [x] 1.5 The owner's review; anything turned down comes back to design.md.
      Approved for the game (owner, 2026-11-22, after opening the Town tab:
      "Ok approved").

## 2. The tree

Built 2026-11-22 on the owner's "start building mission tree for phase 3?".
Tests: `pbd_core::missions::town::tests` (the tree's shape, its words, each
kind from its event, coins once, unlocks across both trees, an old world's
proof) and `pbd_app::town_missions::tests` (opening on reaching a town with
the starter record untouched, talking and trading in the shop, paying once
across a quit and reload and a torn write, an old world with a purse, lamps
and abandon). 2.4: `talk`, `job`, `goodwill`, `build`, `float` and `light`
are raised; `stake`, `mark`, `pave`, `haul`, `hitch` and cart travel wait
for 4.1, their missions reading NOT YET.

- [x] 2.1 `assets/config/town_missions.ron`: the 25 missions of design 2, a
      second `Chain`. Verify: a `pbd-core` test that it loads, is a strict
      tree with one root, and that no id is shared with a parent in the
      starter tree.
- [x] 2.2 The `town_missions` record and its opening on the first `Reach
      town` (design 1, 5). Verify: tests that the record does not exist
      before, exists after, and that the starter record is untouched.
- [x] 2.3 Coins on completion, in one durable line with the done mark.
      Verify: a test that a mission pays once across a save and reload.
      As built: the purse's line carries the paid mark and is written first
      (design "As built"); a save writes one record a line.
- [x] 2.4 `any` as a town and the new kinds of design 4 that the build
      raises (`talk`, `job`, `goodwill`, `build`, `float`); the rest
      `awaited`. Verify: a test per kind from its `Happened` event.
- [x] 2.5 Unlocks across both trees (`Chain::recipe_open`). Verify: a test
      that the mixer is locked until "Crush stone" and that a world holding a
      mixer keeps it open.
- [x] 2.6 Old worlds (design 5). Verify: an app test that a world with a
      purse and no record opens the tree with nothing paid and nothing else
      changed.

## 3. On the screen

- [x] 3.1 The Missions tab's STARTER and TOWN tabs, the HUD's one focus, the
      coin line on completion; every word from the locale file (design 7).
      Verify: captures by day and at night in `docs/screenshots/town-missions-tree/`.
      Done: `game-tree.jpg`, `game-panel.jpg`, `game-hud.jpg`,
      `game-tree-night.jpg` (lavapipe, `tools/capture_town_missions.sh`).
      No chime on a finished town mission: the starter tree has none yet.

## 4. The Roads and Carts branches (after `townsfolk` 13 and `land-transport` 5)

- [x] 4.1 `stake`, `mark`, `pave` raised by the paving; `haul`, `hitch` and
      `Travel cart-road` by the carts; their missions no longer pending.
      Verify: tests per kind.
      - 2026-11-23: the Roads half (`townsfolk` design 45): `stake`
        (`stake`), `mark` (`cell`, the count newly marked) and `pave`
        (`gravel`; `joined` for a paved cell beside a town's footprint) are
        raised by `pbd_app::roads::road_hands` with the durable line, and
        are in `Kind::LIVE`; grinder to join are live, their craft and place
        objectives naming `block:grinder` and `block:mixer`. Test
        `each_town_kind_counts_from_its_event` walks the Roads branch. The
        carts' half waits.
      - 2026-11-24: the road planner (`townsfolk` design 42l) raises them as
        the paving did: `stake` with the count of stakes a road takes,
        `mark` with its cells, from `pbd_app::roads::planner::planner_hands`
        on the frame its line is written. The carts' half is raised too:
        `hitch` (`horse`) by `vehicles::carts::hitch_horse`, `haul`
        (`cart`, metres) and `Travel cart-road` by the carts' step; test
        `vehicles::carts::tests::the_carts_raise_hitch_and_haul`.

## 5. The owner's check

- [ ] 5.1 A gameplay demo: a bot with the goal "finish the town tree", each
      branch a section, through the player's own inputs (CLAUDE.md, Demos).
      Verify: the video, for the batch.

## 6. The owner's known issues (2026-11-22; design 8 to 10)

- [x] 6.1 Lamps at dusk (design 8): the board posts its lamps job from
      17:00, the interact key lights a town lamp during dusk, each lamp once
      a job; `Light` live, so "Light the lamps" counts. Verify: a test that
      a lamp lit at dusk counts once and one out of dusk does not.
      Capture: `game-board-dusk.jpg`, the board at 18:00 with its lamps jobs.
- [x] 6.2 Abandon a taken job (design 9). Verify: a test that an abandoned
      job is off the taken list, back on the board, and its lost villager or
      repair record cleared.
- [ ] 6.3 The lost villager's icon and its map and compass marker (design
      10). Verify: a capture of the board's lost row and of the marker.
      Built (the icon, `lost_directions` feeding the map and the compass,
      tested in `a_lamp_counts_and_an_abandoned_job_goes_back_to_the_board`);
      no capture: the owner asked for no screenshots of the testing plan's
      items now, and a lost job is posted on a board only some days.
