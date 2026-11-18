# The whole river cached: the river run (2026-11-18)

The owner's desktop: RTX 3070, 1440 x 900 window, `--profile fast`, nothing
else running. The write-up is `openspec/changes/river-cache/design.md`,
sections 11 to 13.

The walk is `--walk-river --at 19.25 -6.75 --time 11 --frame-graph` in a
new world each run. The walker is spawned in the owner's river at its source
and swims down it hex by hex through the player's inputs.

## The cache on and off, one sitting, about 75 s each

| | Cache on | `PBD_RIVER_CACHE=off` |
| --- | ---: | ---: |
| Fine set rebuild, mean | 3.6 s (16 sets) | 5.9 s (11 sets) |
| River lookups from the cache | 97% | - |
| Frame median / p99, past 35 s | 15.3 / 24.6 ms | 14.9 / 27.4 ms |
| Frames over 50 ms | 6 | 9 |

## The recording, cold then warm (with the bot's fixes)

`river-cache-runs-v2-2026-11-18-450p60.mp4` (Drive, pale-blue-dot-validation), 5 min 40 s:

| | Run 1, cold | Run 2, warm (run 1's file) |
| --- | ---: | ---: |
| Whole river, source to mouth | 170.5 s | 169.8 s |
| Stalls | 0 | 0 |
| Seconds on the bank | 0 | 0 |
| Lookups from the cache | 86.7% | 97.5% |
| Fine set rebuild, mean | 4.1 s (35) | 4.2 s (35) |

The first recording (`river-cache-runs-2026-11-18-450p60.mp4`) had the bot
before its fixes: 8 and 11 stalls, each freed. Design 12 says why.

Not measured: `tools/perf_suite.py` on the release build.
