## Why

Owner, 2026-11-27, fishing in the open sea on the release build: "its taking
so long to fish in the ocean", then "can you tweak it so a fish spawns every
10 seconds depending on what kind of fish lives there?" and "it should be
random between 10 and 30".

A bite today needs a school that happened to spawn 20-60 m from the angler
(up to 8 of them, anywhere round) to wander within the lure's 3 m scent reach.
In open sea, with the schools spread over the whole ring, that can take minutes.

## What Changes

- While a float rests on the water, every 10-30 s (a fresh random draw each
  time) one school of a fish that lives at the float's spot spawns 4-8 m from
  the float, and comes to the lure by the existing approach and bite rules.
- Which fish comes is the same rule the ring uses: the water class,
  temperature, depth and sea place at that spot. Water where nothing lives
  stays empty.
- The float's school is spawned even when the ring's caps are full, so a
  crowded ring never starves the float.
- The timer starts again on every new cast. A bite, catch or reel-in stops it.
- New tunables in `fauna.ron` `spawn`: `float_s: (10.0, 30.0)` and
  `float_m: (4.0, 8.0)`.

## Impact

- `crates/pbd-core/src/fauna/mod.rs` (`SpawnSettings`, its validation)
- `crates/pbd-app/src/fish.rs` (`Fishery`, `try_spawn`)
- `assets/config/fauna.ron`
- Nothing changes for a world's save. The bite odds are unchanged.
