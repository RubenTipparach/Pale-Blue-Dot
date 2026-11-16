# Proposal: an itch.io deploy pipeline for Windows, Linux and Mac

## Why

The owner, 2026-11-16: "can you also add an itch deploy pipeline as well, one
pipeline I can trigger on any branch for mac, linux and windows".

The repository has no CI at all (no `.github/`), and the game cannot be
shipped as it is. Every runtime read of `assets/` is a path built at compile
time from `env!("CARGO_MANIFEST_DIR")`. An exe built on a CI runner therefore
looks for `D:\a\pale-blue-dot\...\assets` (or `/Users/runner/...`) on the
player's machine and finds nothing. Saves are a path relative to the working
directory. A double-clicked exe on macOS (working directory `/`), or a
shortcut with another start-in folder, would write its worlds somewhere the
player never sees, or fail to write them at all.

## What

1. **One manually triggered workflow**, `.github/workflows/deploy-itch.yml`.
   It is `workflow_dispatch` only, so it runs on whichever branch is picked in
   "Run workflow". It builds `pbd-app` with `--release` on Windows, Linux and
   macOS (Apple Silicon), and packages each as the exe, the whole `assets/`
   folder beside it and a `README.txt`. Each package is uploaded as a workflow
   artifact. When the `deploy` box is ticked (the default), each is pushed to
   itch with butler on the channels `windows`, `linux` and `mac`, each with an
   optional suffix (`-beta`).
2. **The game finds its assets beside its exe.** One function,
   `pbd_app::paths::assets_dir()`, chooses `<exe dir>/assets` when that folder
   exists and the compiled-in checkout's `assets/` otherwise. Every runtime
   site that built the path from `CARGO_MANIFEST_DIR` calls it, Bevy's
   `AssetPlugin` among them. Tests and examples keep reading the checkout.
3. **A shipped game keeps its saves beside its exe.** `pbd_app::paths::saves_dir()`
   is `<exe dir>/saves` in a shipped build (one whose `assets/` sits beside the
   exe). In a dev run it stays today's working-directory-relative `saves`, so
   every world a dev run has written (`saves/` at the repository root,
   `target/fast/saves`) opens exactly where it is today.

## Not in this change

- Code signing or notarisation of the Mac build, and an Intel Mac build.
- A `.app` bundle, installers, or any automatic trigger (push, tag, schedule).
- Version stamping inside the game. The build's identity is itch's
  `--userversion` (short SHA and branch) and the README.
