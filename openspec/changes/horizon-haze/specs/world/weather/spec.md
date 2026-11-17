# Weather: the sky behind distant cloud

## ADDED Requirements

### Requirement: The sky is lit air up to the shell's top

The sky's air SHALL be deep enough inside its shell that, seen from inside
the cloud layer, the sky just over the horizon is lit air, so a distant cloud
fading into it fades into the sky's colour. The air SHALL still end at the
shell's top, and the sky straight up from the ground SHALL keep its colour.

#### Scenario: Looking at the far cloud deck from inside the cloud layer

- **WHEN** the eye is inside the cloud layer and looks toward the horizon over broken cloud
- **THEN** the far deck fades into a blue sky, not into a near-black one under a grey film

#### Scenario: The sky from the ground

- **WHEN** the eye is on the ground at noon under a clear sky
- **THEN** the sky straight up is the colour it was before the change

### Requirement: The sea meets the sky without a seam

From inside the atmosphere, the sky SHALL be drawn in the colour of the air
just over the horizon wherever a ray under the horizon is left uncovered by
the sea or the ground, so that the sea's far edge meets the sky in the sky's
own haze at every tide, eye height, time of day and cloud cover, with no
dark band between them.

#### Scenario: Low over open sea at low tide

- **WHEN** the eye is 1.7 m over the sea at low tide and looks out to the horizon
- **THEN** the rows between the sea's far edge and the sky are the sky's horizon colour, not a band darker than the sky over it and the sea under it

#### Scenario: At dusk and under cloud

- **WHEN** the same view is taken at dusk or under full cloud
- **THEN** the sea meets the sky without a dark line, from 1.7, 10 and 100 m
