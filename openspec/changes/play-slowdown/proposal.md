## Why

The owner's release playtest of 2026-11-28 on World 19: "figure out what it
is thats slwoing things down from 50-60 fps to a consistennt 20-30 fps with
not much changes in the scene complexity to accout for the consistent
slowdown", "I did walk along a river", then "recreate the issue and fix it".

The frame log of that run (`output/perf/play-20261128-0236.csv`, RTX 3070,
1440x900, release):

| Minutes in | main thread p50 | GPU p50 | fps |
| --- | ---: | ---: | ---: |
| 1-3 | 13 ms | 18 ms | ~55 |
| 6-7 | 18-19.5 ms | 14 ms | ~48 |
| 9-10 | 24-30 ms | 12 ms | 16-30 |

The process grew from 0.68 GB to 1.79 GB. Nothing the log names (the field
corners, the town bakes, the fine set landings, the tier image) adds up to
more than a few ms a frame by the end, so the growth is in a system the log
does not time. Along the river the game poured about 113,000 water cells
(`RIVER_FILL`, `SWAMP_FILL`) and the river band near the player doubled
(2,000 to 4,500 cells).

## What changes

1. **A measurement instrument** (CLAUDE.md, the instrument exception): a
   cargo feature `system-times` that turns on Bevy's per-system spans and
   has the existing span clock (`desktop/cpu_times.rs`) sum each system's
   main-thread time, logging the top systems every 30 s as `SYSTEM_TIMES`,
   with the entity count. Off in every normal build; nothing a player sees
   changes.
2. **The repro**: `--walk-river right --at 19.25 -6.75` (the `river-cache`
   walk) in release with the instrument, 12 minutes, on a throwaway world,
   comparing each system's time minute by minute.
3. **The fix**, written into `design.md` once the system is named, then made
   and measured again on the same walk.
