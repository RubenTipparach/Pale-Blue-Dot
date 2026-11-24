# Multiplayer test on two machines (handoff)

The owner, 2026-11-23: "Multiplayer test should use Mac and laptop mining
mike sessions". The procedure is `openspec/changes/multiplayer` design 20,
"The real test is on two machines". This file is what each machine's
session does; PR #148 (`RubenTipparach/Pale-Blue-Dot`) is where they
report, and the cloud session that wrote this watches its comments.

Branch: `claude/medieval-city-html-mockup-8692xs`. Never touch another
checkout's work: build in a separate folder. Never merge the PR, never
force-push.

## Mac: hosts

1. In the Pale-Blue-Dot repo:
   `git fetch origin claude/medieval-city-html-mockup-8692xs && git worktree add ../pbd-mp-test FETCH_HEAD`
   (if `../pbd-mp-test` exists: `cd ../pbd-mp-test && git checkout --detach FETCH_HEAD`).
2. In `../pbd-mp-test`: `cargo build --profile fast -p pbd-app`.
3. The LAN address: `ipconfig getifaddr en0` (`en1` if empty).
4. Start the host in a window, in the background, logging:
   `target/fast/pbd-app --world mp-test --host 47123 2> host.log &`
   It prints `hosting on 0.0.0.0:47123`. If macOS asks whether pbd-app may
   accept incoming connections, the owner clicks Allow.
5. Comment on PR #148, exactly: `MAC HOSTING <lan-address>:47123` and the
   commit built, with the Claude Code attribution footer.
6. Keep it running up to 45 minutes. Dig a hole near the spawn (hold the
   left mouse). Watch `host.log` for `player 1 joined`, `edits from the
   network applied`, `SERVER_DISAGREES`, `refused`.
7. After `edits from the network applied`:
   `mkdir -p docs/screenshots/multiplayer-two-machines && screencapture -x docs/screenshots/multiplayer-two-machines/mac-host.png`,
   and save the matching `host.log` lines as `mac-host.log` beside it.
8. Commit only those two files and push to the branch (fetch and merge
   first if it moved). Comment on the PR with what you saw.

## Laptop: joins

1. Clone beside the other repos, not inside the Mining Mike checkout:
   `git clone https://github.com/RubenTipparach/Pale-Blue-Dot pbd-mp-test && cd pbd-mp-test && git checkout claude/medieval-city-html-mockup-8692xs`
   (if it exists: `git fetch origin claude/medieval-city-html-mockup-8692xs && git checkout --detach FETCH_HEAD`).
2. `cargo build --profile fast -p pbd-app` (the first build is long).
3. Wait for the Mac's `MAC HOSTING <address>:47123` comment on PR #148.
4. Join: `target/fast/pbd-app --join <address>:47123 2> join.log`
   (on Windows: `target\fast\pbd-app.exe`). The join waits up to two
   minutes for the host; `join.log` says `joined <address> as player 1`.
   On Windows, allow the firewall prompt if one appears.
5. Look at the Mac's hole (it should be there), then dig one of your own
   near it.
6. Screenshot showing both holes into
   `docs/screenshots/multiplayer-two-machines/laptop-joined.png`, and save
   the `joined`, `sent to the host`, `edits from the network applied` and
   any `refused` lines of `join.log` as `laptop-joined.log` beside it.
7. Commit only those two files and push to the branch (fetch and merge
   first). Comment on the PR with what you saw.

## If it fails

Comment the error on PR #148 and stop: a build error, `could not host`,
`could not join <address>: no welcome` (the Mac's firewall, or the machines
are on different networks), or anything unexpected in the logs.
