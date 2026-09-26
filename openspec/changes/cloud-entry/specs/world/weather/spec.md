# Weather: flying into a cloud

## ADDED Requirements

### Requirement: The cloud history agrees with the frame

The clouds' history SHALL be clipped to the range of this frame's cloud in a
3x3 neighbourhood of march texels before it is blended, on top of the
silhouette test, so a reprojected history that disagrees with what this frame
sees is not carried.

#### Scenario: Passing through a thin cloud

- **WHEN** the camera flies through a thin cloud at flight speed
- **THEN** no stacked, offset copies of the cloud are drawn

#### Scenario: A cloud's side sweeps across the view

- **WHEN** the side of a cloud moves across the view as the camera flies past it
- **THEN** no smear of it is left behind for more than a frame or two

### Requirement: The near field of a cloud is its own fog

The first `2 x cloud_step_m` of a view ray inside the cloud layer SHALL be
drawn at full resolution from unjittered samples of the cloud's coarse
density, outside the history, so the fog thickens continuously as the eye
enters a cloud.

#### Scenario: Flying into a cloud

- **WHEN** the camera flies from clear air into a cloud
- **THEN** the view fades smoothly into the cloud's lit fog, with no speckle, bands or flicker
