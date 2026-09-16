# Surface Rendering Specification

## ADDED Requirements

### Requirement: The lit body keeps a visible edge at night
The atmospheric limb SHALL retain a floor on the unlit side rather than falling
to zero, so a body's edge stays readable against space at night.

#### Scenario: The night hemisphere from orbit
- **WHEN** the camera views the unlit side of a body from orbit
- **THEN** the limb is still visible
- **AND** the terminator is not washed out by the floor

### Requirement: Look values are data, not literals
Per-body look values - rim colour, rim power, rim intensity, fog height, fog
density, terminator band, ambient and sun tint - SHALL reach the shader as
uniform data with one source for defaults. Adding a second tileset SHALL NOT
require editing WGSL.

#### Scenario: Adding a second body
- **WHEN** a body with a different palette is added
- **THEN** its look values are supplied as data
- **AND** the shader source is unchanged
