# Flight: the far-side route

## ADDED Requirements

### Requirement: A route flies from the ground to the ground on the far side

The far-side route SHALL:
- take off from dry land;
- climb at its configured angle to a cruise height above the atmosphere;
- follow the great circle;
- descend and land within a configured distance of the destination, the
  antipode snapped to dry land.

Commanded acceleration SHALL be continuous across phase changes.

#### Scenario: A headless far-side route

- **WHEN** `--verify-route far-side` runs
- **THEN** it reports completion and a touchdown within the configured distance
  of the destination
- **AND** the clearance never falls below the protection minimum

### Requirement: The route's camera frames the planet and moves like a pilot

The far-side route's camera SHALL keep the planet in frame, with its limb
during the cruise and the landing site during the descent. It SHALL bank and
pitch with the flight, eased with no overshoot. Its peak angular speed and
acceleration SHALL stay within configured limits, and a reduced-motion setting
SHALL damp it.

#### Scenario: A headless far-side route reports its camera

- **WHEN** `--verify-route far-side` runs
- **THEN** it reports the camera's peak angular speed and acceleration, both
  within their configured limits

### Requirement: Fast flight holds a steady frame

Ground detail that cannot be resolved from the camera's height SHALL NOT be
rebuilt, in any flight mode. The far-side route SHALL hold its frame time
under a 60 Hz budget.

#### Scenario: The route on the reference machine

- **WHEN** the far-side route runs at 1440 x 900 on the reference machine
- **THEN** the 99.9th-percentile frame time from `--frame-log` is under 16.7 ms
