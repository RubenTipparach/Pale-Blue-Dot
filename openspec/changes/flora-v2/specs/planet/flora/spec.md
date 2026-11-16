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

Deciduous trees' leaves SHALL turn in autumn and fall in winter, and grass
SHALL dry toward straw in winter, by `pbd_core::flora::season_at`, a function
of the clock's year and the place's latitude: opposite in the two hemispheres
and absent near the equator.

#### Scenario: Autumn in the north and the south

- **WHEN** it is a quarter-year after the +Y hemisphere's midsummer
- **THEN** deciduous trees there are in autumn colour
- **AND** at the same latitude across the equator they are not

### Requirement: Tufts and plants wear the ground's palette

A pixel tuft's three tones SHALL be read off the ground tile its biome draws
on grassy cells, by the approved mockup's rule (the tile's texels sorted by
luminance, cut in thirds, each third's commonest colour), and a test SHALL
hold the shipped table to the committed atlas. Tree crowns and bushes SHALL
be drawn at 0.45 of their tile's contrast about its mean colour, and trunks,
twigs and pebbles at 0.7; a beach crown or bush SHALL wear the fields' leaves
through the dry olive gradient map.

#### Scenario: A repainted ground tile

- **WHEN** a biome's ground tile is repainted and the atlas rebuilt
- **THEN** the atlas test fails until the tuft table is the new tile's colours

### Requirement: No painted flowers on the ground tiles

The atlas build SHALL take the painted flowers off the fields grass top and
sod lip, the swamp moss top and the tundra top, so the only flowers on the
ground are the clutter's own; the source sheets SHALL stay as generated and
the build's check SHALL hold the atlas to them.

#### Scenario: A meadow up close

- **WHEN** the player looks at fields grass and the side of a dug step
- **THEN** no daisy is painted in the grass texels or the sod lip

### Requirement: Fallen leaves follow the calendar

In a deciduous biome, up-facing ground SHALL show fallen leaves in its own
texels, landing through autumn, thinned and browned by midwinter, and gone
before spring, thicker in a cell that stands a tree, with none in the tropics.

#### Scenario: Autumn and summer

- **WHEN** a fields meadow is seen at the height of autumn and again in summer
- **THEN** its ground carries scattered fallen leaves, thickest under trees, in autumn
- **AND** none in summer

### Requirement: Trees bear fruit by biome

About half the trees of a fruiting biome SHALL hang fruit from their own
branches (the near model's anchors), and drop a few under them, within the
near trees' ring: fields apple, pear, plum and cherry; jungle mango and papaya; beach
marula and baobab; tundra pines their cones. Fruit SHALL be heaviest in
autumn and nearly gone in winter; cones SHALL stay all year. Fruit is
decoration and SHALL NOT enter a save.

#### Scenario: A fields orchard in autumn

- **WHEN** the player stands among fields trees in autumn
- **THEN** some trees hang fruit under their crowns, the same trees on every visit

### Requirement: Trees are the approved detailed models

Trees SHALL be drawn as the approved M13 mockup's detailed trees (a
parametric trunk, branches and leaf cards per species, four seeded variants a
species), with mid and far trees baked from the near model, and SHALL
cross-fade between near, mid and far through complementary dither masks, so
a tree never appears, vanishes or changes in one frame. Where a tree stands
and whether a cell can be chopped SHALL stay `pbd_core::flora::grows_tree`'s;
a tree's species, variant, turn and size SHALL be decoration derived from its
id at draw time and SHALL NOT enter a save.

#### Scenario: The same tree at every distance

- **WHEN** the player walks away from a tree past the near and the mid rings
- **THEN** it cross-fades from its model to its crossed cards to its billboard
- **AND** every level shows the same tree, baked from the model

#### Scenario: An old world's trees

- **WHEN** a world saved before the detailed trees is opened
- **THEN** a tree stands on exactly the cells it did, and the same cells can be chopped
