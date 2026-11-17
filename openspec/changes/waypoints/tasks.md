## 1. Model and save

- [x] 1.1 Up to four waypoints, the fifth dropping the oldest; a click on a
  pin removes it (built in `starter-missions` G3, revised 2026-11-16).
- [x] 1.2 Each waypoint carries its number, the lowest free one from 1, saved
  in the `map-mark` record at schema 3; schema 1 and 2 bodies read and are
  numbered in order. Verify: unit tests in `pbd_core::map` and the record
  round trip in `map_screen`.
- [x] 1.3 `--map-mark-east` also places waypoints in play, once the walker
  is out.

## 2. Map

- [x] 2.1 Right-click adds, a click on a pin removes (built in
  `starter-missions` G3).
- [x] 2.2 A "W1" ... label beside each pin, in the kit font with an ink
  outline. Verify: a capture with labelled pins.

## 3. Compass

- [x] 3.1 `MarkKind::Waypoint(i)` from `Place::seen`, every waypoint at every
  distance. Verify: a unit test of the mark's bearing at a distance past a
  town's reach.
- [x] 3.2 The red pins on the bar's foot row over the town studs, and the
  label line "W2 1.2 km" for the waypoint faced, ahead of a town. Verify: a
  unit test of the label rule and a capture facing a pin.

## 4. Close

- [x] 4.1 Shots in `docs/screenshots/compass-waypoints/`, `openspec validate
  --all`.
- [ ] 4.2 The owner plays it; then the spec moves into
  `openspec/specs/player/navigation/` and the change is archived.
