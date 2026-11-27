# Tasks

Phase 3b. Started 2026-11-21 at the owner's ask ("lets get to work on
that"), ahead of phase 3's townsfolk; the order of work is design 16.
Groups 1 and 2 are the dependencies, and each is its own change.

## 1. Prerequisites (other changes)

- [x] 1.1 (Done 2026-11-21; `edit-pipeline` tasks 5.) `edit-pipeline` section 5: ACCEPT and APPLY split, a sequence on the log. Verify: that change's test that replaying a sequence from cold gives the same columns.
- [ ] 1.2 `world-persistence` 4.1, 5.1 and 5.2: the world clock, regions with snapshots and their last sequence, the segmented journal. Verify: that change's tests.

## 2. The rules run without a frame

- [x] 2.1 (Written 2026-11-21, design 17, from a sweep of `WorldSave`'s callers.) List every system in `pbd-app` that writes stored state (edits, records, inventory, drops, crops, animals, missions) and the core rule each calls, in this change's design as a table. Verify: the table names every writer `grep` finds.
- [ ] 2.2 (Pack moves and pickups 2026-11-22, design 19: `pbd_core::accept::pack` and `pick`; `Server::pack` and `Server::pick` with the store's live drops; the pack screen and the magnet ask the server. Tests: `a_pack_move_is_applied_or_refused`, `a_pickup_gives_what_fits`, `a_pack_move_goes_through_the_server`, `two_players_reaching_one_drop_are_given_it_once`, `pack_moves_and_pickups_through_the_server_write_the_same_lines`. First rule 2026-11-21: the terrain edit, `pbd_core::accept::edit`, called by `apply_edit`; core tests `an_edit_is_refused_out_of_the_world_unchanged_or_on_a_second_lamp`, `an_accepted_edit_settles_the_hands`. The rest of design 17's writers follow.) Move each writer's ACCEPT half into `pbd-core` as a function of (world state, player, request) returning a change or a refusal, one rule per commit. Verify: each rule has a core test that runs it with no Bevy app, and single player plays the same (the phase-1 starter missions finish under `--capture`).

## 3. Protocol and digest in the core

- [x] 3.1 (Built 2026-11-21: `pbd_core::net::digest`, with `encode`, `fnv1a64`, `Regions` keeping each region's digest as edits are noted, and `recompute`; the edits section only, design 16. `a_fixed_regions_digest_is_pinned`, `the_kept_digest_equals_a_full_recompute`, `a_change_moves_only_its_own_regions_digest`. The save's material codes moved into `pbd_core::terrain` for it.) The region's canonical encoding and its FNV-1a 64 digest (design 2), kept incrementally. Verify: a pinned digest for a fixed region; the incremental digest equals a full recompute after a random sequence of changes.
- [x] 3.2 (Built 2026-11-21: `pbd_core::net::protocol`, `ToServer` and `ToClient` with hand-written little-endian encoding, `Decode::Unknown` for a newer tag, finite and length guards, and `agree` for the join (design 9, 15.9). `every_message_round_trips_and_a_cut_one_is_refused`, `an_unknown_newer_message_is_refused_cleanly`, `a_value_no_message_has_is_refused`, `a_join_agrees_on_a_protocol_range_and_the_exact_world`.) The message types (join, request, accepted, refused, region change, snapshot, digest, presence, transaction, chat) and their versioned byte encoding. Verify: a round-trip test per message, and a test that an unknown newer message is refused cleanly.

## 4. The server, in process

- [x] 4.1 (Built 2026-11-21: `crates/pbd-server`, on `pbd-core` alone. `Server::join` checks versions and numbers players by install key; `handle` decides a dig or a place with `pbd_core::accept`, from a column it generates from the seed, worms and edits; numbers it with the `Ledger`; journals it through the `Journal` trait before answering; then sends `Accepted` and the `Change` to every player. A refusal re-sends the region. `tick` runs the clock only while someone is on, sent every 5 s. `MemoryJournal` for tests; the save's writer implements `Journal` with 4.2. Tests: `requests_are_journaled_in_order_and_sent_to_everyone`, `two_players_on_one_cell_are_ordered_by_the_server`, `a_place_spends_and_a_refusal_changes_nothing`, `a_join_is_checked_and_numbered`, `the_clock_runs_only_while_someone_plays`.) `pbd-server` crate: the world, the 20 Hz tick, per-region sequences, the journal through `SaveWriter`. Verify: a core-only test that applies requests and journals them with ascending sequences.
- [ ] 4.2 (Edits done 2026-11-22, design 18: `pbd-server` takes a `Store`; `WorldSave` is single player's, through `SaveStore`, and `WorldSave::decide_edit` puts every dig and place through `Server::edit`. `saves::tests::the_in_process_server_writes_the_save_accept_always_wrote`: the same digs through the server and straight through `accept` write byte-identical logs; a capture's digs and places all taken with no `SERVER_DISAGREES` (`docs/screenshots/multiplayer-local-server/`). The player is always 0, so an old save is player 0's. Still to do: the in-memory channel and the other writers of design 17, each as its ACCEPT reaches the core.) Single player starts the server in process and talks to it over an in-memory channel. Verify: the same inputs produce byte-identical saves before and after the switch; the starter missions finish under `--capture`; an old save opens with its player as 0.
- [ ] 4.4 (Design 22, 2026-11-27.) A joined player's own slots: `ToClient::Slots`, sent after `Welcome`, after each accepted request that changed them and after each refusal; the server keeps a departed player's slots for a rejoin; the joined game's `Hotbar` follows them. Verify: the checks listed in design 22.
- [ ] 4.3 Prediction and reconcile on the client (design 4), with Luanti's fix-up (design 15.3): a refusal re-sends the touched cells and the whole inventory, and the region if a snapshot was in flight. Verify: an app test where the server refuses a predicted dig and the column and inventory return to the server's; a second where the region's snapshot was in flight when the dig was predicted.

## 5. Mockups for the owner

- [ ] 5.1 Mockup: host and join screens (pause menu Host, join by address or LAN, the refusal messages). Published, on the tracker. Verify: the owner approves it on the tracker.
- [ ] 5.2 Mockup: another player in the world, by day and lit at night, near with their name and far without. Verify: the owner approves it on the tracker.

## 6. The network

- [x] 6.1 (Built 2026-11-22, design 20: `crates/pbd-net`, `Host` and `Client` over `quinn` on their own thread and runtime, one length-prefixed stream each way, a self-signed certificate per run. `Welcome` now carries the world's seed, generator and topology, and a `Hello` naming world 0 joins any world. Tests: `a_client_joins_a_host_on_loopback`, `a_mismatched_generator_is_refused`, `bytes_that_are_not_a_message_close_the_connection`, `a_host_that_is_not_there_times_out`. Datagrams wait for presence, task 8.1.) `pbd-net` on QUIC: connect, the join check (design 9, with a protocol range per design 15.9), streams and datagrams. Verify: a test connecting two processes on loopback; a mismatched generator version is refused with the plain message.
- [x] 6.2 (Built 2026-11-22, design 20: `--host <port>` and `--join <address:port>` from the command line, no screens until mockup 5.1. Proved 2026-11-23 on two machines at the owner's instruction, the Mac hosting and the Windows laptop joining over Wi-Fi: each saw the other's digs, no refusal or disagreement; `docs/screenshots/multiplayer-two-machines/`. `--server` headless and Host from the pause menu wait for mockup 5.1; the empty joined hotbar and the bare-hand aim found in the run are noted there.) `--server` headless launch and Host from the pause menu without a reload. Verify: a second game process joins a hosted world on loopback in a cloud run, with `--capture` shots from both.
- [x] 6.5 (Built 2026-11-23, design 21: `crates/pbd-headless`; three bots and the live game on the Mac passed, `docs/screenshots/multiplayer-headless/`; three join bugs found and fixed. Tests: `three_bots_build_a_hut_and_agree_with_the_server`, `every_bot_reads_the_same_site`, `a_player_who_rejoins_is_served_on_the_new_connection`.) `pbd-headless` (design 21): a dedicated server in memory (`serve`), a protocol bot that builds a hut with others and checks order, answers and digests (`bot`), and both in one process (`soak`). Verify: a cargo test soak of three bots passes; on the Mac the live game joins the headless server and sees the bots' hut, every bot prints `BOT_OK`, shots and logs in `docs/screenshots/multiplayer-headless/`.
- [ ] 6.3 Region subscription and hysteresis (design 3): the client sends its tier reach and the regions it dropped; snapshots go nearest first with a per-client in-flight cap and acks; separate simulate, subscribe and presence radii in validated config (design 15.4-15.6). Verify: a test that no fine cell is built in a region before its snapshot arrives; a walk into a region another client edited shows the edit; a test that snapshots arrive in distance order.
- [ ] 6.4 Near changes sent, far changes mark the region stale (design 15.2). Verify: a test that a subscriber beyond the near radius receives no single change, and holds the region's new state after its re-send.

## 7. Checksums and resync

- [ ] 7.1 The periodic digest round-robin and the resync request (design 2). Verify: a test that corrupts one cell in a client's copy and sees the region replaced and the digests match within one check cycle.

## 8. Sharing the world

- [ ] 8.1 Other players' presence: snapshots at 20 Hz, interpolation, drawn in the client's own origin. Verify: two-client `--capture` shots; a test that a rebase on one client leaves the other's positions unchanged.
- [ ] 8.2 (Drops made by a dig, and pickups, through the server: design 23, 2026-11-27.) Drops, containers, crops, lamps and stations replicated as region records. Verify: an app test where one client drops an item and the other picks it up, and the item exists exactly once.
- [ ] 8.3 Animals run on the server for subscribed regions; fishing's catch is a server request. Verify: a test that taming an animal on one client shows it tamed on the other.
- [ ] 8.4 Water flow runs on the server; liquid changes ride the region stream. Verify: a dug channel floods the same on both clients, and their digests match after it settles.
- [ ] 8.5 Far transactions: give, trade, shared container, chat, map marker (design 12). Verify: a test that a gift whose receiver's pack is full leaves both inventories unchanged.
- [ ] 8.6 The world clock runs only while a player is connected (design 11), sent every 5 s and run forward by the client (design 15.8). Verify: a test that a server with nobody on it does not advance its clock.
- [ ] 8.7 Cheat checks (design 15.7): movement speed with a tolerance, interaction distance, dig time against the tool; each a config flag, logged with player and reason, never run in process. Verify: a test per check that an out-of-bounds request is refused and logged, and that single player never runs them.

## 9. The owner's gate

- [ ] 9.1 Screenshots in `docs/screenshots/` of two clients side by side: digging the same cell, a dropped item crossing, a tamed animal, flooding water. Verify: the owner has seen them.
- [ ] 9.2 The gameplay demo (CLAUDE.md "Demos"): two bots with a stated goal, for example "build a farm together and cook a meal from it", recorded with `obs-record` by the owner. Verify: the owner watches and approves.
- [ ] 9.3 The owner runs `perf_suite.py` on the desktop with a second client connected; a report in `docs/benchmarks/`. Verify: the report is checked in and linked from the tracker.

## Workflow follow-up

- Sync `world/multiplayer` into `openspec/specs/` requirement by requirement, each in the commit that makes it true with its test.
- Follow-ups named in the design: the coarse-LOD region summary, a relay for NAT, seats on moving craft, rollback by author (design 15.11), content by hash for mods (design 15.10).
