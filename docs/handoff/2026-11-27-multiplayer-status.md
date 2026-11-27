# Multiplayer: where it stands (handoff, 2026-11-27)

Written for whichever session picks up multiplayer (phase 3b) next. The
plan is `openspec/changes/multiplayer` (proposal, design, tasks); the
launcher is `openspec/changes/game-launcher`. This file says what is real
today, what is not, and the order to take the rest in. Branch and PR:
`claude/phase-2-1`, https://github.com/RubenTipparach/Pale-Blue-Dot/pull/152
(one branch, one PR: CLAUDE.md).

## In one line

The pipe works between two real machines, but only terrain edits, pack
moves and pickups (and survival's vitals, server side) go through the
server. Nobody sees the other player, and animals, crops, water, drops,
containers and stations are not shared. `multiplayer` is 8 of 27 tasks
done; `game-launcher` is 0 of 16.

## What is built and tested

| Task | What | Proof |
| --- | --- | --- |
| 1.1 | `edit-pipeline` 5: ACCEPT and APPLY split, a sequence on the log | that change's replay-from-cold test |
| 2.1 | Design 17: the table of every system that writes stored state and its core rule | the table names every writer `grep` finds |
| 2.2 (part) | Core ACCEPT rules: the terrain edit (`pbd_core::accept::edit`), pack moves and pickups (`accept::pack`, `accept::pick`; `Server::pack`, `Server::pick`) | `a_pack_move_goes_through_the_server`, `two_players_reaching_one_drop_are_given_it_once`, and the rest named in tasks 2.2 |
| 3.1 | Region encoding and FNV-1a 64 digest, kept as edits land (`pbd_core::net::digest`) | `a_fixed_regions_digest_is_pinned`, `the_kept_digest_equals_a_full_recompute` |
| 3.2 | The protocol (`pbd_core::net::protocol`, `ToServer` / `ToClient`), versioned little-endian, unknown newer tags refused | a round trip per message, `an_unknown_newer_message_is_refused_cleanly` |
| 4.1 | `crates/pbd-server` on `pbd-core` alone: join check, numbering, journal before answer, broadcast, the clock only while someone plays | `requests_are_journaled_in_order_and_sent_to_everyone` and the rest in tasks 4.1 |
| 4.2 (part) | Single player's digs and places go through the in-process server (`WorldSave::decide_edit`) | `the_in_process_server_writes_the_save_accept_always_wrote` (byte-identical logs); `docs/screenshots/multiplayer-local-server/` |
| 6.1 | `crates/pbd-net`: QUIC host and client (`quinn`), self-signed cert per run, one stream each way | `a_client_joins_a_host_on_loopback`, `a_mismatched_generator_is_refused` |
| 6.2 | `--host <port>` and `--join <address:port>` on the command line | the owner's Mac hosting and Windows laptop joining over Wi-Fi, 2026-11-23: each saw the other's digs, no refusals or disagreements; `docs/screenshots/multiplayer-two-machines/` |
| 6.5 | `crates/pbd-headless`: a dedicated server in memory, protocol bots, a soak | `three_bots_build_a_hut_and_agree_with_the_server`; the live Mac game joined it and saw the bots' hut (`docs/screenshots/multiplayer-headless/`) |

Survival (phase 4) already keeps its vitals on the server:
`crates/pbd-server/src/survival.rs` (`Server::survive`, `survival_act`, a
`vitals` record per player, journaled at once on an event and every 5 s
of play). Test `vitals_are_journaled_on_a_hit_and_every_few_seconds_and_read_back`.
Single player runs it through `WorldSave`. It has not been run with two
clients.

## Known faults from the two-machine run (2026-11-23)

- The joining player's hotbar is empty.
- The joining player's aim falls back to bare hands.

Both are noted in `docs/screenshots/multiplayer-two-machines/` and not
fixed yet. They come from the pack and held item not being sent on join.

## Not built, in the order to take it

1. **The rest of design 17's writers into the core (2.2), then 4.2's
   in-memory channel.** Crops, animals, missions, containers, stations,
   drops, records: each ACCEPT half becomes a `pbd-core` function of
   (world, player, request), one rule a commit, with a core test and the
   starter missions still finishing under `--capture`. Until a rule moves,
   it is not shared.
2. **1.2, `world-persistence` 4.1, 5.1, 5.2**: the world clock, regions
   with snapshots and their last sequence, the segmented journal. Region
   streaming needs it.
3. **4.3 prediction and reconcile**: a refused dig puts the cells and the
   whole pack back to the server's, and the region if a snapshot was in
   flight.
4. **6.3 and 6.4, region subscription**: tier reach, snapshots nearest
   first with an in-flight cap and acks, separate simulate, subscribe and
   presence radii in validated config; near changes sent one by one, far
   ones mark the region stale.
5. **7.1 checksum round robin and resync.**
6. **8.1 presence**: other players at 20 Hz, interpolated, drawn in each
   client's own origin (a rebase on one leaves the other's positions
   alone). This is the first thing the owner will notice is missing.
7. **8.2 to 8.7**: drops, containers, crops, lamps and stations as region
   records; animals and the fishing catch on the server; water flow on the
   server; gifts, trade, shared containers, chat and markers; the clock
   sent every 5 s; cheat checks (never run in process).
8. **5.1 and 5.2 mockups**, which gate the screens: host and join (pause
   menu Host, join by address or LAN, the refusal messages) and another
   player in the world by day and lit at night. `--server` headless and
   Host from the pause menu wait on 5.1. They can be made at any point and
   are the quickest thing to put in front of the owner.
9. **9.1 to 9.3, the owner's gate**: side-by-side shots, a two-bot
   gameplay demo recorded with `obs-record` by the owner, and a
   `perf_suite.py` run with a second client connected (the owner's, on
   the desktop; this session does no perf).

## Phase 4 tasks that wait on this

- `survival-mode` 4.7: two players and a fox show the same hit; a swing at
  another player does nothing (the second half has a single-player test,
  `combat::tests::a_swing_at_another_player_does_nothing`).
- `wildfire` 4.6: two clients, one fire, the same scar.

Both need 8.1 (presence) and the animals and fire on the server first.

## How multiplayer is tested

On the owner's machines, not two processes in a cloud container (CLAUDE.md,
"Multiplayer is tested on the owner's machines"): the Mac
("[Mac] pale-blue-dot") hosts, the laptop ("[Laptop] mining mike
continue") joins. The procedure is `multiplayer` design 20; the last
run's step-by-step is `docs/handoff/2026-11-23-multiplayer-two-machines.md`
(its branch name is old: use `claude/phase-2-1`). The laptop does the
join, the checks and a screenshot, then closes the game; anything long
runs on the Mac or the desktop. In a cloud session, `pbd-headless`'s bot
soak and the loopback tests are what can be run.

## Later, named in the design

The coarse-LOD region summary, a relay for NAT, seats on moving craft,
rollback by author (design 15.11), content by hash for mods (design
15.10), and the launcher that keeps the game current from itch.io
(`game-launcher`).
