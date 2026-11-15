# Flora

## ADDED Requirements

### Requirement: Plant decoration is derived and never stored

Tuft grass, flowers, mushrooms, ferns and the plants' seasonal colour
SHALL be drawn from the cell record and the clock alone, by the planet's
existing indirect draws. They SHALL NOT change any cell's material, collide,
enter a save, or need a new generator version, so an existing world opens with
the same ground and new decoration on it.

#### Scenario: An old world opens on the new build

- **WHEN** a world saved before this change is opened
- **THEN** its cells, edits and records are what they were
- **AND** its grassy cells grow the new clutter, by the same hash on every run

### Requirement: Pixel tuft grass is a player's choice

With the Pixel grass setting on, every grass slot of the clutter draw SHALL
draw the pixel tuft - two crossed cut-out cards of a 16 px nine-blade pattern -
in the three-tone ramp of the biome the cell is in, in place of the blade cut
from the ground tile, within the same vertex budget. Off, the blades SHALL be
drawn as before.

#### Scenario: Turning it on

- **WHEN** the player ticks Pixel grass on the settings' GRAPHICS page
- **THEN** the grass near them is drawn as tufts in the biome's ramp
- **AND** the choice is kept in the settings file

#### Scenario: Different biomes

- **WHEN** tufts are drawn in the fields, the jungle and the swamp
- **THEN** each uses its own ramp from `pbd_core::flora::TUFT_RAMPS`, dark at
  the root and light at the tip

### Requirement: Clutter thins out rather than popping

Every clutter piece SHALL dither out through the screen-door mask over the
last `clutter_fade_m` of the clutter reach, so no piece appears or vanishes
in one frame at the edge of its reach.

#### Scenario: Walking toward a meadow

- **WHEN** a grassy cell crosses into the clutter reach
- **THEN** its tufts or blades come in through the dither, not all at once

### Requirement: Clutter varies by biome

Flower heads SHALL take one of the biome's flower colours, and mushrooms and
ferns SHALL grow on grassy cells at a base chance times a per-biome weight
from `pbd_core::flora::BIOME_FLORA`; the visibility pass SHALL list a cell for
them on the same rolls and weights the vertex path draws them with.

#### Scenario: A jungle and a desert

- **WHEN** clutter is drawn in the jungle
- **THEN** ferns are common there
- **AND** a desert grows no ferns or mushrooms

### Requirement: Plants follow the planet's one calendar

The prism trees' deciduous leaves SHALL turn in autumn and dull toward bare
twigs in winter, and grass
SHALL dry toward straw in winter, by `pbd_core::flora::season_at`, a function
of the clock's year and the place's latitude: opposite in the two hemispheres
and absent near the equator.

#### Scenario: Autumn in the north and the south

- **WHEN** it is a quarter-year after the +Y hemisphere's midsummer
- **THEN** deciduous trees there are in autumn colour
- **AND** at the same latitude across the equator they are not
