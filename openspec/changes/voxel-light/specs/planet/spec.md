# Planet: voxel light

## ADDED Requirements

### Requirement: Light is what reached a cell, not how deep it is

Every cell of the column tier SHALL carry a sky level derived by propagation
from the open sky, losing one level per cell in every direction and stopping at
anything solid.

#### Scenario: An enclosed cave is dark

- **WHEN** a cell is enclosed by solid material beyond the propagation range
- **THEN** its faces are lit only by the ambient floor

#### Scenario: A cave mouth is bright although it is deep

- **WHEN** a cell is open to the sky
- **THEN** it is at full sky level whatever its depth below the surrounding
  ground

#### Scenario: The open surface is unchanged

- **WHEN** a cell's column is open to the sky
- **THEN** the light term is the same as before the field existed

### Requirement: A face darkens where it is wedged into stone

A face's corner SHALL be darkened by how many of the two neighbouring cells
sharing that corner are solid at the layer the face opens onto, and a face's
corners SHALL be sampled independently so the value varies across it.

#### Scenario: A corner against one block is darker than an open one

- **WHEN** one of the two side neighbours is solid at that layer
- **THEN** that corner is darker than a corner with neither

#### Scenario: A corner wedged between two is darker still

- **WHEN** both side neighbours are solid at that layer
- **THEN** that corner is darker than one with a single neighbour

#### Scenario: A wall standing on flat ground has no black line at its foot

- **WHEN** every cell at a face's lowest metre is solid
- **THEN** the corner takes the light of the air above rather than reading dark

### Requirement: Nothing unlit is fully black

A surface the sun and sky never reach SHALL still be lit to an ambient floor,
so that an unlit interior reads as a dark place rather than as an absence.

#### Scenario: A sealed interior is legible

- **WHEN** a face has a sky level of zero
- **THEN** it is drawn at the ambient floor and its material is discernible
