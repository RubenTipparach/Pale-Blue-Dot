# rain-shells: the rain volume against streak shells

## Findings

- Apple M4 Max, 1440 x 900 window (2880 x 1800 Retina), release build, unlocked, 2 runs per variant, interleaved.
  `PBD_RAIN_RENDERER=volume|shells` on one build; `rain-off` is `PBD_PASS_OFF=rain`.
- **Near the ground (`clouds`, clearance below 200 m, landings left out), the shells cost a third to a half of the volume:**

| Sitting | Variant | p50 ms | p95 ms | Frames over 16.7 ms |
| --- | --- | --- | --- | --- |
| 1 (five shells) | volume | 17.7 | 34.6 | 64% |
| 1 | shells | 14.6 | 27.1 | 27% |
| 1 | rain off | 13.2 | 24.5 | 19% |
| 2 (six shells) | volume | 17.2 | 35.1 | 59% |
| 2 | shells | 15.7 | 28.5 | 38% |
| 2 | rain off | 14.3 | 26.5 | 31% |

  The volume costs 3-4.5 ms at p50 above rain off; the shells 1.4 ms. The sixth shell (1,500 m, added so a storm 1-2 km off is read)
  is within the spread between sittings. Sitting 2 ran about 1 ms slower in every variant, so it is compared within itself.
- **`walk` (sitting 1):** p50 9.1 (volume), 7.8 (shells), 8.3 (rain off); p95 17.0, 11.8, 11.2.
- Above 200 m the three are the same: nothing is drawn there differently.
- One 304 ms frame in `walk_shells_0`, at the walk script's `WALK_TURN stalled` line, with no landing. It is the script's turn,
  not the rain; noted for the CPU profile (`frame-budget` 2.8).

## Limits

- No storm 1-2 km off was found in the meadow view to compare the far veil; that capture is still owed.
- The near-ground frame is still over budget with rain off (p95 24-26 ms): rain was one cost of several.
