# Surface Rendering Specification

## MODIFIED Requirements

### Requirement: View-dependent shading is computed in the body's own frame
Every pass that shades a body SHALL receive the camera in that body's local
frame, and SHALL compute altitude, view direction, view distance and every
Fresnel or specular term in that frame. A pass SHALL NOT treat the world origin
as a body's centre.

#### Scenario: A body away from the world origin
- **WHEN** a body is rendered at a position far from the world origin
- **THEN** its altitude-gated fog and limb rim behave exactly as they do at the
  origin
- **AND** its Fresnel and specular terms resolve against the true view vector

#### Scenario: The same body at two positions
- **WHEN** a body is rendered at the origin and then offset, with nothing else
  changed
- **THEN** the two images match

## ADDED Requirements

### Requirement: A body's look is data
The values that decide how a body appears SHALL be supplied per body as data:
atmosphere presence, shell radii, scattering scales and wavelength ratios, the
sunset set, fog, rim colour and power and intensity and night floor, water
colours and absorption and sky-reflection tones, and cloud colour and density.
Adding a body with a different appearance SHALL NOT require editing shader or
renderer source.

#### Scenario: Adding a body with a different sky
- **WHEN** a body is added that needs a different sky hue
- **THEN** its wavelength ratios are supplied as data
- **AND** no shader source changes

### Requirement: An omitted value inherits, and zero is a value
A body SHALL override only the values that differ from the global default. An
absent value SHALL mean inherit. A zero SHALL be a real value, so a body can ask
for no rim, no fog or no cloud and be given exactly that.

#### Scenario: A body that wants no limb rim
- **WHEN** a body declares a rim intensity of zero
- **THEN** it renders with no limb rim
- **AND** it does not inherit the global rim instead

### Requirement: One predicate answers whether a body has air
Sky scattering, distance fog, the cloud layer and any precipitation SHALL all be
gated on a single predicate. They SHALL NOT be four independent conditions.

#### Scenario: An airless body
- **WHEN** a body has no atmosphere
- **THEN** it draws no sky scattering, no distance fog, no clouds and no weather
- **AND** all four follow from the same answer

#### Scenario: Turning a body's atmosphere off
- **WHEN** a body's atmosphere is switched off
- **THEN** sky scattering and distance fog stop
- **AND** the surface and water keep their own colours, so the switch isolates
  what the atmosphere was contributing
