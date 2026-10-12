# World: storm

## ADDED Requirements

### Requirement: Clouds cover the sea

Clouds between the camera and the sea SHALL be drawn over the sea.

#### Scenario: The ocean from orbit

- **WHEN** the camera is above the cloud layer over an ocean under cover
- **THEN** the clouds are drawn over the sea as they are over land

### Requirement: Distant rain is a volume

Rain SHALL be drawn as an animated volume wherever the weather field rains in
view, in front of the ground and under the clouds.

#### Scenario: A storm on the horizon

- **WHEN** the field rains 500 m from a dry camera
- **THEN** falling rain is drawn in front of the ground there

### Requirement: Snow falls where it is cold

Where the field's precipitation is snow, snow SHALL be drawn instead of rain,
and the ground and the lens SHALL NOT get wet.

#### Scenario: A tundra storm

- **WHEN** it precipitates over a snow biome
- **THEN** flakes fall and no rain drops run down the lens

### Requirement: Lightning lights the storm

In heavy rain, lightning SHALL strike at times and places decided by the field
alone, and each strike SHALL light the clouds, the rain and the ground.

#### Scenario: No lightning in fair weather

- **WHEN** nothing is raining near the camera
- **THEN** no strike happens

### Requirement: The weather can be called

The pause menu SHALL carry a control that sets the storm forcing from clear to
full storm.

#### Scenario: Calling a storm

- **WHEN** the player drags the weather control to full
- **THEN** the field's forcing is one and it rains
