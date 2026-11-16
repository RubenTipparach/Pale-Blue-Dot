# Tasks

## 1. The record

- [x] 1.1 `pbd_app::place::heading_at_yaw` and `yaw_of`; the walker's and the flight's starts call the one function.
- [x] 1.2 `pbd_app::repro::Record`: the fields, the text, and the command line.
- [x] 1.3 `--generator <v>` for a memory-only run.
- [x] 1.4 `build.rs`: `PBD_GIT_COMMIT`.

## 2. The key

- [x] 2.1 The chord row in the controls table (listed, not rebindable).
- [x] 2.3 The block dump (`repro/blocks.rs`): 3 hex rings, 3 layers past the feet and the head, as a table and as JSON, with a test against the tier.
- [x] 2.2 `desktop/repro.rs`: the physical-key reader, the screenshot, the files, the `REPRO` line, the clipboard, the note.

## 3. Verify

- [x] 3.1 Test: every field is in the record; the command parses back through `Launch::parse` to the same place, yaw, pitch, hour and day.
- [x] 3.2 In the game: the key pressed by a scripted input, the command run, both shots in `docs/screenshots/repro-capture/`.
- [x] 3.3 fmt, clippy, tests (`--profile fast`), a launch to READY with no `LATE_PIPELINE`, `openspec validate --all`.
