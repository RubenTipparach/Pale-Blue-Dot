# Rendering: the sun seen from space

## ADDED Requirements

### Requirement: The sun is drawn from anywhere

The sun's disc and its glow SHALL be drawn in the sun's direction from any eye
position, inside or outside the atmosphere shell, wherever nothing nearer
covers it. Its outline SHALL NOT depend on the atmosphere shell's silhouette.
The planet and the moon SHALL hide it when they stand in front of it, and the
clouds SHALL hide it as they do from the ground.

#### Scenario: Sun above the shell's rim from orbit

- **WHEN** the eye is 6 km above the sea and the sun stands above the atmosphere shell's rim as seen from there
- **THEN** the whole disc and its glow are drawn against space

#### Scenario: Sun across the shell's rim

- **WHEN** the eye is 6 km above the sea and the sun stands on the atmosphere shell's rim
- **THEN** the disc is round, not cut along the rim

#### Scenario: Sun behind the planet

- **WHEN** the sun stands below the planet's limb as seen from orbit
- **THEN** no part of the disc is drawn over the planet

#### Scenario: Leaving the shell

- **WHEN** the eye climbs from 900 m to 1050 m above the sea, past the shell's top at 960 m, with the sun 5 degrees under the horizontal
- **THEN** the sun is drawn at both heights

### Requirement: The sun from space is seen through the air

From outside the atmosphere shell, the sun SHALL be dimmed and reddened by the
transmittance of the air along the ray toward it, with the same density and
extinction the atmosphere dome uses. The attenuation SHALL come in
continuously as the eye climbs past the shell's top, so the sun does not
change in one frame. From inside the shell the sun SHALL look as it did before
this requirement.

#### Scenario: Sun through the limb

- **WHEN** from orbit the sun is seen through the air just above the planet's limb
- **THEN** it is dimmer and redder than the sun seen against space

#### Scenario: From the ground

- **WHEN** the eye stands on the ground at noon or at sunset
- **THEN** the sun is drawn as it was before this requirement
