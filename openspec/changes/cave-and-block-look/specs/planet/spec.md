# Planet: the blocks and the cave

## ADDED Requirements

### Requirement: A cap draws the tile its sheet names for it

Every material code SHALL sample, for its cap, a tile whose name in the
sheet's own manifest describes that material, and a test SHALL hold the
shader's tile table to those names.

#### Scenario: Snow is snow

- **WHEN** a snow cap is drawn from the tundra sheet
- **THEN** the tile it samples is named as snow

#### Scenario: Sand is sand

- **WHEN** a seabed, beach or desert cap is drawn from its sheet
- **THEN** the tile it samples is named as sand

### Requirement: Atmosphere reaches only where the sky does

The distance haze and the limb rim on a terrain face SHALL be scaled by the
face's sky light, so a face the sky does not reach carries no atmosphere.

#### Scenario: A sealed cave has no haze

- **WHEN** a face has a sky level of zero
- **THEN** its colour carries no haze and no rim term

### Requirement: Lens droplets fall

Droplets drawn on the lens SHALL move down the screen.

#### Scenario: A drop's path

- **WHEN** a drop advances through its cycle
- **THEN** its screen position descends monotonically

### Requirement: A wall darkens where a cell stands past its edge

A wall vertex SHALL be darkened for each of the side cell at its layer and the
across and side cells one layer past the wall's edge that are solid, on a
four-rung ladder, with the same rule in the core and the shader.

#### Scenario: A wall standing on a floor

- **WHEN** the cell across from a wall is solid one layer below the wall's foot
- **THEN** the wall's foot is darker than its middle

#### Scenario: A wall under a lid

- **WHEN** the cell across from a wall is solid one layer above the wall's top
- **THEN** the wall's top is darker than its middle
