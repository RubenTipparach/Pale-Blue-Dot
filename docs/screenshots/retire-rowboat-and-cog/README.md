# retire-rowboat-and-cog

Screenshots for `openspec/changes/retire-rowboat-and-cog` (task 3.2): a new
world's harbour before and after the rowboats and the cog are retired, an
old world (made by the build before) opened by the new one with a made
rowboat and a rowboat good refunded, and a player saved aboard the made
rowboat put ashore.

How they were taken, 2026-11-18:
- on the M4 Max Mac's GPU (Metal), from the fast build (`--profile fast`),
  2880 x 1800, saved here at 1440 px wide;
- "before" is `origin/main` at `05d1bec2`; "after" is this branch;
- every shot with `--time 12 --rain 0 --weather-at 7200 --frames 300` (the
  harbour's local time is night at that clock);
- the old world is a throwaway one made by the before build (`--walk
  --world rrc-old --at 37.07 171.25 --frames 600`, then `--aboard raft` for a
  made raft), beside the harbour Marenstrand: 82 craft, 68 harbour rowboats
  and 6 cogs among them. In its copies the made raft's `kind` was edited to
  `rowboat` (the build before made rowboats only at a workbench) and a pack
  line holding a rowboat good and 3 planks was appended to its log. Each
  shot opens its own copy; all were deleted afterwards.

| Shot | What it shows |
| --- | --- |
| `new-world-before.jpg` / `new-world-after.jpg` | A new memory-only world at Marenstrand (`--walk --at 36.617 171.603 --yaw 200 --pitch -7`). Before: a harbour rowboat in the foreground and the cog moored at its gangplank behind the pier, its mast up. After: the water is empty; the piers, the light and the beached boats on the shore (dressing, not craft) stay. Logs: before `Marenstrand: 12 boats moored and its cog, 11 berths skipped`; after `Marenstrand: 1 boats moored, 22 berths skipped (over land, too shallow, or a retired boat's)`: its one canoe deep enough to float. |
| `refund-before.jpg` / `refund-after.jpg` | The old world as saved, the walker on land. Before: the hotbar holds the rowboat good and 3 planks. After: 35 planks and 4 rope, the made rowboat's refund (16 planks, 2 rope) and the good's (16, 2) on top of the 3. Log: `craft 82, a retired rowboat the player made: refunded ...`, `a retired boat good swapped for its materials`. Opened again, nothing more is given (no refund line; the pack line is the same). The vehicle file it writes still holds all 82 records, the 75 rowboat and cog records byte for byte. |
| `aboard-before.jpg` / `aboard-after.jpg` | The old world with the walker saved at the made rowboat's seat, as a player who quit aboard is. Before: loaded there, under the water at the seabed. After: `the player was aboard craft 82, a retired rowboat: put ashore`, standing on the beach (`PLACE spawn at 36.911 171.013`). |

`LATE_PIPELINE` lines after `READY` appear in these captures, before and
after alike: a `--capture` in fixed steps lifts the loading screen without
warming. Frame cost was not measured.
