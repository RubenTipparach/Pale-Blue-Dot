# Handoff: phase 3 in the game (2026-11-25)

**PR #150 merged to main** (2026-11-25, on the owner's "MERGE PR TO MAIN").
New work starts from main again: restart `claude/phase-2-1` from
origin/main, or ask the owner for the next branch. One branch, one PR
(CLAUDE.md).

Before that, PR #149 (phase 3's bulk) merged on 2026-11-24.

## What PR #150 landed

- **Forge redo** (`townsfolk` 26b, task 3.2b).
  - Fire: tongues of flame (`forgefire`) over a coal bed (`embers`).
  - Anvil: one lofted anvil with its face, horn and heel.
  - Every fire in a town goes through `towns::is_fire`.
  - The other session's redo (an animated flame sheet, the anvil on a stump) is in the history at `5c99a764`. Porting its flicker is a follow-up.
- **Clippy 1.97** lints fixed across the workspace, including main's newer bot code in the last merge. No behaviour changes.
- **Roads version 4, the world's own wide roads** (design 43a, task 8.6).
  - Pinned first: `roads::tests::version_3_cells_are_pinned`, 916 cells.
  - Gravel between walled towns is 2 wide, and 3 wide for the last `gate_cells` (6) at each walled end.
  - The lanes use the player planner's own `Lanes` code.
  - Roads with lanes are a schema 3 `road` record. Worlds from versions 1 to 3 keep their roads and are never replanned.
- **Paving's compass tick and bill test** (task 14.5).
  - The nearest unpaved marked cell within 60 m is a red pin on the compass bar while the shovel is in hand. Faced, it reads "Next cell" (`roads.pave.next`).
  - Test: `the_bill_falls_a_gravel_a_cell_paved_and_a_short_good_is_red`.
  - Capture instrument: `--road-plan slope|bridge`. It finds, stakes and paves a two-wide climb or a plank bridge, then logs where to stand.
- **Main merged in**: the bot playtest of the starter and town trees.

## For the owner to decide

- **No new world gets a wide world road yet.** Walled towns stand at least 2,500 m apart (`spacing_m`, `sites.ron`), and version 3's links reach only 2,000 m (`link_m`), with the walled towns' long link dropped (design 43). So no gravel road is ever laid: a fresh world plans 8 roads, all dirt. Making version 4 show up means changing design 43's link rule. That's a look change, so it goes to the survey with screenshots; recorded in 43a "As built" and task 8.6.
- **The forge flame's flicker** from `5c99a764`: port it or not.

## Still owed (cloud session can do these)

- **Captures, task 14.5:**
  - `docs/screenshots/townsfolk/road-slope.jpg` and `road-bridge.jpg`. Use `--road-plan slope|bridge` and stand where it logs. Earlier takes faced past the road, or stood in a pit or under water; frame by the logged stand point and yaw.
  - `docs/screenshots/building-planner/scaffold-ladder-close.jpg`: `ONLY=scaffold-ladder-close LADDER_DIST=12 LADDER_TILT=28 LADDER_YAW=±45 tools/building_planner_shots.sh`. At 9 m the camera sat inside the ghost walls.
  - Look at each one close up (no obvious primitives, textured, lit as the terrain).
- **`world-wide-road-slope.jpg`** waits on the link decision above.
- **`LATE_PIPELINE`:** about 31 lines on every launch, plain launches too. A loading-screen bug (CLAUDE.md): add those pipelines to the prewarm.
- Ghost lamps and decks in the road planner; the trader's map marker.

## For the owner's live desktop session

Videos (batch), real-time checks, and frame cost on the GPU (`perf_suite.py`). The cloud session measured no frame cost; its captures are lavapipe.

## Deferred by the owner

Multiplayer (3b), crew missions, stealing, sleeping, enchanted tools.

## How to pick it up

- `CARGO_TARGET_DIR=/home/user/Pale-Blue-Dot/target`, and build `--profile fast` only.
- Disk is tight (about 4 GB free). A full worktree costs about 1.9 GB of screenshots, so use a sparse one: `git worktree add --no-checkout`, then `git sparse-checkout set --no-cone '/*' '!/docs/screenshots/' '!/docs/benchmarks/' '!/docs/videos/'`, which comes to about 240 MB.
- Never `git stash`: an unrelated old stash exists.
- GitHub: `gh api` REST only (porcelain fails on GraphQL).
- Checks before a push:
  - `cargo fmt --all -- --check`;
  - `cargo clippy --profile fast --workspace --all-targets -- -D warnings`;
  - `cargo test --profile fast -p pbd-core --lib`, `-p pbd-app --lib` and `-p pbd-app --test locale`;
  - `openspec validate --all`.
- Known flake source: tests that install process-global terrain state must take `pbd_core::settlement::ground::hold()`, or use the `install_held` variants (see `roads/paving.rs`).
