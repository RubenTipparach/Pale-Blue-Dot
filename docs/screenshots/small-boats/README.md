# small-boats

Screenshots for `openspec/changes/small-boats`: the inner tube and the raft
in the game, the raft's hold in the pack, a cast from the raft's seat, and
the new world's Loon after the tide fix (design 2c).

How they were taken:
- in-game captures, on 2026-11-16, in a cloud container on lavapipe with no
  GPU;
- at 1440 x 900, from the fast build (`--profile fast`), under `xvfb-run`;
- each in a new memory-only world, at `--time 11 --rain 0 --weather-at 7200`.

They show what the boats look like, not how smoothly they run. Frame cost
was not measured.

A capture boards with `--aboard`:
- **Tube and raft.** A new world has no tube or raft, so `--aboard tube|raft`
  launches a made one in open water at least 8 m from the world's Loon, the
  way a player's launch makes it. The log says "launched a Raft for the
  capture".
- **`--load-hold`** puts twelve logs in the raft's hold through the pack's
  own move and save line.
- **`--cast`** takes up the rod and casts once from the seat, as `--fish`
  does on shore.

The tube and the raft are M34 River Craft's models (design 2a).
`raft-beside-m34.jpg` and `tube-beside-m34.jpg` put each beside the
mockup's own tab (M34 rendered in Chromium; the game on lavapipe).

| Shot | What it shows | Launch line |
| --- | --- | --- |
| `raft-chase.png` | The raft from the chase view, M34's: five logs, three planks across, the crate, the lantern post, the poler standing with the pole upright. The logs are a little over half under. | `pbd-app --capture raft-chase.png --aboard raft --frames 200 --time 11 --rain 0 --weather-at 7200` |
| `tube-chase.png` | The inner tube, M34's: the felted ring with its pitched seam, the lantern pole, the paddler sitting in it with arms out. Freeboard 0.24 m. | `... --aboard tube ...` |
| `loon-chase.png` | The new world's Loon at anchor, dry: freeboard 0.23 m, 0 kg aboard. Before the tide fix the same berth showed it swamped, with 700 kg aboard, heeled 60 degrees and freeboard -1.07 m (the trace in design 2c). | `... --aboard loon ...` |
| `raft-hold.png` | The pack opened aboard the raft. The HOLD row stands under it holding the twelve logs, and the boat's readout is hidden while the pack is open. | `... --aboard raft --load-hold --menu pack ...` |
| `bench.png` | The workbench panel: seventeen recipes, the last three the inner tube, the raft and the rowboat. | `... --craft bench ...` |
| `raft-cast.png` | The seat view aboard the raft with the rod in hand, after one cast. The status line reads "Watch the float - right click winds in". The log follows the line from Ready to Charging to Floating, with the float on the water at +0.3 m (the tide's surface); the float is too small to make out at this distance. | `... --aboard raft --seat --cast --frames 300 ...` |

Each `--aboard` capture logs 28 `LATE_PIPELINE` lines after `READY`. The
switch to the vehicle camera causes them, not the boats: `--aboard loon`
logs the same 28 on the same build (design 8).
