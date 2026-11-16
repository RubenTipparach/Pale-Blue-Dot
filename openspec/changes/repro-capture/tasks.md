# Tasks

## 1. The record

- [ ] 1.1 `pbd_app::place::heading_at_yaw` and `yaw_of`; the walker's and the flight's starts call the one function.
- [ ] 1.2 `pbd_app::repro::Record`: the fields, the text, and the command line.
- [ ] 1.3 `--generator <v>` for a memory-only run.
- [ ] 1.4 `build.rs`: `PBD_GIT_COMMIT`.

## 2. The key

- [ ] 2.1 The chord row in the controls table (listed, not rebindable).
- [ ] 2.2 `desktop/repro.rs`: the physical-key reader, the screenshot, the files, the `REPRO` line, the clipboard, the note.

## 3. Verify

- [ ] 3.1 Test: every field is in the record; the command parses back through `Launch::parse` to the same place, yaw, pitch, hour and day.
- [ ] 3.2 In the game: the key pressed by a scripted input, the command run, both shots in `docs/screenshots/repro-capture/`.
- [ ] 3.3 fmt, clippy, tests (`--profile fast`), a launch to READY with no `LATE_PIPELINE`, `openspec validate --all`.
