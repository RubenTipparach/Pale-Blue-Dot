# River craft in the game (`river-craft` design 11)

The first part of M34 River Craft's port, in the game: the raft rowed with
two oars, its one anchor, the boat's menu, the raft's lantern at night and
the tube rider paddling with the hands. Compare with the mockup's stills in
`docs/screenshots/raft-rowing/` (`oars-v4-*`, `boat-menu-v6-*`,
`anchor-v4-*`, `tube-v4-*`) and the live mockup,
https://claude.ai/artifact/GZTZJ4Vg1bSbg4SH2rYY97.

Rendered headless in a Claude Code cloud container: `xvfb-run` with Mesa's
lavapipe (a software renderer, no GPU), the `fast` build, fixed-step
capture. They show what things look like, never how smoothly they run; the
frame cost was not measured.

Shots logged `ROW_TRACE` lines with `PBD_ROW_TRACE=1` (the stroke's stage and the right grip each time the stage changes, `vehicles.rs`). Every shot is on the owner's desert river (`--at 23.644 -2.685`), dry
(`--rain 0 --weather-at 7200`), at 1200 x 750 (`PBD_WINDOW=1200x750`), from
the fast build:

```sh
cargo build --profile fast -p pbd-app
PBD_WINDOW=1200x750 PBD_KEY_SCRIPT="<keys>" xvfb-run -a -s "-screen 0 1200x750x24" \
  target/fast/pbd-app --capture docs/screenshots/river-craft-game/<shot>.png \
  --aboard raft --at 23.644 -2.685 --rain 0 --weather-at 7200 <args>
```

`PBD_KEY_SCRIPT` presses a key on a capture frame and lets it go two frames
later (`desktop/repro.rs`), so one W starts one stroke; the capture is taken
the given number of frames (1/60 s each) after it.

| Shot | What it shows | `<keys>` and `<args>` |
| --- | --- | --- |
| `rowing-chase-drive.png` | Third person, mid-drive (0.58 s into the stroke): the rowing frame across the planks, both oars out with their blades aft in the water, the rower on the bench facing the bow, his arms from the shoulders to the handles. | `115:KeyW`, `--frames 150 --time 11` |
| `rowing-seat-drive.png` | First person, late in the drive (0.85 s into the stroke), the look 40 degrees down (the rower's default, design 11.6): the right hand on its handle at the frame's right edge, the left out of frame. See design 11.6: with the game's 60-degree view the hands sit at the edges through most of the stroke. | `99:KeyW`, `--frames 150 --time 11 --seat` |
| `rowing-seat-release.png` | First person at the release (0.95 s in): the hands pushed forward, the blades coming out. | `93:KeyW`, `--frames 150 --time 11 --seat` |
| `anchored-chase.png` | T dropped the anchor 3 s before: the panel reads "RAFT - AT ANCHOR" and "T WEIGH THE ANCHOR"; the raft has swung on its rode, which stopped at the bed (4.2 m out). | `40:KeyT`, `--frames 220 --time 11` |
| `boat-menu.png` | C at anchor: the pack with the HOLD and the BOAT panel beside it: "RAFT DRY", Anchor "Holding" with Weigh, Rope "To the bed" with - and + (- and To the bed dead while it is to the bed), "Out 4.2 m", Lantern on. | `40:KeyT,170:KeyC`, `--frames 210 --time 11` |
| `lantern-night-chase.png` | 22:00, far from any town: the raft's own lantern lights its deck, the oars, the water round it and the near bank, warm, the rest of the canyon dark; the lantern's flame burns. | none, `--frames 120 --time 22` |
| `lantern-night-seat.png` | 22:00, first person mid-drive (taken with the look 31 degrees down, before the default moved to 40): the hands on the handles and the deck lit by the lantern at the bow. | `85:KeyW`, `--frames 120 --time 22 --seat` |
| `tube-chase-paddle.png` | The inner tube 0.2 s into a stroke on the right: the rider sits back in the ring, the right hand pulling through the water beside it, the left arm resting along the rim; no lantern. | `110:KeyW`, `--aboard tube`, `--frames 122 --time 11` |

Each `--aboard` capture logs 32 `LATE_PIPELINE` lines after `READY`, the
vehicle camera's switch (`docs/screenshots/small-boats/README.md` counted
28 before this branch's later work); not looked into here.
