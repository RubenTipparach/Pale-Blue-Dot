# waypoints: marks the player places on the map, shown on the compass

## Why

The owner, 2026-11-14: "need waypoint system on map, and it should show on
compas". The map (`world-map`) shows the player, the craft and the towns, and
the compass bar (`compass-bar`) shows the towns in reach. `world-map` listed
"markers the player places, and waypoints" as a non-goal, to come later; this
is that change.

The map half was built first, inside `starter-missions` (G3, revised on the
owner's 2026-11-16 "up to 4 waypoints allowed, the next one despawns the
first, you can click on a waypoint to get rid of it anytime"): a right click
on the map sets a waypoint, up to four, saved as the `map-mark` record. The
compass half was not. The owner, 2026-11-17, on a pin set on a coast:
"waypoint not showing in compas, next to it should be w1,w2.... and on the
compas when facing the waypoint I should see that text".

What was missing on 2026-11-17, measured against the build:
- the compass bar draws towns only; a waypoint is not on it at all;
- the map's pins carry no label, so two pins cannot be told apart;
- a waypoint has no number of its own: the record is a bare list, so a
  number taken from its place in the list would shift every time the oldest
  is dropped or one is removed.

## What Changes

- **Waypoints are numbered W1, W2, W3, W4.** A new waypoint takes the lowest
  number not in use, so numbers go up in the order placed and a waypoint
  keeps its number until it is removed. The number is saved with it.
- **The map labels each pin** with its number, beside the pin, in the kit's
  pixel font with an ink outline.
- **The compass shows every waypoint** as a small red pin on the bar's foot
  row at its bearing, whatever its distance (towns show only the nearest in
  reach). Facing one (within the bar's naming window of the middle), the line
  under the bar reads its label and distance, "W2 1.2 km", ahead of a town.
- **Saving**: the `map-mark` record moves to schema 3, each waypoint carrying
  its number. A schema 1 or 2 body (no numbers) reads as before and is
  numbered by its order, oldest W1. Nothing else in the save changes.
- **The capture instrument** `--map-mark-east` also works in play, not only
  with the map open, so a capture can face a waypoint on the compass.

## Capabilities

### New Capabilities
- `player/navigation`: waypoints, placed and removed on the map, drawn on the
  map and the compass with their numbers, and saved with the world.

### Modified Capabilities
- None. The compass and map requirements are still in their changes
  (`compass-bar`, `world-map`); the waypoint behaviour is specified here.

## Impact

- `crates/pbd-core/src/map.rs`: `Mark::number`, the lowest free number in
  `add_mark`, numbering in `read_marks`, schema 3.
- `crates/pbd-app/src/compass.rs`: `MarkKind::Waypoint`, the waypoint marks
  and the label rule.
- `crates/pbd-app/src/desktop/compass_bar.rs`: the pins on the foot row and
  the label line.
- `crates/pbd-app/src/desktop/map_screen.rs`: the label beside each pin; the
  instrument in play.
- `crates/pbd-app/src/ui_kit.rs`: the waypoint red as a palette token.
- No change to generation or terrain. Old saves open with their waypoints.
