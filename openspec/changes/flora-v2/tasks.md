# Tasks

Built on `claude/flora-v2` (2026-11-15): 1, 2 and 4; the leaf cards (3.1)
were built and removed at the owner's word, and the baked impostors (3.2)
are not built; captures and their notes in
`docs/screenshots/flora-v2/README.md`. Frame cost not measured (the owner's
perf sitting); the owner's video waits for the batch.

Ordered by value. Captures go in
`docs/screenshots/flora-v2/`, before and after, from fresh worlds.

## 1. Pixel tuft grass and its toggle

- [x] 1.1 `pbd_core::flora`: the tuft ramps per biome and the per-biome table. Verify: core tests that every biome has a ramp, dark to light by luminance, and that the table's weights are non-negative and the desert grows no fern.
- [x] 1.2 `PlayerSettings::pixel_grass`, file version 5, the GRAPHICS check row, `--grass tufts|blades`, and `FloraLook` to the render world. Verify: settings round-trip and old-file tests.
- [x] 1.3 The uniform's flora lanes (both shaders), the `scatter.ron` knobs and their validation. Verify: the Params struct-size test; config validation tests.
- [x] 1.4 Tufts in the grass slots: crossed cards, the fragment's pattern and ramp, the cut-out in `dropped`, the sway. Verify: shader compiles; captures of tufts in the fields, jungle and swamp, and the blades unchanged with it off.

## 2. Fading and richer clutter

- [x] 2.1 Every clutter piece dithers out over `clutter_fade_m`. Verify: shader test that `dropped` gates kind 3 by the clutter reach.
- [x] 2.2 Flower colours from the biome palette; mushrooms and ferns, with the visibility pass listing them on the same rolls; vertex budget 564. Verify: the visibility GPU test's draw arguments; captures.

## 3. ~~Leaf cards~~ (dropped by the owner, 2026-11-15)

- [x] ~~3.1 Eighteen cards per tree in the foliage draw (306 vertices), the leaf mask, the reach dither, shadows through the cut-out.~~ Built, then removed: the owner, "Hmm trees have weird leave structures on them I don't really like". Q1 is A, trees as today (the owner's own decision); the foliage draw is back to 198 vertices.
- [ ] 3.2 Baked mid and far impostors (design 4, Q2). Deferred: not in this change.

## 4. Seasons

- [x] 4.1 `pbd_core::flora::season_at`, mirrored in WGSL, a test holding the constants together. Verify: core tests (hemispheres opposite, no seasons at the equator, peaks where the design puts them).
- [x] 4.2 The tints and the winter thinning in the shader; the year in the uniform. Verify: captures at `--day` for autumn and winter.

## 5. Checks

- [x] 5.1 `cargo fmt --all -- --check`, clippy, the relevant tests, `openspec validate --all`.
- [x] 5.2 A fresh world launched (not a capture) to READY with no ERROR and no `LATE_PIPELINE`.
- [ ] 5.3 The fungal forest (Q7). Not in this change: a new biome is a new generator version, its own change.
- [ ] 5.4 The owner's video waits for the batch (CLAUDE.md, 2026-09-27); frame cost is the owner's perf sitting.
