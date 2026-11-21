# Proposal: logging, by raft and by river

## Why

The owner, 2026-11-18: "add logging system to the game (in phase 3). you
should either be able to load logs on a raft, or drop logs on to rivers, and
the log flows down river", and "logs stop flowing when the water stops
flowing".

## What Changes

- Logs are loaded onto a raft (its hold, `small-boats` design 5) and poled or
  carried downstream with it.
- A log dropped into a river floats as a body on the water and is carried by
  the water's own flow, the flow field the river and the flowing water
  already have (`water-flow`, `river-cache`).
- A log comes to rest where the water stops flowing: in still water (a lake,
  a pool, the sea), against a bank, or where it grounds in the shallows. It
  stays there until picked up.
- Drawn from M34 River Craft's log drive tab (rollway, logs in the current,
  a boom to catch them).

## Impact

- Phase 3 of the roadmap (after environment v2). Not started: built when the
  owner asks.
- Floating logs are world state: each is a stored record with its position,
  saved on the durable path; a log never vanishes.

## Started (owner, 2026-11-21)

**The owner, 2026-11-21: "People approved. Build.phase 3. Do it in parallel
if you can".** Logging is a step of phase 3 (CLAUDE.md, "The first game
loop"), so that is taken as the start this change waited for (task 1.1).
M34 River Craft (`docs/mockups/river-craft.html`) is approved for the game
(owner, 2026-11-19, "Approved. Build rest of phase 2"); its Log drive tab is
the reference.
