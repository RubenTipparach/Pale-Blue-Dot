# Proposal: the storm descent's four spikes

## Why

`lighting-perf`'s checkpoint (`openspec/changes/lighting-perf/design.md`,
"The checkpoint") found four frames over 30 ms that recur at the same route
metres in every run of every build on the `storm` descent: 3544 m (42-44 ms,
110 m up), 3808 m (61-65 ms, 58 m up), about 4122 m (30-32 ms, 36 m up) and
touchdown at 4223 m (about 36 ms). A frame that long is a visible hitch at
the moment the player is landing.

## What

1. **Name the work** in each spike (design, "Measured").
2. **Stop the physics step waiting on the async pool.** Avian optimizes its
   collider trees in a task on the pool the game's fine-set builds and town
   cuts fill, and blocks on it in the solver; it optimizes them in place
   instead (design, "Decision").
3. **Show it**: an A/B on `storm`, and captures that the look is unchanged.

## Not in this change

- Standing a town costs 3-14 ms of `Update` (`STAND_TIMING`); most of it is
  the casters' chunking (`crab-breeze-lessons` rank 5), which could move to
  the town's cut task. Not where the spikes are.
