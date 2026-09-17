# Water Specification

## Purpose
The sea is one sheet drawn by one shader from both sides of its surface, over
a seabed the terrain draws. Which side the camera is on is decided once, on
the CPU, and every water term reads that decision.

## Requirements

### Requirement: The camera's side of the surface is one of three states
The renderer SHALL classify every view as dry, straddling or under from the
camera's body-local radius against the sea radius and whether its direction is
over water. The straddling band SHALL be at least as wide as the wave
amplitude, so the wave function is not evaluated a second time on the CPU.

#### Scenario: Diving through the surface
- **WHEN** a camera descends from above the sea through the surface band
- **THEN** the state passes dry, straddling, under in that order
- **AND** an emerge window arms when it comes back up and expires 2.6 s later

#### Scenario: Over land at any radius
- **WHEN** the camera's direction is over a land cell
- **THEN** the state is dry whatever its radius
