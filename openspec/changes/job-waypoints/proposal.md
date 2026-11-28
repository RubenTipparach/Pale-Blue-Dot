## Why

Owner, 2026-11-27, starting the phase 3 playtest: "if you can generate
waypoints for missions that require locations or to find a missing person,
or the job board requiring to deliver something, these waypoints would help
the UX greatly (they might already be in the game, if they are just
playtest, if they arent fix/add feature)".

Already marked on the compass bar and the map:
- a lost villager;
- an escort job's trader;
- a picked-up parcel's town;
- a building site's next piece;
- the next cell to pave.

Two taken jobs send the player to a place with no mark at all:
- a **repair**: a broken wall somewhere in the job's town;
- a **rockfall**: rock across a road between towns.

## What Changes

- A taken, unfinished repair job marks its broken wall's gap on the compass
  bar and the map, named "Repair". It is marked while the town is loaded,
  since the gap is read off the town's chart.
- A taken, unfinished rockfall job marks the middle of its fallen rock
  (the first uncleared cell) on both, named "Rockfall".
- Both marks go after the parcels and before the site's tick, two pins each.

## Impact

- `crates/pbd-app/src/economy/special.rs` (`repair_directions`),
  `economy/road_jobs.rs` (`rockfall_directions`)
- `desktop/compass_bar.rs`, `desktop/map_screen.rs`
- `assets/locale/locale_en.yaml`: `jobs.repair_marker`, `jobs.rockfall_marker`
