## 1. Model and save

- [ ] 1.1 A `Waypoints` resource:
  - waypoints as body directions with numbers 1 to 8;
  - place takes the lowest free number and refuses past eight;
  - remove by number.

  Verify: unit tests for the lowest free number and the cap.
- [ ] 1.2 The `waypoint` record kind (schema 1) in the save:
  - written on place and removed on remove, through the durable path,
    authored by the player;
  - loaded into `Waypoints` on open.

  Verify:
  - a round-trip test: place two, remove one, reload;
  - an old-save test: it opens with none;
  - check how an older build treats the removal of an unknown kind (design
    risk 3).
- [ ] 1.3 `--waypoint LAT LON` (repeatable) places waypoints at launch.

## 2. Map

- [ ] 2.1 Right-click (and Ctrl+click) places a waypoint, or removes one
  within 12 px; a left-click or drag never places one; the cap message.
  Verify: a test on the steer logic and a real-time check.
- [ ] 2.2 Numbered pins in their colours. The readout names the pin under the
  pointer with its distance. Verify: a capture with three pins.

## 3. Compass

- [ ] 3.1 `MarkKind::Waypoint(i)` from `Place::seen`, every waypoint at every
  distance. Verify: a unit test of the mark's bearing and distance.
- [ ] 3.2 Pins on the bar's foot row over the town diamonds, and the label
  rule (nearest the middle, ahead of a town). Verify:
  - a capture facing a pin;
  - a capture with a pin and a town on one bearing.

## 4. Close

- [ ] 4.1 Before and after shots in `docs/screenshots/waypoints/`, the spec
  moved into `openspec/specs/player/navigation/`, and `openspec validate
  --all`.
