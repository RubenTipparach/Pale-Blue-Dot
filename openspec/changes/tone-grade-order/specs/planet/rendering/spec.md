## ADDED Requirements

### Requirement: Turning the view leaves no trail
Each frame SHALL be drawn from a cleared target: nothing of an earlier frame
SHALL show through the sky or any other translucent surface. The display
transform SHALL run once a frame, after tonemapping and before the end of the
main pass's post-processing, however many cameras carry it.

#### Scenario: Turning under a clear night sky
- **WHEN** the walker looks up at a clear night sky with trees in view and turns 45 degrees a second
- **THEN** the stars are points and the trees' edges are sharp against the sky, with no fading copies behind either

#### Scenario: Several cameras carry the grade
- **WHEN** the walking, flight and vehicle cameras all carry the tone grade
- **THEN** the grade's node runs after tonemapping in the 3D graph
