# Weather: clouds without ghosts, without columns, and fading into the haze

## ADDED Requirements

### Requirement: The cloud history never carries cloud across a silhouette

The clouds' temporal history SHALL be limited to the range of the current
frame's neighbourhood and dropped where the previous frame's view was
blocked, so an object moving in front of a cloud leaves no cloud in its shape.

#### Scenario: A tree line passes in front of a cloud

- **WHEN** the camera moves so that trees cross in front of a cloud
- **THEN** no cloud-coloured trail remains over the trees in the following frames

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
