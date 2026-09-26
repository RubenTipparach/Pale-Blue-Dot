# Weather: clouds without ghosts, without columns, and fading into the haze

## ADDED Requirements

### Requirement: Cloud shape varies with height

The clouds' shape noise SHALL be a function of the point in three dimensions,
so a cloud seen from inside or from above has no columns running from its
base to its top.

#### Scenario: Looking down through broken cloud

- **WHEN** the camera is inside the cloud layer looking down past clouds
- **THEN** no vertical columns hang from the clouds to the base

### Requirement: Distant clouds fade into the haze

A cloud's contribution SHALL fade with the air between the eye and the cloud,
so clouds near the horizon dissolve into the sky rather than ending at a hard
edge, while clouds seen from above the atmosphere keep their contrast.

#### Scenario: Clouds at the horizon from the ground

- **WHEN** clouds lie many kilometres out near the horizon
- **THEN** they fade toward the colour behind them instead of standing opaque
