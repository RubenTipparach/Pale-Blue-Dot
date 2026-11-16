# Design: itch-deploy

## 1. The workflow

`.github/workflows/deploy-itch.yml`, triggered by `workflow_dispatch` alone.
GitHub runs a dispatched workflow on the branch picked in "Run workflow", so
"any branch" needs no input of its own.

Inputs:

| Input | Type | Default | Meaning |
| --- | --- | --- | --- |
| `deploy` | boolean | `true` | Push to itch. Untick for a build-only dry run. |
| `channel_suffix` | string | empty | Appended to every channel name: `-beta` gives `windows-beta`. |

Jobs:

- **`build`** is a matrix over `windows-latest`, `ubuntu-latest` and
  `macos-15` (Apple Silicon), with `fail-fast: false` so one platform's
  failure does not hide the others'. Each job:
  - checks out the code;
  - installs the pinned toolchain, `dtolnay/rust-toolchain@1.97.1`. That is
    the rustc the owner's Mac builds this tree with today (2026-11-16), and it
    is above the workspace's `rust-version` of 1.89 and edition 2024's 1.85.
    The tree is not checked on 1.89 itself, so the pin is the known-good one;
  - restores `Swatinem/rust-cache`;
  - on Linux, installs Bevy's system packages: ALSA for kira's cpal, udev,
    Wayland, xkbcommon and the X11 headers for the `x11` feature;
  - runs `cargo build --release --locked -p pbd-app`. `--release` is the
    profile for a shipped build (CLAUDE.md: `fast` for play-testing,
    `--release` for performance and for what players run).
- **Packaging** makes a folder holding the exe, a copy of the whole
  `assets/` and a `README.txt`. The README says what the build is, its commit
  and branch, how to run it, and the Mac quarantine step. The job zips the
  folder itself so the Unix executable bit survives
  (`actions/upload-artifact` would drop it). The zip is uploaded as the
  artifact `pale-blue-dot-<platform>` whether or not the run deploys.
- **`deploy`** runs on Ubuntu after every build succeeds, and only when
  `deploy` is true. It:
  - checks the `BUTLER_API_KEY` secret first and fails with a message saying
    to add it;
  - installs butler and downloads the three zips;
  - pushes each through `.github/scripts/butler-push.sh` to
    `ruben-tipparach/pale-blue-dot:<windows|linux|mac><suffix>` with
    `--userversion <short sha>-<branch>`. The script is godot-sandbox's
    wrapper, which retries only itch's "latest build is still processing"
    refusal.

  Two things keep runs from racing on a channel: the one job pushes one
  platform at a time, and the deploy job's `concurrency` group
  (`deploy-itch`, `cancel-in-progress: false`) queues runs. The group never
  kills a run that has already pushed some channels. It sits on the deploy
  job alone, so two runs still build at the same time. GitHub keeps one
  pending run per group: a third run queued behind a waiting second replaces
  it.

One secret, `BUTLER_API_KEY`: an itch API key, from
https://itch.io/user/settings/api-keys. The game is not a secret: the owner,
2026-11-16, gave it as https://ruben-tipparach.itch.io/pale-blue-dot, so the
workflow names `ITCH_GAME: ruben-tipparach/pale-blue-dot` in its top-level
`env`.

## 2. Assets beside the exe

`crates/pbd-app/src/paths.rs`:

```rust
pub fn assets_dir() -> PathBuf          // the one the game uses
pub fn shipped_root() -> Option<PathBuf>
fn shipped_in(exe: Option<&Path>) -> Option<PathBuf>  // the rule, tested
```

The rule: if the current exe's directory holds an `assets` directory, that is
the asset root. Otherwise it is the compiled-in
`CARGO_MANIFEST_DIR/../../assets`. A dev build in `target/fast/` has no
`assets/` beside it, so a dev run is unchanged. The choice is made once and
cached (`OnceLock`), so every reader in a run agrees.

These runtime sites move to it, each a one-line change so the concurrent work
on water files merges cleanly:

- `AssetPlugin { file_path }` in `desktop.rs`;
- `config::config_dir`;
- `atmosphere::climate_dir`;
- `world_map::tileset_dir`;
- `night_lights::rules_path` and `sites::rules_path`;
- `towns::asset`;
- `ui_kit::assets_dir`;
- `audio::engine::audio_dir`;
- `vehicles::model::folder`.

Everything else that reads assets at run time already goes through one of
these (`missions`, `planet_trees`, `audio::mod`, the showcases).
`include_str!` sites embed the file at compile time and need nothing. Tests
and examples keep `CARGO_MANIFEST_DIR`, because they run in the checkout.

## 3. Saves beside the exe, in a shipped build only

`saves::ROOT` is `"saves"`, resolved against the working directory. In a dev
run that is the repository root (`play.bat`, `cargo run`), or `target/fast`
when the exe is started from there; worlds exist in both today. A shipped
build started from a Finder double-click on macOS has `/` as its working
directory. From a Windows shortcut it has whatever the start-in folder is.

`paths::saves_dir()` returns:

- **shipped** (the exe's directory holds `assets/`): `<exe dir>/saves`;
- **dev** (otherwise): `PathBuf::from("saves")`, exactly today's path.

The "shipped" test is the same one `assets_dir` makes, so a run is one thing
or the other for both. No existing world moves: no dev exe has `assets/`
beside it, and no shipped build existed before this change.

These readers of `saves::ROOT` call `saves_dir()` instead:

- `open_world`, and the world create and list calls in `desktop.rs`;
- the two menu fallbacks;
- `WorldSave::default`;
- `tone::preferences_path`.

`ROOT` stays as the dev name. `settings.ron` already lives in the OS's
per-user config folder and is untouched.

Itch's own app installs a game into a writable per-user folder. A player who
unzips by hand unzips somewhere they can write. So `<exe dir>/saves` is
writable in both cases.

## 4. Limits, stated

- **The Mac build is unsigned.** macOS quarantines a downloaded unsigned
  binary, and Gatekeeper refuses to open it ("cannot be opened because the
  developer cannot be verified"). The player runs
  `xattr -dr com.apple.quarantine <folder>` once, or right-clicks the exe and
  chooses Open. The itch app does not set the quarantine flag, so a player who
  installs through it is not affected. The README says this.
- **Apple Silicon only** (`macos-15` is arm64). An Intel or universal build
  would add an `x86_64-apple-darwin` target and `lipo`. That was not asked for.
- **Build time.** A cold `--release` build (thin LTO) is long: the desktop
  takes 9-15 minutes, and GitHub's runners are slower. `rust-cache` keeps
  dependencies between runs of the same branch and OS.
- **Linux glibc.** The Linux build links the glibc of `ubuntu-latest`, so an
  older distribution may refuse it.
