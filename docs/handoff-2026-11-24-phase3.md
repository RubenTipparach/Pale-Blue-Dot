# Handoff: phase 3 in the game (2026-11-24)

Branch `claude/phase-2-1`, draft PR
https://github.com/RubenTipparach/Pale-Blue-Dot/pull/149. One branch, one
PR (CLAUDE.md). The PR body lists what is built and what is open.

## Owner's standing asks in this stretch

- "cool implement all the phase 3 stuff!", then "leave multiplayer out for
  now, we'll playtest it after we get all the other stuff done".
- Survey P1 to P4, N1 and N2 answered and moved to "Already decided"
  (https://claude.ai/artifact/HvNgkEFwnJkadM3Y3ELqi9).
- Talk framing (P1), then "Kind of center the character between the left
  most edge of panel and left edge of the screen. Also if you can add some
  lights that fade in light the character a bit", then "same presentation
  style for dialog boxes of any kind of interaction with npcs".

## Built and pushed

- Building planner (`building-planner`): table, mallet, planner screen,
  sites, scaffolding, change list, cactus check. Captures in
  `docs/screenshots/building-planner/`.
- Road planner (`townsfolk` 13.4, 14, design 42k/42l). Captures in
  `docs/screenshots/townsfolk/road-planner-*`.
- Market, barter, derived values (`townsfolk` 11, 12), furniture 4.3.
- Roads (landed a53944d4): wide roads on one plane, player bridges, lamp
  posts built, insert a stake, road missions (rockfall, escort), P2's 55%
  ghost and lifted planner view. Open: the world's own wide roads (new
  roads version), captures of a slope and a bridge, ghost lamps/decks,
  14.5's compass tick and bill test, trader map marker.
- NPC dialogs (landed 2c57b424): panel on the right, the townsperson
  centred in the open space left of it, a warm talk light faded in through
  the carried-light list; one presentation for Talk, Trade and Board
  (`desktop/talk.rs`, `assets/config/talk.ron`).
- Full suites passed at `4e55e54c`: pbd-app 700, pbd-core 852, locale 3,
  openspec 234.

## In flight when this was written (background worktree agents)

Each lands on `claude/phase-2-1` when done; check `git log origin/claude/phase-2-1`
and `git worktree list` (`.claude/worktrees/agent-*`). A worktree whose
HEAD is already on origin and is clean can be removed.

2. Town/building: forge (townsfolk 3.2, design 26, new town version if
   layout moves), townsfolk 2.2 idle captures, scaffold ladder bay,
   player-building 4.5 (double-click walk, cutaway); plus survey P3: raise
   `ghost_night` (was 0.05) so the site ghost glows more at night.

If an agent died mid-way, its uncommitted work is in its worktree; read
its diff before deciding to finish or drop it.

## After they land

- Pull, look at their captures, send them to the owner.
- Run the full suites (`cargo test --profile fast -p pbd-app --lib`,
  `-p pbd-core --lib`, `-p pbd-app --test locale`, `openspec validate
  --all`); use `CARGO_TARGET_DIR=/home/user/Pale-Blue-Dot/target`. Disk is
  tight (~6 GB free); if a build fails against a stale pbd_core, touch a
  pbd-core source.
- Update the PR body; end the reply with the artifact links line.
- Known flake source: tests that install process-global terrain state must
  take `pbd_core::settlement::ground::hold()` or use the `install_held`
  variants (see `roads/paving.rs`).

## Still open regardless

Videos (batch), frame cost on the owner's GPU, multiplayer and crew
missions (deferred by the owner), stealing, sleeping, enchanted tools.
