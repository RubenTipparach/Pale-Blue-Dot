# Weather: the clouds' history is held to what this frame shows

## ADDED Requirements

### Requirement: The cloud history is clipped to the current neighbourhood

The clouds' reprojected history SHALL be limited to the range of the current
frame's cloud in the texel's 3 x 3 neighbourhood before it is blended, after
the silhouette test of `cloud-ghosting`, so a history that passed that test
but no longer matches the cloud (a bad reprojection inside a cloud, a cloud
that changed) cannot persist.

#### Scenario: Flying into a cloud

- **WHEN** the camera flies into and through cloud
- **THEN** no offset copies of the cloud pile up behind the motion
