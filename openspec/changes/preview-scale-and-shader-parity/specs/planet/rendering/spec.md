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

### Requirement: Water preserves the previous appearance
The live water renderer SHALL preserve the appearance and terms of the existing
faithful Tenebris water port: wave displacement and normals, Fresnel reflection,
refraction, foam, optical path-length absorption and the underwater view. The
short water approximation in the terrain surface shader SHALL NOT remain as a
second live implementation.

#### Scenario: Viewing the ocean from above
- **WHEN** a fixed camera and fixed simulation time render the ocean from above
- **THEN** the result matches the faithful water reference for waves, Fresnel,
  refraction, foam and depth-dependent absorption

### Requirement: Above-water and underwater views share one water system
The above-water and underwater views SHALL use the same water geometry, wave
state, shader implementation and parameter set in the same body-local frame.
Crossing the surface SHALL select the appropriate view in that implementation,
not switch to an independently tuned water material or approximation.

#### Scenario: Crossing the water surface
- **WHEN** the camera moves through the surface at a fixed place and time
- **THEN** wave phase, water colour and surface geometry remain continuous
- **AND** the underwater path applies refraction and path-length absorption
- **AND** both views resolve camera and water positions in the same body-local
  frame
