## Why

Owner, 2026-11-27, after the town playtest's bot sat in a flooded pit 442 m
from Holbrook for minutes: "improve bot scripts so it never gets stuck like
this ever". The log showed every existing escape firing in turn and failing:
- "held short ... planning round it";
- the 60 s watchdog's "up and out";
- a stair dug up;
- then thousands of "select step 0" frames, a tool swap that never landed.

Each escape so far was a patch for one trap (a pit, a lake, a trench, a tunnel's
trunk). A new trap always finds the gap between them.

## What Changes

- A last resort over all of them, checked every frame outside every act. If
  the bot has a point to reach more than 5 m off and has not covered 3 m in
  100 s, whatever it tried, the walker is put ashore 20 m toward the goal by
  the game's own `walking::put_ashore` rule (the start search's dry land).
- Every rescue is logged `BOT_RESCUE` with the time and the distance left, so
  each one is still a trap to fix properly, never hidden.
- It is a bot rule only. The player's game is untouched.

## Impact

`crates/pbd-app/src/desktop/showcase.rs`. A gameplay demo recorded for the
owner should show no `BOT_RESCUE`; a demo with one is retaken after the trap is
fixed, because a rescue sets state behind the player's back (CLAUDE.md, Demos).
