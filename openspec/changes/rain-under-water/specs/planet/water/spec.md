## ADDED Requirements

### Requirement: The rain volume is in the air, never in the water
The rain volume SHALL be gathered only along air. From a camera under the sea
the rain volume SHALL draw nothing and its passes SHALL not run: what the eye
sees of the air through the surface is the water sheet's. Above the sea the
rain SHALL stop at the sea surface and at the scene's depth.

#### Scenario: Under water in rain
- **WHEN** the camera is 5.4 m under the sea in rain with the rain volume on
- **THEN** the frame is the same as with the rain volume off

#### Scenario: Above water in rain
- **WHEN** the camera stands on the shore in rain with the rain volume on
- **THEN** the rain is drawn as before this change
