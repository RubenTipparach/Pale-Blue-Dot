# Tasks

This comes with step 2a, before any change writes new kinds of state
(`bigger-biomes`' version, `city-sites`' records). Group 1 fixes a bug in the
current game, and the owner may pull it ahead of the lights (Open Questions).

## 1. Exact cell keys

- [ ] 1.1 A core test that counts `point_id` collisions over every finest cell. It pins the measured 202,571 before the fix, then asserts zero with the new key. Verify: the test fails on the old key and passes on the new.
- [ ] 1.2 The packed finest-level key (face, i, j in 29 bits), with face-edge points on their lowest face, carried in `metadata[3]`. Verify: tests that every finest cell has a unique key, that the same cell gets the same key from two anchors, and that a key unpacks to its own cell.
- [ ] 1.3 Migration of hashed keys, resolved by surface and position, with the ambiguous remainder applied to both and listed. Verify: a test save with a colliding pair migrates to the right cell, and a truly ambiguous one is applied to both and logged.

## 2. Identity

- [ ] 2.1 `TOPOLOGY_VERSION`, and `identity.ron` written with a barrier at creation; an old slot gains one as generator 4, topology 1, key 0. Verify: format tests for both, and a test that a build refuses an unknown version with the version named.
- [ ] 2.2 The world's config is chosen from the identity, so `bigger-biomes` can add version 5 without touching the save. Verify: an app test that an old slot generates version 4's terrain.

## 3. Records and the authored journal

- [ ] 3.1 The record type (kind, id, schema version, body) and the store, with unknown kinds kept and written back unchanged. Verify: a test that a record of an unknown kind survives a dig and a quit, byte for byte.
- [ ] 3.2 Author tokens on every journal line, and the `rec` line; old lines read as the player's. Verify: format tests for each author, an old log's replay unchanged, and a torn last line still costing only that line.
- [ ] 3.3 The yield set and the yield check. Verify: tests that a world proposal touching a player-edited cell, piece or record field is refused whole.

## 4. World processes

- [ ] 4.1 The process interface, the steward, `evaluated_through`, and a test-only process. Verify: the catch-up test (ten periods stepwise equals ten at once), and a test that loading runs no process.
- [ ] 4.2 `--age-world <days>`. Verify: an app test that aging a world with the test process journals its entries under the process's author.

## 5. Regions and checkpoints

- [ ] 5.1 The region of a key by shifts, and region snapshots with their last sequence number. Verify: tests that every finest cell's region contains it, and that a snapshot round-trips.
- [ ] 5.2 The segmented journal and the four-step checkpoint on the writer thread, with a test-only fault hook. Verify: the writer is killed at each step in turn, and every reload holds every committed entry exactly once.
- [ ] 5.3 Loading by region, ahead of the fine set, with held entries for regions not yet loaded. Verify: a synthetic save with a million edits loads reading only nearby regions, the load time is recorded in the risk note, and a far-side flight over it never builds a fine cell before its region.

## 6. Tools and the owner's check

- [ ] 6.1 `tools/save_inspect.py`. Verify: it prints a test save's identity, records, authors and regions, and a test runs it.
- [ ] 6.2 The video (`step-videos`): digging one cell of a colliding pair before and after the fix, with the twin shown far away; a quit and reload; and `save_inspect` on a large save. Published on the gate page and linked from the PR, with a note that frame cost was not measured in the cloud session. Verify: the page is linked.
- [ ] 6.3 The owner watches the video and approves. Verify: the quote is in `proposal.md`. Sync `world/persistence` with each requirement's test named, and archive.
