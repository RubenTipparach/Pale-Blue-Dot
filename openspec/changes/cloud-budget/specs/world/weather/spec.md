# Weather: clouds inside a frame budget

## ADDED Requirements

### Requirement: Rain streaks are fixed in the world

The distant rain's streak pattern SHALL be a function of the body-local point
and the clock only, never of the eye's position, so it falls at the rain's own
speed whether the eye is still or moving.

#### Scenario: Walking toward a shower

- **WHEN** the eye moves toward or away from falling rain
- **THEN** the streaks keep falling downward at the fall speed, and do not
  slide up or down with the eye's motion

### Requirement: The clouds' cost is bounded by data

The cloud march SHALL step by length from the eye, under a step cap, at a
resolution scale, and the step, cap and scale SHALL be validated values in
`weather.ron`.

#### Scenario: Inside a storm's cloud layer

- **WHEN** the camera is inside a full-cover cloud layer on the reference
  machine at the default settings
- **THEN** the frame holds 120 fps at the 95th percentile

### Requirement: No seam at the cloud base's horizon

Rays on either side of the cloud base's horizon SHALL sample the cloud at
matching step lengths near the eye, so that no curve is drawn across solid
cloud.

#### Scenario: Looking out from inside the layer

- **WHEN** the camera is inside a full cover and looks toward the horizon
- **THEN** no arc is visible across the cloud where rays change from leaving
  through the base to leaving through the side
