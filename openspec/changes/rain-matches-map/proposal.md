# Proposal: the rain on the map is the rain you stand in

## Why

The owner, 2026-11-16, with the map and the world at the same paused moment:
"there shouldnt be rain here, theres no rain on the map". The player stood at
map 26.562S 14.111W, west of Durnacrag, at night, in falling rain with drops
on the lens; the map's rain layer had no blue there, the nearest blue blobs
being south-east of Durnacrag and by Holbrook.

## What is measured first

Before any behaviour changes, one measurement instrument (an `#[ignore]`d
test in `crates/pbd-app/src/atmosphere.rs`, `map_rain_against_local`): on
the shipped settled climate, at 3,000 places through ten minutes of weather,
it reads

- the rain the player stands in (`weather::cloud_cell(..).rain`, which
  `weather::sample_field` sets as `Weather.rain` and which the streaks, the
  lens and the wet ground follow), and
- what the map draws there: the "Clouds and rain" layer's blue
  (`cloud_bytes`, `map_live.wgsl`) and the RAIN overlay's colour and alpha
  (`pbd_core::overlay`, `overlay_rgba`),

and counts the places where the player is in rain the map does not show.
The findings and the design are in `design.md`.
