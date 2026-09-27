# Tasks

The first change in the plan (the owner, 2026-09-27: "fix cell number yes").
It ships before `step-videos` and the lights, so its video is made by hand.

## 1. The measurement, as a test

- [x] 1.1 `pbd_core::cell_key` with `point_id`'s mix moved into it, and a test that counts collisions over every finest cell. It pins 202,571 for the hash. Verify: it reproduces the scratch measurement exactly, and its run time is recorded in the design.
- [x] 1.2 A tool mode that lists the colliding pairs nearest a point. Verify: it names the pair nearest the spawn, which is used for the video. Done: `pbd-app --colliding-pairs [count]`. The nearest pair to the spawn (25 m) has its cells' ground 31 m apart, so an edit at one surface is air at the other, and the video uses the nearest pair on one ground layer instead (303 m; `docs/screenshots/exact-cell-keys/README.md`).

## 2. The key

- [x] 2.1 Pack and unpack (face, level, i, j) in 31 bits, with edge points re-addressed on their lowest face. Verify: tests that every finest key is unique (the collision test now asserts zero), that every key unpacks to its own cell, and that every icosahedron edge's points get one key from either side.
- [x] 2.2 `planet_lod.rs` writes the key into the record's `spare[1]` (design decision 2, revised), and every Rust reader takes it through `GpuCell::key()`. Verify: an app test that the same cell gets the same key from anchors on two different faces, and the existing dig, crack and adopt tests pass unchanged.

## 3. The same look

- [x] 3.1 The seed lane, `metadata[3]`, keeps the old hash, so no shader changes. Verify: an app test that every record of a spawn tier carries `cell_key::old_hash` of its address in `metadata[3]` and its exact key in `spare[1]`, and `git diff` shows no `.wgsl` file changed.
- [x] 3.2 Captures of the `meadow` and `surface` views on the old build and the new. Verify: the two are pixel-identical, and both are in `docs/screenshots/exact-cell-keys/`. Done: 0 of 1,296,000 pixels differ in either view.

## 4. Old saves

- [x] 4.1 The migration to `edits.v1.log`: the used hashes' reverse table, resolution by surface and then position, both candidates written when neither settles it, and a barrier before the world is shown. Verify: test saves with a unique key, a pair told apart by the surface, a pair told apart by position, and a truly ambiguous pair; `edits.log` unchanged byte for byte; the load time for a synthetic 100,000-edit log recorded in the design.
- [x] 4.2 The loader prefers `edits.v1.log` and appends only to it. Verify: format tests, and a test that a second open does not migrate again.

## 5. The owner's check

- [x] 5.1 The gate video, hand-made from a capture sequence stitched with Playwright's ffmpeg. Its shots are:
  - the colliding pair's cell A dug;
  - B dug on the old build;
  - B untouched on the new build;
  - a reload;
  - the unchanged ground.

  It is published on a gate page, with a note that frame cost was not measured in the cloud session. Verify: the page is linked from the PR.
  Done: the gate page is https://claude.ai/artifact/UkLbLNDm3mfS17Wu3K6uWe, and the video and stills are in `docs/screenshots/exact-cell-keys/`.
- [ ] 5.2 The owner watches it and accepts. Verify: the quote is in `proposal.md`. Sync `world/persistence`, and archive.
  Deferred to the batch (the owner, 2026-09-27: "skip videos for now, we'll verify them all together all in a batch"). The change is merged and stays open until then.
