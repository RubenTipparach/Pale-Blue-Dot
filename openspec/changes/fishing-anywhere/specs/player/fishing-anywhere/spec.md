# Player: fishing anywhere there is water

## Purpose

Where a line floats and fish swim: every water the world holds.

## ADDED Requirements

### Requirement: A cast floats on any water

A float SHALL float on the sea, a river, a lake or water a player poured, at that water's own surface, and SHALL find no water on dry ground.

#### Scenario: A cast into a river above the sea

- **WHEN** a player casts into a river whose water stands above the sea's level
- **THEN** the float floats on the river's surface and a fish there can bite

### Requirement: Smaller water holds smaller fish

A school in fresh water SHALL take a size from its body of water, and its fish SHALL be that much shorter. A school in the sea SHALL keep its species' size.

#### Scenario: A pond against a river

- **WHEN** the same species is caught in a small pond and in a wide river
- **THEN** the pond's fish is the shorter
