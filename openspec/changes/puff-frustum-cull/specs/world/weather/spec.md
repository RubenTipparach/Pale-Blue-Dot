## ADDED Requirements

### Requirement: Puffs outside the view are not drawn
A cloud puff SHALL be culled when its bounding box, which holds its billboard
in every orientation, lies wholly outside the view frustum. A puff whose
bounding box touches or crosses the frustum SHALL NOT be culled.

#### Scenario: A puff behind the camera
- **WHEN** a puff's bounding box lies wholly behind the eye
- **THEN** it is not drawn

#### Scenario: A puff at the edge of the screen
- **WHEN** a puff's centre is off screen but its bounding box crosses the frustum
- **THEN** it is drawn
