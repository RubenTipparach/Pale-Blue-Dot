# Simulation Clock Specification

## Purpose
One clock, asked in one place. Everything that integrates reads the same fixed
step, so a slow frame cannot move one subsystem further than another.

## Requirements

### Requirement: Acceleration is integrated exactly once
A constant acceleration SHALL be applied to the physics integrator exactly once
per fixed tick. A system SHALL NOT add its own copy of a force the integrator
already applies.

#### Scenario: A constant acceleration over many ticks
- **WHEN** a constant acceleration is applied for a known number of ticks
- **THEN** the resulting velocity matches a single exact integration

### Requirement: Time scaling keeps every clock together
Scaling time, stopping time and changing the fixed timestep SHALL move every
subsystem's clock together. A subsystem SHALL NOT integrate on the frame's own
delta while others use the fixed step.

#### Scenario: Scaling, stopping and resizing the step
- **WHEN** the time scale changes, time is stopped, or the timestep is resized
- **THEN** all clocks stay consistent with one another
