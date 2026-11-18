# retire-tern

Screenshots for `openspec/changes/retire-tern` (task 3.2): which boat the
owner's screenshot shows, a new world's harbour and an old world's harbour
before and after the Tern is retired, a player saved aboard a Tern, and a
swamped rowboat (the capsize the owner asked about, which the boats that stay
can show too).

How they were taken, 2026-11-17:
- on the M4 Max Mac's GPU (Metal), from the fast build (`--profile fast`),
  2880 x 1800, saved here at 1440 px wide;
- "before" is `origin/main` at `2de3c03a`; "after" is this branch;
- every shot with `--time 23 --rain 0 --weather-at 7200` (11 at the start
  for the two craft shots) and `--frames 300` (200 for the craft);
- the old world is a throwaway one made by the before build
  (`--walk --world rb-old --at 37.07 171.25 --frames 600`), beside the
  harbour Marenstrand: 112 craft, 31 of them Terns. Each shot opens its own
  copy of it; only the walker's position, heading and pitch in the copy's
  `world.ron` were changed. All were deleted afterwards.

| Shot | What it shows |
| --- | --- |
| `craft-tern.jpg` | The Tern (`--aboard tern`, before): off-white hull, salmon deck strip, white sail. The only white hull the world spawns. |
| `craft-loon.jpg` | The Loon (`--aboard loon`, before): dark driftwood. Not the boat in the owner's screenshot. |
| `old-save-before.jpg` / `old-save-after.jpg` | The old world from the water at Marenstrand. Before: a harbour Tern, white hull and sail, among the brown rowboats. After, the same copy opened by the new build: the Tern is gone, the rowboats are where they were. Its log reads `craft 9 is a retired tern: kept in the save, not in the world` for each of the 31 Terns, and the file it writes still holds all 112 records. |
| `new-world-before.jpg` / `new-world-after.jpg` | A new memory-only world (`--walk --at 36.617 171.603 --yaw 200 --pitch -7`). Before: three harbour Terns, the rowboat, the cog. After: the rowboat and the cog only. Logs: before `Tern placed`, `Loon placed`, `Marenstrand: 17 boats moored and its cog, 6 berths skipped`; after `Loon placed` only, `Marenstrand: 12 boats moored and its cog, 11 berths skipped (over land, too shallow, or a retired sailing boat's)`. |
| `aboard-before.jpg` / `aboard-after.jpg` | The old world with the walker saved at a harbour Tern's seat, as a player who quit aboard is. Before: loaded at the seat, under the water beside the Tern. After: `the player was aboard craft 9, a retired tern: put ashore`, standing on the quay 1.6 m above the ground (`PLACE spawn at 36.953 171.720`). |
| `swamped-rowboat.jpg` | Not from this change: the same copy after 300 frames, looking at harbour rowboat 13 lying swamped under the water in the foreground. It took on 150 kg over its rim at its berth and heeled 27 to 34 degrees, the same in the before build (design, "Why it was capsized"). |

`LATE_PIPELINE` lines after `READY` appear in every one of these captures,
before and after alike: a `--capture` in fixed steps lifts the loading screen
without warming. Frame cost was not measured.
