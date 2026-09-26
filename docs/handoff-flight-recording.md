# Handoff: recording the flights (parked 2026-09-25)

Parked on the owner's word ("cancel the footage for now"). Everything below
is on `claude/keen-rubin-8b8tb6`. The design and the measured results are in
`openspec/changes/far-side-flight/`.

## State

| Piece | Status |
| --- | --- |
| `--route far-side`: take off, 60 degree climb, 3 km cruise, land on the far side | Built, verified headless, recorded: `dungeon-crawler-2026/pale-blue-dot-far-side-flight.mp4` on Drive (98 s, 60 fps, cadence median 60/60) |
| `--route scenic`: into the clouds, then low over the land | Built, verified headless. **Not recorded**: see "Open" |
| `obs-record` skill (`.claude/skills/obs-record`, also `~/.claude/skills/obs-record`) | Works; hardened by every failure listed under "Gotchas" |
| Frame pacing for flight | Done: see `far-side-flight` tasks 2 and 4 |

## Resume: record the scenic route

1. Close every running `pbd-app.exe`. The skill refuses otherwise, because OBS
   cannot tell two game windows apart.
2. Build: `cargo build --release -p pbd-app`.
3. Record. Timing starts at the route's own lift-off line, and a fresh world
   is used, because a saved world skips the weather spin-up:

```bash
rm -rf target/release/saves/scenic-rec
python .claude/skills/obs-record/scripts/obs_record.py \
  --launch "target/release/pbd-app.exe --route scenic --world scenic-rec" \
  --window-exe pbd-app.exe --start-on ROUTE_LIFTOFF --stop-on ROUTE_COMPLETE \
  --duration 90 --scale 0.5 --fps 60 --out output/scenic.mp4 \
  --contact-sheet --check-cadence
rm -rf target/release/saves/scenic-rec
```

4. Look at the contact sheet and at single frames before calling it done, and
   check the log line `scenic route: into N m of cloud...` to confirm it found
   cloud. The cadence check undercounts on still or near-uniform frames: the
   ground before lift-off, the inside of a cloud, a night sea. Judge those
   seconds by eye.
5. If the game was slow to open, trim the start: `ffmpeg -ss <lift-off - 2 s>`.
   Once, the window took 55 s to appear after a rebuild.
6. Upload by copying the MP4 into `G:/My Drive/dungeon-crawler-2026/`. The Drive
   connector cannot carry a video; Drive for Desktop uploads it. Then confirm
   with a Drive search for the title.

## Open

- The scenic route's camera does not turn toward features as it passes them;
  it uses the far-side rig (pitch from the horizon's dip).
- Clouds drift while the route flies. A cloud planned at lift-off may have
  moved a few hundred metres by the time the ship arrives. It is fine at the
  measured pace, but it is not guaranteed.

## Gotchas found (all handled in the skill now)

- **OBS force-closed:** the next launch stops at "Run in Safe Mode?", and Safe
  Mode has no WebSockets. The skill clears stale `.sentinel` markers and closes
  OBS politely; never use `--minimize-to-tray`.
- **A dead `obs64.exe` (status Unknown)** can hold OBS's single-instance lock.
  The skill launches with `--multi`; a reboot clears the dead process.
- **OBS answers "not ready" (code 207)** while it is still loading: the skill
  waits.
- **A window capture reports no size** until its scene is on program.
- **OBS refuses to change video settings** while the recording output is still
  finishing. The skill waits for it, then restores every setting one by one,
  including removing its own input: removing a scene does not remove its
  inputs, and an orphan was once saved into the owner's scene collection.
- **Capture mode (`--capture`) is not real time**: it steps the simulation a
  fixed amount per frame. Measure pacing and record in an ordinary windowed
  run.
