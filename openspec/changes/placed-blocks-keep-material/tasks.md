# Tasks

## 1. Find the cause
- [x] 1.1 Trace a placement: the save (`saves/format.rs`), the column
      (`column::generate_edited`), the record (`reconcile_surface`), the
      column pass's codes (`gpu_materials`), the dig and its drop
      (`digging.rs`), the footstep (`audio/hooks.rs`). Stored right; drawn
      from the biome's sheet (`planet_surface.wgsl`).

## 2. The rule in core
- [x] 2.1 `Column::place`, `Column::placed`, `Column::drawn_sheet`,
      `column::home_biome`; `set` clears the mark; edits apply with `place`.
- [x] 2.2 A placed block is not dressed as a river bed.
- [x] 2.3 Tests: every placeable block on desert sand, snow, a river bed, a
      beach under the sea and a lane, and on another placed block; under
      water; only an edit places (`column::placed_tests`).

## 3. Drawing it
- [x] 3.1 The tier's `set_layer` places; `gpu_placed_sheets` after the codes;
      the record's placed sheet in `reconcile_surface`.
- [x] 3.2 `planet_surface.wgsl`: the cap, the heightfield wall's top metre and
      the column pass take a placed block's own sheet.
- [x] 3.3 The pack's `block_art` reads `home_biome`.
- [x] 3.4 Tests: written, saved, reopened, built and drawn, every block on
      every ground (`planet::column::placed_block_tests`); untouched ground
      unchanged; the dig's yield (`digging::placed_tests`); the slot and the
      ground share a sheet (`slots::tests`); the shader's constants.

## 4. Verify
- [x] 4.1 `cargo test -p pbd-core`, `cargo test -p pbd-app` (the ground
      digests included), fmt, clippy, `openspec validate --all`.
- [x] 4.2 Captures, before and after, of a row of placed blocks on desert
      sand (`--place-row`), in `docs/screenshots/placed-blocks/`.
- [ ] 4.3 `tools/perf_suite.py` on a quiet machine: the upload grew by one
      nibble buffer (design 4). Not run here.
- [ ] 4.4 The owner places blocks in the desert and agrees.
