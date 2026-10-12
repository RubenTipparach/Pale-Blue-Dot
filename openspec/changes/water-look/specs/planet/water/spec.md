# Planet Water Specification

## ADDED Requirements

### Requirement: The sea is water, not a shallow flat
The sea floor SHALL fall away from the shoreline fast enough that the water a
standing player looks across is deep enough to read as water rather than as the
seabed seen through a film. The ocean relief SHALL NOT be compressed harder
than the land relief: the land is compressed so a walker can climb it, and
nothing walks on the sea floor.

#### Scenario: Standing at the waterline
- **WHEN** a player stands at the shore and looks out to the horizon
- **THEN** the water within that horizon is deep enough that its colour comes
  from the water rather than from the sand under it

#### Scenario: Measuring the shelf
- **WHEN** the seabed is sampled outward from a shoreline
- **THEN** it reaches at least the depth that absorbs the seabed's own colour
  within the distance a standing player can see

### Requirement: The water is authored for this project's tone mapping
Water look values SHALL be authored against the tone-mapping this project
actually applies, and the divergence from Tenebris's values SHALL be recorded
with its reason, since Tenebris applies no tone mapping anywhere and its values
are authored for a framebuffer that simply clips.

#### Scenario: A reader comparing the two
- **WHEN** a reader compares a water value here against Tenebris's
- **THEN** the documentation states that the reference does not tone-map
- **AND** it states that copying the reference's values back in would not
  restore its look
