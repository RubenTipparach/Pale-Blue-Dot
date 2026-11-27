## Why

The owner, 2026-11-26: "I started walking down a river, and the game
started out smooth, but quickly dipped in perf, we need to look into hitches
of when it is loading new section", and "maybe introduce regular playthrough
with ui and stuff".

**Measured** (release build, RTX 3070, 1 km bot walk from the owner's river
at `--at 19.25 -6.75`, `--frame-log`, nothing else running):

| | |
| --- | ---: |
| Frames while walking | 10,578 |
| p50 / p99 / max | 11.3 / 25.1 / 279 ms |
| Frames over 33 ms / over 50 ms | 54 / 16 |
| GPU p50 / p99 | 5.6 / 14.8 ms |

Steady walking holds 60 fps. The slow frames are the **landings**: a new
fine set lands every 4.7 to 5.8 s (about every 45 m on foot), and on and just
after it the main world spends 30 to 80 ms and the render world up to 90 ms.
Near the river each landing also builds the river band's water (up to 1,824
cells, 178 ms on its worker). The light bakes (bounce 22 to 33 ms, lamps
15 ms) run on the set's worker, not the main thread.

## What changes

1. **Find the main-thread cost of a landing**, system by system: a trace
   build (`--features trace`) on the same walk, the slowest frames' spans
   ranked. (Measurement instrument.)
2. **A regular playthrough as a scenario** (`assets/showcase/play-soak.ron`,
   `perf_suite.py` `play`): from a fresh world at the owner's river, the
   starter missions with the tracker up, the pack, the map, crafting and
   chopping, then on into a town, every panel opened by its key. It measures
   what the owner plays, not a stripped walk.
3. **The fix**, once 1 names the cost: spread or move off the main thread
   whatever a landing does at once, so a landing costs no frame over 16.7 ms.
   Nothing the player sees changes.

## Impact

`planet_lod.rs` (the landing), whatever the trace names; `tools/perf_suite.py`.
