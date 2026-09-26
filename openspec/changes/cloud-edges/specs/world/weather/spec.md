# Weather: cloud edges and entering a cloud

## ADDED Requirements

### Requirement: A cloud is hazed to its edge

Every march texel that holds cloud SHALL carry the distance of that cloud
along its ray, whether the cloud came from this frame's march or from the
history, and the composite SHALL NOT measure a cloud's haze to a distance of
zero unless the eye is inside the cloud layer.

#### Scenario: A cloud near the limb from high up

- **WHEN** a cloud lies near the planet's limb seen from 1.5 km up or higher
- **THEN** its edge is hazed like its inside, with no bright rim of unhazed texels

### Requirement: Entering a cloud keeps its history

The clouds' history SHALL be refused only where the previous frame's march did
not reach the cloud being read, or held cloud beyond this ray's stop; not for
the parallax of the ground behind a near cloud. A texel with no cloud this
frame SHALL read its history at the distance of the cloud that history holds.

#### Scenario: Flying into a cloud

- **WHEN** the camera flies into the cloud layer over land or sea
- **THEN** the frame does not fill with single-sample speckle, and no cloud is dragged across the screen in curtains
