# Mission playthrough: every objective completable

## Why

The owner, 2026-11-19: "Do release build and start play testing by moving
down the tech tree. Record your findings as one long video of doing the
missions. I want to make sure all objectives are completable in missions".

Two checks were made on `claude/river-downhill` with `origin/main` merged
(`0bf02efc`), on the `--release` build:

- **A code audit** of every objective in `assets/config/missions.ron`:
  which system raises its event, and whether its inputs exist in the world.
- **A playthrough**: the showcase bot (`--showcase`, `step-videos` 2.1)
  plays the phase-1 branches in order on one world through the player's own
  inputs, and every stall is read off its log.

One objective cannot be completed in play, and the playthrough's bot had
three faults of its own. The details and the plan are in `design.md`.

## What changes

- **Coal counts when it is dug.** A dug ore cell that gives coal counts as
  gathering coal, as a dug stone block counts as gathering stone. Today
  nothing raises it, and the mining branch stops at "Find ore"
  (`ore` → `forge`, `smelt`, `bronze-tool`, `iron`).
- **"See a reef" can be met after "Into the water".** The reef is noticed
  each time the swimmer comes onto one, not once a session.
- **Two help lines say what the game does.** "A mixed field" and "The iron
  age" tell the player where the seeds and the iron really come from.
- **The showcase bot** (an instrument): a wild plant F offers nothing for
  is given up, the farming shots plant and harvest the six the missions
  ask for, and the full bucket is picked before it is poured.
- **New shot lists** for the missions no showcase played yet, and one long
  recording of the bot playing down the tree.

## Not changing

- Mission counts and timings (six game days for wheat, an egg a day). A
  player waits for them; the recording speeds the waits up, captioned.
- Chili's climate: it stays a hot-country crop. The help line says so.
