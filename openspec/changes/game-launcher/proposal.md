## Why

The owner, 2026-11-18: "as part of 3b is a self contained launcher that pulls
the latest from itch io based on your OS", and "this launcher should be
responsible for keeping the game up to date always".

Multiplayer (`multiplayer`, phase 3b) refuses a join when the two games'
versions differ. Without something that keeps every player on the latest
build, that refusal is what players will meet most. The launcher is the other
half of the join check: it makes "Game version differs" rare.

Today a player downloads a zip from itch by hand (`itch-deploy`). Nothing
tells them a newer build exists, and nothing installs it.

**Found while writing this up (2026-11-18):**
- itch's public version endpoint, `wharf/latest`, needs no key and returns
  the latest user-version of a channel. For our game it answers
  `{"errors":["invalid game"]}` on all three channels. itch's documentation
  (`itch.io/docs/api/serverside`) says that is what it returns for a game
  whose visibility is Private. The page itself loads. **The game must be
  public, or at least not Private, before any launcher can check it.**
- itch documents no endpoint that downloads a game's build for someone who
  is not its developer. The itch app does it with the player's own itch
  login. butler does it with an API key. Neither is a documented anonymous
  download. How the launcher downloads from itch is therefore the first
  thing to prove (design 2), before anything is built on it.
- Shipped builds keep their saves beside the exe (`itch-deploy`,
  `paths::saves_dir`). A launcher that replaces the game's folder would
  delete them. Saves have to move out of the build folder first, without
  losing a world (CLAUDE.md, "Saved games survive every change").

## What Changes

- **A small, self-contained launcher per OS** (Windows, Linux, Mac), the one
  thing a player downloads and runs. It checks itch for the latest build of
  its OS's channel, downloads it, verifies it, and starts the game.
- **The game is always up to date.** The launcher checks on every start, and
  the game is not started on an older build while a newer one can be
  fetched. Offline, it starts the last build it installed and says so in one
  short line.
- **An update can never break the install.** A new build is downloaded and
  unpacked beside the current one, checked, then switched to in one step. The
  previous build is kept, so a failed or broken update falls back to it.
- **Saves and settings live outside the builds**, in the player's own data
  folder, so an update never touches them. Worlds already beside an older
  exe are moved there once, on first launch, and never deleted.
- **The launcher updates itself** from its own itch channel.
- **A running game hears about a new build.** The game shows a plain "Update
  ready" on the menu when the launcher has one waiting. A multiplayer join
  refused for version names the launcher as the fix.
- The deploy workflow (`itch-deploy`) pushes the launcher on its own channels
  and stamps every build with the version the launcher compares.

Not in this change: delta patches (whole builds first; butler's wharf
patches are a follow-up), code signing and notarisation, installers or
Start-menu entries, beta channels in the launcher's UI, and Steam.

## Capabilities

### New Capabilities
- `platform/launcher`: what the launcher guarantees: which build it fetches
  for an OS, that the game starts only on the latest reachable build, that an
  update is atomic and reversible, that saves and settings are never inside a
  build, and how it updates itself.

### Modified Capabilities
<!-- None in openspec/specs. Where saves are kept is `itch-deploy`'s
`paths::saves_dir`, a change not yet synced to the main specs; this change's
tasks move it, with the old location migrated. -->

## Impact

- **New crate `pbd-launcher`**: a small binary with no Bevy, no renderer and
  no game code. Its window is a mockup first, in the stone-and-carved-wood
  UI, lit at night.
- **`pbd-app`**: takes its saves folder from the launcher (`--data-dir`),
  migrates worlds from beside the exe once, shows "Update ready", and names
  the launcher in the version-refusal message.
- **`.github/workflows/deploy-itch.yml`**: launcher builds on
  `launcher-windows`, `launcher-linux` and `launcher-mac`, and a build version
  the launcher can compare.
- **itch page settings (the owner's):** the game's visibility must not be
  Private for `wharf/latest` to answer.
- **Saves:** moved once, never rewritten. Every world opens as before.
