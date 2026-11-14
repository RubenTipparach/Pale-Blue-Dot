# Proposal: a new world is visibly, and actually, a new world

## Why

The owner, 2026-11-14, playing the game normally: "when doing new world, it
looks like my position didnt change, I dont really know if my game even
really loaded".

Measured (`docs/screenshots/new-world-arrival/before.log`, the fast build on
the M4 Max Mac, with the measurement instrument `--new-world <name>` and the
`WORLD_SWITCH` log line added for it): a world `nw-old-a1` was made at
28.3 S 1.5 E (away from the default spawn) with three blocks dug, reopened,
and a new world `nw-new-a1` made from inside it a third of the way to the
shot, as the saves page's NEW WORLD does.

```text
WORLD_SWITCH before world 'nw-old-a1' ... edits 3 saved_pose 4279.3,2305.0,112.1
  walker 4279.3,2305.0,112.1 (lat 28.3000 lon 1.5000) spawn_dir (0.8802, 0.4741, 0.0230) clock_s 1440
world 'nw-new-a1' loaded: 0 edits across 0 cells
WORLD_SWITCH after  world 'nw-new-a1' ... edits 0 saved_pose none
  walker 4279.3,2305.0,112.1 (lat 28.3000 lon 1.5000) spawn_dir (0.8802, 0.4741, 0.0230) clock_s 1440
```

What happens today:

1. **A new save IS made and opened.** `saves::create` writes its identity and
   world file, and `load_world` opens it: 0 edits, no pose, the new world's
   kit dealt. The old world's edits are not carried into the new save.
2. **The player does not move.** The walker is put at
   `WalkingState::spawn_direction`, which for a world opened with a saved
   pose is *that pose* (`setup_walking` sets it to the restored `up`). So a
   new world made from inside a saved world opens exactly where the player
   was standing in the old one, to the centimetre: `lat 28.3000 lon 1.5000`
   before and after. The reset key has the same flaw. A plain launch opens the
   most recent world behind the saves page, so this is also what NEW WORLD on
   the front door does whenever a world exists.
3. **There is no loading screen.** The switch happens inside one frame
   (`load_world`, ~0.9 s here, a hitch); the old world's planet keeps
   standing, and the new edits arrive when the next fine set lands, dissolving
   the old world's holes away a second or two later. Nothing says a world was
   made or opened: no loading screen, no name. `loading-screen` left this as
   task 5.1.
4. **The old world's clock is carried.** "One never played keeps the clock it
   had", so the new world opens at the old world's hour and season
   (`clock_s 1440` before and after).
5. **The old world's last seconds may be lost.** The pose is written on a
   five-second timer and when a menu opens; `load_world` does not write the
   leaving world's pose or weather itself.
6. **The new world's towns are never stored in-session.** The sites list is
   re-read for the new save by `keep_sites`, but every world shares one seed,
   so the list's ids are the same and `build_towns` keeps the old world's
   town state standing; the new save gets its towns only on its next launch.

## What

- **A world opened from the saves page goes through the loading screen**, the
  same build-once path as a launch (`loading-screen` 5.1): the leaving world is
  written (pose, clock, weather, craft) and drained; the screen comes up; the
  new save is opened, its sites and towns read or made and stored, their
  ground installed and the planet built round where the player will stand, on
  the loading thread; the world then warms behind the screen and lifts at
  `READY`.
- **A new world starts at the new-world spawn**, the place a fresh launch
  starts a new world (the launch's spawn direction and its level-ground
  search), never where the last world's player stood.
- **A new world starts on the new-world clock** (day 0 at the start hour, or
  the launch's `--time`/`--day`), not the last world's.
- **Its name is shown on arrival**: a banner with the world's name and "A NEW
  WORLD" or "WELCOME BACK" for a few seconds after `READY`, on every open.

## Saved games

Nothing on disk changes shape. No save format, no generator, no record kind
is touched. Opening a world reads it exactly as today; the only differences
are that the leaving world is written before it is left (as the exit path
does), and that a world with no pose is placed at the new-world spawn. A
saved world keeps its pose. The world is never reset or relaunched: the
switch is in place, behind the screen.

## Not in this change

- **A seed per world.** Every world is the one planet (`TerrainConfig::
  TENEBRIS.seed`); a save of another seed is refused. So every new world
  starts at the same spawn, by design today. A per-world seed is a question
  for the owner with screenshots (recommendation: keep one planet; it is what
  the sites, towns and the map were approved on).
