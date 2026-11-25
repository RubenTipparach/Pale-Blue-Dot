# Tasks

Phase 3b, beside `multiplayer`. Nothing here starts until the owner asks.

## 1. Before anything is built

- [ ] 1.1 The owner sets the game's itch visibility to anything but Private. Verify: `wharf/latest` answers each of `windows`, `linux` and `mac` with a version, not `invalid game`.
- [ ] 1.2 Spike (a measurement instrument, design 2): download the current Linux build through itch with a player's own login, and record what worked, with the exact endpoints and scopes, in design 2. If it cannot work for a free game, record why and take the fallback. Verify: the finding is written in design 2 before 3.1 starts.

## 2. Builds the launcher can trust

- [ ] 2.1 The deploy workflow stamps one version into `version.txt`, `pbd-app --version` and butler's `--userversion`. Verify: a workflow run's three zips each carry the same version in all three places.
- [ ] 2.2 The deploy workflow builds and pushes the launcher on `launcher-windows`, `launcher-linux` and `launcher-mac`, plus `latest.json` with a GitHub Release if 1.2 took the fallback. Verify: a run on a branch shows the six channels on itch.

## 3. Saves out of the build

- [ ] 3.1 `pbd-app --data-dir <path>` keeps worlds in `<path>/saves`; without it, today's rule. Verify: a test per case, and an old world opens from both.
- [ ] 3.2 The launcher's one-time copy of `saves/` beside an older exe (design 7): hash-verified, never deletes, never overwrites. Verify: a test with a synthetic old install, a name clash, and a copy interrupted halfway.

## 4. The launcher

- [ ] 4.1 Mockup of the launcher window (design 9), published and on the tracker. Verify: the owner approves it.
- [ ] 4.2 `pbd-launcher` crate: data folder layout, OS channel choice, `wharf/latest`, offline detection within a few seconds. Verify: unit tests for channel choice and for the offline path against a refused connection.
- [ ] 4.3 Download, resume, check, unpack, `--version` check, atomic switch (design 4). Verify: tests that cut the download, corrupt the zip and remove the exe, and each leaves the old build current and starting.
- [ ] 4.4 Fall back from a build that never reaches `READY` (design 5). Verify: a test with a stub build that exits early.
- [ ] 4.5 Self-update (design 6). Verify: on all three systems in CI, an older launcher replaces itself with a newer one and restarts once.
- [ ] 4.6 The window, from the approved mockup. Verify: screenshots of each state in `docs/screenshots/game-launcher/`.

## 5. The running game

- [ ] 5.1 "Update ready" on the menu (design 8). Verify: an app test with a stubbed version answer.
- [ ] 5.2 The multiplayer refusal names the launcher (with `multiplayer` 6.1). Verify: the message appears on a version mismatch.
- [ ] 5.3 `pbd-launcher --server` (design 11). Verify: a test where a new build waits until no player is connected.

## 6. The owner's gate

- [ ] 6.1 A fresh machine for each OS: download only the launcher, start it, play, push a new build, start again, see it update; a world survives. Screenshots of each step. Verify: the owner has seen them.

## Workflow follow-up

- Sync `platform/launcher` into `openspec/specs/` requirement by requirement, with the test that proves each.
- Follow-ups: wharf delta patches, code signing and notarisation, a beta channel in the window.
