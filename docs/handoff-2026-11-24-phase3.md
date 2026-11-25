# Handoff: phase 3 in the game (2026-11-24)

**PR #149 was MERGED to main at 23:04.** New work goes in a NEW draft PR
from `claude/phase-2-1` (merge origin/main in first).

**Forge redo: done, in PR #150** (2026-11-25, `townsfolk` 26b, task 3.2b):
tongues of flame (`forgefire`) over a coal bed (`embers`), and one lofted
anvil with its face, horn and heel. Two redos landed at once; 26b's was
kept on merge (12238945), and the other (animated flame on a stump) is at
5c99a764, its flicker a follow-up to port. Clippy 1.97's new lints fixed
across the workspace. Full suites green before the merge (core 863, app
709, locale 3, openspec 234); settlement, towns, locale and openspec green
after it.

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
- Full suites passed at `9bd719e1`: pbd-app 708, pbd-core 861, locale 3,
  openspec 234.

## In flight

One agent: redoing the forge fire and anvil (below). Everything else landed (head dc05edc9). Town/building landed: the
forge (layout 3, old towns pinned), idle capture, scaffold ladder bay,
double-click to walk and the End cutaway, ghost_night 0.12.

## Known faults to fix next

- The forge's fire is five pale cones: obvious primitives (CLAUDE.md "No
  obvious primitives"). Redo as one shaped, textured flame mesh, and look
  at it close up. The anvil reads as a dark blob.
- Ladder bay needs a close capture.

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
