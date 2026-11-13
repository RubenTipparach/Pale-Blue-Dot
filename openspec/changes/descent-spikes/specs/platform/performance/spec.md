# Performance: the physics step

## ADDED Requirements

### Requirement: The physics step takes no longer when the world changes round it

The game's physics step SHALL take no longer on a frame when a fine set
lands or a town stands than on the frames around it.

#### Scenario: A fine set lands during the storm descent

- **WHEN** the `storm` route descends and a fine set lands, or a town
  stands
- **THEN** that frame's fixed step takes no longer than on the frames
  around it
