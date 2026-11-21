# Tasks

Phase 3b. Started 2026-11-21 at the owner's ask ("lets get to work on
that"), ahead of phase 3's townsfolk; the order of work is design 16.
Groups 1 and 2 are the dependencies, and each is its own change.

## 1. Prerequisites (other changes)

- [ ] 1.1 `edit-pipeline` section 5: ACCEPT and APPLY split, a sequence on the log. Verify: that change's test that replaying a sequence from cold gives the same columns.
- [ ] 1.2 `world-persistence` 4.1, 5.1 and 5.2: the world clock, regions with snapshots and their last sequence, the segmented journal. Verify: that change's tests.

## 2. The rules run without a frame

- [ ] 2.1 List every system in `pbd-app` that writes stored state (edits, records, inventory, drops, crops, animals, missions) and the core rule each calls, in this change's design as a table. Verify: the table names every writer `grep` finds.
- [ ] 2.2 Move each writer's ACCEPT half into `pbd-core` as a function of (world state, player, request) returning a change or a refusal, one rule per commit. Verify: each rule has a core test that runs it with no Bevy app, and single player plays the same (the phase-1 starter missions finish under `--capture`).

## 3. Protocol and digest in the core

- [ ] 3.1 The region's canonical encoding and its FNV-1a 64 digest (design 2), kept incrementally. Verify: a pinned digest for a fixed region; the incremental digest equals a full recompute after a random sequence of changes.
- [ ] 3.2 The message types (join, request, accepted, refused, region change, snapshot, digest, presence, transaction, chat) and their versioned byte encoding. Verify: a round-trip test per message, and a test that an unknown newer message is refused cleanly.

## 4. The server, in process

- [ ] 4.1 `pbd-server` crate: the world, the 20 Hz tick, per-region sequences, the journal through `SaveWriter`. Verify: a core-only test that applies requests and journals them with ascending sequences.
- [ ] 4.2 Single player starts the server in process and talks to it over an in-memory channel. Verify: the same inputs produce byte-identical saves before and after the switch; the starter missions finish under `--capture`; an old save opens with its player as 0.
- [ ] 4.3 Prediction and reconcile on the client (design 4), with Luanti's fix-up (design 15.3): a refusal re-sends the touched cells and the whole inventory, and the region if a snapshot was in flight. Verify: an app test where the server refuses a predicted dig and the column and inventory return to the server's; a second where the region's snapshot was in flight when the dig was predicted.

## 5. Mockups for the owner

- [ ] 5.1 Mockup: host and join screens (pause menu Host, join by address or LAN, the refusal messages). Published, on the tracker. Verify: the owner approves it on the tracker.
- [ ] 5.2 Mockup: another player in the world, by day and lit at night, near with their name and far without. Verify: the owner approves it on the tracker.

## 6. The network

- [ ] 6.1 `pbd-net` on QUIC: connect, the join check (design 9, with a protocol range per design 15.9), streams and datagrams. Verify: a test connecting two processes on loopback; a mismatched generator version is refused with the plain message.
- [ ] 6.2 `--server` headless launch and Host from the pause menu without a reload. Verify: a second game process joins a hosted world on loopback in a cloud run, with `--capture` shots from both.
- [ ] 6.3 Region subscription and hysteresis (design 3): the client sends its tier reach and the regions it dropped; snapshots go nearest first with a per-client in-flight cap and acks; separate simulate, subscribe and presence radii in validated config (design 15.4-15.6). Verify: a test that no fine cell is built in a region before its snapshot arrives; a walk into a region another client edited shows the edit; a test that snapshots arrive in distance order.
- [ ] 6.4 Near changes sent, far changes mark the region stale (design 15.2). Verify: a test that a subscriber beyond the near radius receives no single change, and holds the region's new state after its re-send.

## 7. Checksums and resync

- [ ] 7.1 The periodic digest round-robin and the resync request (design 2). Verify: a test that corrupts one cell in a client's copy and sees the region replaced and the digests match within one check cycle.

## 8. Sharing the world

- [ ] 8.1 Other players' presence: snapshots at 20 Hz, interpolation, drawn in the client's own origin. Verify: two-client `--capture` shots; a test that a rebase on one client leaves the other's positions unchanged.
- [ ] 8.2 Drops, containers, crops, lamps and stations replicated as region records. Verify: an app test where one client drops an item and the other picks it up, and the item exists exactly once.
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
