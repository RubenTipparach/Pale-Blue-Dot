# Tasks

## 1. Write-up

- [x] 1.1 Proposal, design and spec delta (this change); `openspec validate --all`.

## 2. Paths

- [x] 2.1 `pbd_app::paths`: `assets_dir()` and `saves_dir()`, with the choice
      in a pure function and tests for both branches.
- [x] 2.2 Every runtime `CARGO_MANIFEST_DIR` asset site and `AssetPlugin`
      call `assets_dir()`; tests and examples untouched.
- [x] 2.3 Every runtime `saves::ROOT` reader calls `saves_dir()`.
- [x] 2.4 Local check of a shipped build. A `--profile fast` exe is copied
      into a temp folder with `assets/` beside it and launched from another
      working directory. It reaches `READY` with no `LATE_PIPELINE` and writes
      its world under `<exe dir>/saves`. The same exe with no `assets/` beside
      it falls back to the checkout's assets.
      Done 2026-11-16 on the M4 Max Mac: with assets, `READY` at 10.2 s, 0
      `LATE_PIPELINE`, the world in `<exe dir>/saves`; a corrupted
      `sites.ron` in the copy panicked naming the copy's path, so the copy is
      what is read. Without assets, `READY` at 10.4 s, 0 `LATE_PIPELINE`, the
      world in the working directory's `saves`, as before.

## 3. Workflow

- [x] 3.1 `.github/workflows/deploy-itch.yml` and `.github/scripts/butler-push.sh`.
- [x] 3.2 `actionlint` passes.
- [ ] 3.3 The owner adds the `BUTLER_API_KEY` secret, runs the workflow
      once as a dry run and once deploying, and opens each platform's build.
