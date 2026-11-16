# sea-life-fixes: sea life that vanished up close, floes on the beach (`ocean-environments` B17)

The owner, 2026-11-16: "what the heck are these fish doing" and "they
disappear when I get close to them", on a cold coast with tipped, stacked
floes and a dark seal between them.

Fixed-step captures on the M4 Max Mac (Metal, GPU), `--profile fast`,
memory-only worlds, generator version 9. "Before" is `claude/env-v2` at
`1218e2a`; "after" is this branch. Looks only: frame time was NOT measured.

## The floes

- `shore-floes-before.jpg`, `shore-floes-after.jpg`,
  `shore-floes-before-after.jpg`: `--walk --at 86.965 -17.827 --time 12
  --frames 400 --pitch -10`. The walker stands on sand 0.6 m above the sea.
  Before: floes lie all over the beach, the terrain's columns standing up
  through them, so they read as slabs tipped and stacked (the owner's
  frame). After: a floe is laid only where its 0.6 m of ice clears the
  ground at low water, so the beach is bare and the floes float in the
  channel and the open water.

## The seals

- `seal-approach-before-after.jpg`: `--life seal:D --yaw 35`, D = 30, 20,
  12, 8 and 5 m (rows), before (left) and after (right), the same shore.
  `--life seal:3` and `seal:2` put the seal at 5 m too: no water nearer is
  deep enough for a seal at this shore, so the sequence ends at 5 m.
  Honest read: at this spot the seals are hard to pick out among the floes
  at this size; the frames show the floes moving off the beach more than
  the seal. The seal's behaviour is read off the trace below.
- `seal-approach-trace.txt`: the `PBD_LIFE_TRACE=1` instrument's log for the
  same runs (after): each frame, the `--life` seal's distance from the eye,
  how far under it is drawn (0 lying out, 1 wholly dived), its school's fade
  and its fright. At 30, 20 and 12 m it fades in over 0.8 s and lies out;
  at 8 and 5 m it fades in and dives over 1.2 s, the share rising a little
  every frame. Those five runs were taken before the flee was strengthened,
  and the seal stayed near 5 m; the last block, after it, has the seal dive
  and swim off from 5.6 m to 17 m in 6 s. Before this branch the same seal
  was drawn 1.2 m under the surface on its first frame within 9 m (seen from
  above the water, gone), and a floe's seal was skipped outright within 9 m.

## Fish and the other creatures

- `lineup-surface-after.jpg`: `--life lineup` from the 0.7 m shore at
  (73.858, 151.125), 12:00, pitch -18, 560 frames: every body still draws,
  the ducks on the water. (The lineup is a static instrument and puts a few
  bodies on the sand terrace beside the water.)
- The swimming fish were not vanishing: they have no shy rule (only a cast's
  splash scatters them, and they swim off), Bevy recomputes each school
  mesh's bounds every frame it changes, and they draw at 2 m in the lineup.
  What they did do was appear and go in one frame, spawned 20 to 60 m out
  and dropped at 90 m; they now fade in and out over 0.8 s.

READY with no new `LATE_PIPELINE` (25 lines before and after, the same list).
