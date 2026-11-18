# Design

## Context

- `itch-deploy` builds `pbd-app` for Windows, Linux and Mac (Apple Silicon)
  and pushes each zip with butler to `ruben-tipparach/pale-blue-dot` on the
  channels `windows`, `linux` and `mac` (an optional suffix makes `-beta`
  channels). The user-version is the short SHA and branch.
- A shipped build finds `assets/` beside its exe and keeps its worlds in
  `saves/` beside the exe (`paths::saves_dir`). The player's settings are
  already in the user's config folder (`player-settings`).
- `wharf/latest?target=<user/game>&channel_name=<channel>` needs no key and
  returns the channel's latest user-version. Today it answers `invalid game`,
  because the game's itch visibility is Private (probed 2026-11-18; itch's
  serverside API page gives that as the meaning).
- itch documents no download endpoint for someone who is not the game's
  developer. The itch app downloads with the player's own login, and butler
  with an API key.

## Goals / Non-Goals

**Goals:**
- One download per OS; after that, the player never fetches a build by hand.
- The game never starts on an old build while a new one is reachable.
- No update can leave the player without a game that starts, or touch a save.
- The launcher keeps itself current.

**Non-Goals:**
- Delta patches. Whole builds first; butler's wharf patches are a follow-up.
- Signing, notarisation, installers, Start-menu or dock entries.
- Choosing a beta channel in the launcher's window (a flag only).
- Updating while the game runs. The update happens at the next launcher start.

## Decisions

### 1. Where things live

One data folder per player (Windows `%LOCALAPPDATA%\PaleBlueDot`, Linux
`$XDG_DATA_HOME/pale-blue-dot`, Mac `~/Library/Application Support/PaleBlueDot`):

```text
PaleBlueDot/
  launcher(.exe)          the launcher itself
  current                 one line: the version folder in use
  previous                one line: the version kept to fall back on
  builds/<version>/       one folder per installed build (current + previous)
  downloads/              partial downloads, resumed on the next start
  saves/                  every world, never inside a build
  launcher.log
```

`current` and `previous` are plain files written to a temporary name and
renamed, which is atomic on all three systems. Symlinks are not used, since
Windows needs extra rights for them. Older builds than `previous` are deleted
once the current build has started once.

### 2. How a build gets from itch to the player (the open question)

**What is latest** comes from `wharf/latest`, which is documented and needs no
key. The game's visibility on itch must not be Private for it to answer. That
is a setting on the owner's itch dashboard, not code.

**How it is downloaded** is not settled by documentation, so the first task
is a spike: a measurement instrument, not a feature. It runs once the game is
not Private, and tries in this order:
1. The player signs in to itch once, in the launcher's window, with itch's
   OAuth. The launcher then downloads through the itch API with the
   player's own key, as the itch app does. The key is kept in the system's
   credential store, never in a file.
2. If (1) needs scopes or endpoints itch does not offer for a free game, the
   deploy workflow also publishes the same zips with a small `latest.json`
   (version, size, SHA-256 per OS) to a public GitHub Release of the build.
   itch still says which version is latest; the launcher downloads exactly
   that version's zip from the release.

Scraping itch's web download page is ruled out. It is undocumented, and the
page shows no download ids to a script (tried on a public free game,
2026-11-18).

Recommendation taken (ask only with screenshots): try (1), fall back to (2).
The spike's result is written here before the launcher is built on it.

### 3. Versions

The launcher compares for **equality**, not order. If the channel's latest
is not the installed version, it updates. A rollback pushed to the channel
is then also an "update", which is what a rollback should do.

Every build carries its version twice: in `version.txt` at the root of the
zip, and in `pbd-app --version`. The deploy workflow stamps both from the same
value it gives butler. The launcher reads `version.txt` to know what is
installed, and checks `--version` after unpacking.

### 4. Installing

1. Download into `downloads/<version>.zip`, resuming a partial file.
2. Check it: the size, and the SHA-256 where `latest.json` gives one.
3. Unpack into `builds/<version>.partial/`, set the execute bit on Linux and
   Mac, and check the exe, `assets/` and `version.txt` are there and match.
4. Run `pbd-app --version` from it and compare.
5. Rename the folder to `builds/<version>/`. Then write `previous` with the
   old current version, and `current` with the new one.

Any failure before step 5 leaves the old build in place and running.

### 5. A new build that does not start

The game already logs `READY` once its loading screen finishes. Launched by
the launcher, it also writes `builds/<version>/started-ok` at that moment. If
a freshly installed build exits before writing it, the launcher:
- puts `previous` back as `current`;
- marks that version bad, and skips it until a different version is latest;
- starts the previous build.

### 6. Keeping itself current

The launcher checks `launcher-<os>` before anything else. It downloads the new
launcher next to itself. Then it renames its own running file to
`launcher.old` (Windows allows renaming a running exe, but not overwriting
it), moves the new one into place, and restarts once. The next start deletes
`launcher.old`.

The launcher is kept small and dull, so it rarely needs an update.

### 7. Saves move out of the build

The game takes `--data-dir <path>` from the launcher and uses
`<path>/saves`. Without the flag it behaves as today: a dev run uses
`saves/`, and a hand-unzipped build uses the folder beside its exe.

On first run, the launcher looks for `saves/` beside any exe it was pointed at
(the folder it was started from, or one the player picks), and copies each
world into the data folder. It compares every file by hash. The old copy is
left exactly where it was: nothing is deleted. A world already in the data
folder under the same name is never overwritten; the copy is put beside it
with a suffix, and the launcher logs it.

### 8. The running game

Launched with `--update-channel <channel> --build <version>`, the game asks
`wharf/latest` every 15 minutes, off the main thread. When the answer differs,
its menu shows "Update ready", and nothing else. In multiplayer, a join
refused for a version difference reads "Game version differs. Restart the
launcher to update."

### 9. The window

A small fixed-size window with:
- the game's title art;
- one line of state ("Checking for updates", "Downloading 312 of 540 MB",
  "Offline", "Starting");
- a progress bar;
- Play, which is pressed for the player when the build is ready.

Plain game text, no notes. It is built with `eframe` (egui), so the launcher
is one small binary with no runtime to install. Its look is a mockup first:
the stone-and-carved-wood UI, lit at night.

### 10. The network

HTTPS through `ureq` with `rustls`, so there is no OpenSSL to find on Linux.
Each request has a timeout, and "itch cannot be reached" is decided within a
few seconds, so an offline start is not slow.

### 11. A dedicated server

`pbd-launcher --server` keeps a headless `--server` build current. It checks
on a timer, and switches to a new build only when no player is connected, so
that every player and the server move to a new version together.

## Risks / Trade-offs

- **itch may not offer a download a free game's launcher can use.** That is
  why the spike comes first, and why decision 2 has a fallback that keeps itch
  as the source of "what is latest".
- **The game must not be Private on itch.** Until the owner changes that,
  nothing can check for updates.
- **Unsigned binaries.** On Mac the player must right-click and Open the
  launcher the first time. On Windows, SmartScreen warns. The builds the
  launcher downloads carry no quarantine flag, since it is set by browsers,
  not by the launcher. Signing is a follow-up.
- **Whole builds are large.** Every update is the full zip until wharf patches
  are added.
- **Saves move.** That is the one step that touches player data. It copies,
  verifies and never deletes, so the worst case is two copies.
