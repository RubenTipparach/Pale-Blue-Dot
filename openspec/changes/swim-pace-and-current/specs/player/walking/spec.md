## ADDED Requirements

### Requirement: A swimmer is half as fast as before
A walker swimming (eyes under and not wading, or in the water with its feet
off the bottom) SHALL move at `swim_speed_mult` (0.5, `swimming.ron`) of the
speed it had in the water before, at the surface, under it and sprinting.
A wader SHALL keep its speed
(`walking::swim_pace_tests::a_swimmer_goes_half_as_fast_and_a_wader_does_not`).

#### Scenario: Swimming ahead
- **WHEN** a walker swims ahead in still deep water
- **THEN** it makes half the speed it made before this change

### Requirement: The water's current pushes a swimmer whose feet are off the bottom
While a walker is in the water with its feet off the bottom, it SHALL be
carried toward `current_share` (0.5) of the water's velocity there, capped
at `current_max_mps` (1 m/s), easing at `current_rate_per_s` (1.5 /s). With
its feet down it SHALL NOT be pushed, and its carried way SHALL ease to zero
(`walking::swim_pace_tests::a_current_carries_a_swimmer_downstream_and_not_one_standing`,
`walking::swim_pace_tests::the_push_eases_without_a_jolt`).

#### Scenario: Floating in a river
- **WHEN** a swimmer floats with no input in a river flowing at 1.2 m/s
- **THEN** it drifts downstream, along the flow, at 0.6 m/s within a few
  seconds

#### Scenario: Standing on the bed
- **WHEN** a walker stands on the river's bed
- **THEN** the current does not move it
