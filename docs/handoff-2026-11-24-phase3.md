# Handoff: phase 3 in the game (2026-11-25, evening)

**PR #150 merged to main** (2026-11-25, on the owner's "MERGE PR TO MAIN").
Work since is on `claude/phase-2-1` again (restarted from main), one branch,
one PR (CLAUDE.md); the owner opens the PR.

Before that, PR #149 (phase 3's bulk) merged on 2026-11-24.

## What landed on `claude/phase-2-1` since #150 (2026-11-25)

- **Captures owed by task 14.5**, taken (lavapipe, no frame times):
  - `docs/screenshots/townsfolk/road-slope.jpg`: a two-wide gravel road
    climbing ahead, from its first stake.
  - `docs/screenshots/townsfolk/road-bridge.jpg`: the paved plank bridge,
    on foot from its first cell. Two findings in it, recorded in design 42o
    ("Found in the captures") and below for the owner.
  - `docs/screenshots/building-planner/scaffold-ladder-close.jpg`: the
    scaffold's ladder bay.
  - Why earlier takes faced past the road: the instrument's `--yaw` was a
    hand-worked bearing with north and south swapped. It is now
    `place::yaw_of`, the launch's tested inverse (also the roads' log
    line it was copied from).
- **Design 42o, task 14.13** (written up first):
  - the road planner's ghost of a bridge is the bridge piece paving
    builds, cut by the same `roads::piece::cut` (deck, posts, rails);
  - the ghost stands its lamp posts not yet built, one lofted post and
    lantern in the ghost tint (`models::ghost_lamp`);
  - each taken escort job's trader is marked on the map and the compass
    bar, "Trader" (`jobs.escort_marker`), where the stored record puts them;
  - tests `roads::tests::the_ghost_carries_a_bridge_s_deck_posts_and_rails_and_its_lamp_posts`
    and the escort test's marker checks;
  - capture `road-planner-ghost-lamps-1100.jpg` and its `-close` crop
    (`--road-plan ghost-lamps`; the aim is held by the capture-only
    `AimPin`). The ghost over a bridge is not captured: at the
    instrument's bridge spot the lifted eye (42n) looks down on forest
    canopy over the road. `ghost-bridge:A:B:W` works (the bill counts the
    bridge's planks); it wants a bridge spot in the open.
- **The sweep's owed tests:**
  - player-building 4.1: `build_mode_gives_the_hotbar_and_its_selection_back`;
  - player-building 4.4: `a_cottage_blueprint_filled_is_walked_into_through_its_doorway`;
  - town-missions-tree 7.1 ticked on its existing test and the bot's log.
- **A stale test fixed:** `a_campfire_and_a_dish_survive_the_round_trip`
  asked for the chest at code 30 (the grinder's); the chest is 42 since
  #149's merge. It failed on main.

## For the owner to decide

- **No new world gets a wide world road yet.** Walled towns stand at least
  2,500 m apart (`spacing_m`, `sites.ron`), and version 3's links reach only
  2,000 m (`link_m`), with the walled towns' long link dropped (design 43).
  So a fresh world plans 8 roads, all dirt. Making version 4 show up means
  changing design 43's link rule: a look change for the survey with
  screenshots (43a "As built", task 8.6). `world-wide-road-slope.jpg` waits
  on it.
- **The forge flame's flicker** from `5c99a764`: port it or not.
- **Bridges, from `road-bridge.jpg`** (design 42o, "Found in the
  captures"; for the survey with that screenshot):
  - where a river runs bank-full the deck stands at the water's surface and
    the river draws its ripples over the planks. Recommended: the deck a
    quarter metre over such water, its ends ramped, as a new road version
    for the world's bridges. A render-side fix would be the rendering
    session's;
  - the rails are the approved mockup's solid timber panels, which box the
    deck in; a rail of posts and a top bar is the alternative.

## Still owed

- **`LATE_PIPELINE`** (about 31 a launch): dropped from this session on the
  owner's word, "DO NOT DO PERF, THE OTHER SESSION WILL DO PERF". The perf
  session owns it.
- **Not done in the sweep, and why:**
  - townsfolk 4.1, 4.2, 9.2; town-missions-tree 5.1; building-planner 4.3;
    player-building 1.2a, 5.1a: videos and gameplay demos, batched for the
    owner's desktop.
  - townsfolk 4.2, 13.2; player-building 5.2; building-planner 1.6;
    land-transport 1.5, 1.6: the owner's look, acceptance or publishing.
  - building-planner 4.2: frame cost (perf; the other session).
  - townsfolk 7.4: its check is a bot playing one of each job to pay;
    that is a bot run of the game, and lamps and repairs are still not
    posted on boards.
  - townsfolk 8.6: the world wide road link rule (owner's decision).
  - townsfolk 9.1: captures of trade, a board, each special job and a road
    between two towns; the shipped spawn has no road within a capture's
    walk. Not attempted.
  - townsfolk 12.6: enchanted tools (deferred).
  - land-transport 2.x to 4.x: phase 5, waiting on the owner's look at M38.

## For the owner's live desktop session

Videos (batch), real-time checks, and frame cost on the GPU (`perf_suite.py`).
The cloud session measured no frame cost; its captures are lavapipe.

## Deferred by the owner

Multiplayer (3b), crew missions, stealing, sleeping, enchanted tools.

## How to pick it up

- `CARGO_TARGET_DIR=/home/user/Pale-Blue-Dot/target`, and build `--profile fast` only.
- Disk is tight (about 6 GB free). A full worktree costs about 1.9 GB of screenshots, so use a sparse one: `git worktree add --no-checkout`, then `git sparse-checkout set --no-cone '/*' '!/docs/screenshots/' '!/docs/benchmarks/' '!/docs/videos/'`, which comes to about 240 MB.
- Never `git stash`: an unrelated old stash exists.
- GitHub: `gh api` REST only (porcelain fails on GraphQL).
- **After editing `assets/locale/locale_en.yaml`, touch
  `crates/pbd-core/src/locale.rs` before a `fast` build.** The built-in copy
  is an `include_str!` in pbd-core, and on 2026-11-25 the `fast` build did
  not rebuild pbd-core for the binary after a YAML edit: the game logged
  `LOCALE_MISSING` for a key that was in the file (the tests, built fresh,
  passed). Check with `strings target/fast/pbd-app | grep '<key>: '`.
  Once more the same day a capture ran a binary that logged the old
  `--yaw` formula after the source had changed; a fresh build gave the
  right one. Before trusting a capture after an edit, check the binary's
  log says what the new code says.
- Capture worlds persist under `saves/` and their names are cut short: a
  capture script's `--world` name should lead with something unique (the
  road planner's now leads with its PID).
- `--view column` stands no walker; the paving instrument
  (`--paving`/`--road-plan`) needs one, and that launch stops on a missing
  resource. Shoot roads on foot.
- Checks before a push:
  - `cargo fmt --all -- --check`;
  - `cargo clippy --profile fast --workspace --all-targets -- -D warnings`;
  - `cargo test --profile fast -p pbd-core --lib`, `-p pbd-app --lib` and `-p pbd-app --test locale`;
  - `openspec validate --all`.
- Known flake source: tests that install process-global terrain state must take `pbd_core::settlement::ground::hold()`, or use the `install_held` variants (see `roads/paving.rs`).
