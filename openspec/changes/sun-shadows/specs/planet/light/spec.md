## ADDED Requirements

### Requirement: The sun casts shadows
The sun SHALL cast shadows from the terrain, the column tier, trees and
towns onto every surface that takes direct sunlight: the terrain, trees,
clutter, columns and every field-lit object. A shadow SHALL take away the
direct sun only. Shadows SHALL be drawn from the one published sun, and
SHALL hold still while the view moves or turns.

#### Scenario: A ridge shades the valley behind it
- **WHEN** the sun is low behind a ridge
- **THEN** the ground in the ridge's lee takes no direct sun, and the
  ridge's sunward face does

#### Scenario: A house shades the lane
- **WHEN** the sun is low on one side of a house in a town
- **THEN** the lane on the other side takes no direct sun within the
  house's shadow, and does beyond it

#### Scenario: Shadows do not crawl
- **WHEN** the view moves by less than a texel of a cascade, or turns in
  place
- **THEN** that cascade's projection is unchanged

#### Scenario: No shadow from a sun under the horizon
- **WHEN** the sun is below the horizon
- **THEN** no surface takes direct sun, so no shadow is drawn

## MODIFIED Requirements

### Requirement: Nothing unlit is fully black
A surface the sun and sky never reach SHALL still be lit to an ambient floor,
so that an unlit interior reads as a dark place rather than as an absence.
A surface in the sun's shadow SHALL keep the sky's fill.

#### Scenario: A sealed interior is legible
- **WHEN** a face has a sky level of zero
- **THEN** it is drawn at the ambient floor and its material is discernible
  (`planet::terrain::tests::the_shader_carries_the_reference_light_constants`,
  `field_light::tests::the_sky_the_cave_the_night_and_a_torch`)

#### Scenario: A face in shadow keeps the sky
- **WHEN** a face open to the sky is in the sun's shadow at noon
- **THEN** it is lit by the sky's fill, as a face turned away from the sun
  is, and never darker
