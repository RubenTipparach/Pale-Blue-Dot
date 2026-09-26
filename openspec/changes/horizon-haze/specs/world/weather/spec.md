# Weather: the air between the eye and a cloud

## ADDED Requirements

### Requirement: Distant cloud takes the colour of the air in front of it

The air between the eye and a cloud SHALL both dim the cloud and add its own
colour over the cloud's coverage. The ground, the water sheet, the clouds and
the sky SHALL haze with one air model (one density, one scale height, one
colour, with the sky's dusk tint), so that at the same distance they haze
alike.

#### Scenario: Looking at the far cloud deck from inside the cloud layer

- **WHEN** the eye is inside the cloud layer and looks toward the horizon over broken cloud
- **THEN** the far deck reads as pale haze of the sky's colour, not as a grey film over dark sky

#### Scenario: A cloud near the limb from over the tops

- **WHEN** a cloud lies near the planet's limb seen from 1.5 km up
- **THEN** it fades into the sky's horizon glow, with no step in colour where it meets the sky behind it

#### Scenario: A deck at dusk

- **WHEN** the sun is within a few degrees of the horizon and the eye looks toward it over a cloud deck
- **THEN** the haze on the deck, on the ground and in the sky at the horizon share the dusk tint
