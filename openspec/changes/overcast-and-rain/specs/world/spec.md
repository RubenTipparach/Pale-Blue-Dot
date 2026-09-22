# World: weather

## ADDED Requirements

### Requirement: Cloud cover dims the light

The direct sun and the ambient fill on the ground, the water's sun specular,
and the sky dome SHALL be dimmed in proportion to the field's cloud cover over
the player, by the configured overcast factors.

#### Scenario: A storm is not lit like noon

- **WHEN** the cover over the player is 1
- **THEN** the ground's direct sun is scaled by `1 - overcast_sun_dim`
- **AND** its fill by `1 - overcast_amb_dim`
- **AND** the sky's blue scattering is cut by `overcast_sky_blue_cut`

### Requirement: Wet ground reflects the sky

A wet surface SHALL reflect the sky by a Fresnel term along its rippled
normal, and the ripple SHALL NOT only darken the surface.

#### Scenario: A ring on a puddle is a bent reflection

- **WHEN** a raindrop ring perturbs the normal of a flat wet face
- **THEN** the face is brighter than the unperturbed wet face on one side of
  the ring and darker on the other

### Requirement: Rain can be seen from outside it

Rain SHALL be drawn over every raining cell in view, as streaks within the
detail range and as curtains beyond it, and the lens drops SHALL NOT be drawn
above `rain_lod_alt_m`.

#### Scenario: A storm on the horizon

- **WHEN** a cell 400 m from the camera is raining and the camera's is not
- **THEN** a rain curtain is drawn over that cell
- **AND** no rain runs down the lens
