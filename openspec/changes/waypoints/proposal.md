# waypoints: marks the player places on the map, shown on the compass

## Why

The owner, 2026-11-14: "need waypoint system on map, and it should show on
compas". The map (`world-map`) shows the player, the craft and the towns, and
the compass bar (`compass-bar`) shows the towns in reach. Neither lets the
player mark a place of their own and walk or fly to it. `world-map` listed
"markers the player places, and waypoints" as a non-goal, to come later; this
is that change.

## What Changes

- **Place a waypoint on the map.** Right-click a point on the open map
  (left-drag already pans it) to put a waypoint there. Right-click a waypoint
  to remove it. Up to eight waypoints are kept, each numbered with its own
  colour; a ninth is refused with a line on the map, not dropped silently.
- **The map draws them** as numbered pins over the terrain, beside the
  player's arrow and the craft, and the readout names the one under the
  pointer with its distance from the player.
- **The compass shows them.** Every waypoint is a pin on the compass bar at its
  bearing, in its colour, whatever its distance (towns show only the nearest
  in reach). The waypoint nearest the bar's middle is named under it with its
  distance, as a town is, and a waypoint takes the label over a town.
- **They are saved with the world.** Each waypoint is a stored record
  (`world-persistence` decision 11) of a new kind, `waypoint`, written through
  the durable path when it is placed or removed. A save from before this
  change opens with none. An older build that opens a newer save keeps the
  records as it found them (records of an unknown kind are kept as text).
- **An instrument for the shots:** `--waypoint LAT LON` places one at launch
  (in `--at`'s frame), so a `--capture` can show the pin on the map and the
  compass.

## Capabilities

### New Capabilities
- `player/navigation`: waypoints, placed and removed on the map, drawn on the
  map and the compass, and saved with the world.

### Modified Capabilities
- None. The compass and map requirements are still in their changes
  (`compass-bar`, `world-map`); the waypoint behaviour is specified here.

## Impact

- `crates/pbd-app/src/compass.rs`: a `MarkKind::Waypoint(usize)` beside
  `Place`, sharing `Place::seen` for the bearing and distance.
- `crates/pbd-app/src/desktop/compass_bar.rs`: pins on the foot row and the
  label rule.
- `crates/pbd-app/src/desktop/map_screen.rs`: right-click to place or remove,
  the pins, the readout.
- `crates/pbd-app/src/saves/`: the `waypoint` record kind, placed and removed
  through the journal.
- A new `Waypoints` resource in `pbd-app`, loaded from the save.
- No change to generation, terrain or the save format of existing lines.
