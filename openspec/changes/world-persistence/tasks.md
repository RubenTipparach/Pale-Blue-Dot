# Tasks

This comes with step 2a, before any change writes new kinds of state
(`bigger-biomes`' version, `city-sites`' records). The exact cell keys that
were group 1 here are their own change, `exact-cell-keys`, which ships first
(the owner, 2026-09-27: "fix cell number yes").

## 1. Exact cell keys

Moved to `openspec/changes/exact-cell-keys`, which lands before this change.

## 2. Identity

- [x] 2.1 (Built 2026-09-28: `pbd_core::terrain::TOPOLOGY_VERSION`, `saves::format::Identity`, `identity.ron` written first with `writer::replace_durably`, an old slot's written on open and upgraded to exact keys after its migration, and the saves screen and the launch refusing an unknown version by name. The identity requirement is synced into `openspec/specs/world/persistence`.) `TOPOLOGY_VERSION`, and `identity.ron` written with a barrier at creation; an old slot gains one as generator 4, topology 1, and key version 1 once `exact-cell-keys` has migrated it. Verify: format tests for both, and a test that a build refuses an unknown version with the version named.
- [ ] 2.2 The world's config is chosen from the identity, so `bigger-biomes` can add version 5 without touching the save. Verify: an app test that an old slot generates version 4's terrain.

## 3. Records and the authored journal

- [ ] 3.1 The record type (kind, id, schema version, body) and the store, with unknown kinds kept and written back unchanged. Verify: a test that a record of an unknown kind survives a dig and a quit, byte for byte.
- [ ] 3.2 Author tokens on every journal line, and the `rec` line; old lines read as the player's. Verify: format tests for each author, an old log's replay unchanged, and a torn last line still costing only that line.
- [ ] 3.3 The yield set and the yield check. Verify: tests that a world proposal touching a player-edited cell, piece or record field is refused whole.

## 4. World processes

- [ ] 4.1 The process interface, the steward, `evaluated_through`, and a test-only process. Verify: the catch-up test (ten periods stepwise equals ten at once), and a test that loading runs no process.
- [ ] 4.2 `--age-world <days>`. Verify: an app test that aging a world with the test process journals its entries under the process's author.
- [ ] 4.3 World time passes only while the world is played (the owner, 2026-09-27). Verify: a test that a world saved, left closed and reopened has the same clock and runs no process on opening.

## 5. Regions and checkpoints

- [ ] 5.1 The region of a key by shifts, and region snapshots with their last sequence number. Verify: tests that every finest cell's region contains it, and that a snapshot round-trips.
- [ ] 5.2 The segmented journal and the four-step checkpoint on the writer thread, with a test-only fault hook. Verify: the writer is killed at each step in turn, and every reload holds every committed entry exactly once.
- [ ] 5.3 Loading by region, ahead of the fine set, with held entries for regions not yet loaded. Verify: a synthetic save with a million edits loads reading only nearby regions, the load time is recorded in the risk note, and a far-side flight over it never builds a fine cell before its region.

## 6. Tools and the owner's check

- [ ] 6.1 `tools/save_inspect.py`. Verify: it prints a test save's identity, records, authors and regions, and a test runs it.
- [ ] 6.2 The video (`step-videos`): a large synthetic save loading near the player, a flight across it with regions loading ahead, a quit and reload, and `save_inspect` showing the identity, the records and the journal's authors. Published on the gate page and linked from the PR, with a note that frame cost was not measured in the cloud session. Verify: the page is linked.
- [ ] 6.3 The owner watches the video and approves. Verify: the quote is in `proposal.md`. Sync `world/persistence` with each requirement's test named, and archive.
