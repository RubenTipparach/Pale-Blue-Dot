# World: water environments

## Purpose

Where each water place occurs and what it is made of, how light behaves under
water, depth zones, and the beaver dams that change a valley.
Proposed with the seas, rivers and marshes mockup; nothing here is built.

## ADDED Requirements

### Requirement: One function says which water place a point is

The game SHALL decide which water environment a point belongs to (open sea,
coast, reef, kelp forest, polar sea, vent field, trench, river, beaver stream,
marsh, estuary) by one function of the seed, the generator version, the
direction and the climate normals. The generator, the fauna spawner and the
map's environment layer SHALL all read that function.

#### Scenario: The three agree

- **WHEN** a sample of directions is classed by the generator, the spawner
  and the map
- **THEN** every direction gets the same environment from all three

#### Scenario: A reef needs warm shallows

- **WHEN** a point is 4 m deep and its coldest month's sea is 21 C
- **THEN** it is not a reef

### Requirement: New water places are a new generator version

Reef, kelp, vent, trench, sea-ice, stepped-river and marsh-pool cells SHALL be
made only by a new terrain generator version, for new worlds. A world SHALL
keep the generator version it names, cell for cell, and opening it SHALL NOT
rebuild it with the new places.

#### Scenario: An old world opens unchanged

- **WHEN** a world saved on the previous generator version is opened by a
  build that carries the new one
- **THEN** its ground digest matches the previous version's pinned digest

### Requirement: Turbidity per environment
Once the environment function exists, the turbidity SHALL be the
environment's (reef, kelp, polar, marsh and the rest), replacing the reading
off the water class that `world/water-environments` makes today.

#### Scenario: A marsh is murky
- **WHEN** the eye is in water the environment function calls marsh
- **THEN** the turbidity is the marsh's, 0.35 per metre

### Requirement: The sea's surface reaches every cell its water reaches

The sea's drawn surface, the eye's water test and the swimmer SHALL agree on
where the sea is: on the column tier, every cell joined to the open sea
through cells that are not solid, up to the high water's layer, SHALL be
under the sea's surface, roofed or open, and a cell no water reaches
SHALL be dry. The sea's own water SHALL be drawn by one surface at one
height, in a cave as in the open.

#### Scenario: A cavern behind a beach is under the sea

- **WHEN** a dry pit and a roofed cavern under sea level are joined to the
  sea across a beach at the sea's level
- **THEN** both are marked reached, the sheet is listed over them, and an
  eye in either under the sea's surface is in the sea's water

#### Scenario: A hollow inland stays dry

- **WHEN** a pit under the high water is joined to no water
- **THEN** it is not reached and no sheet is listed over it

#### Scenario: One surface in a sea cave

- **WHEN** the sea's water stands under a roof joined to the open sea
- **THEN** the sheet draws its top and the mesher draws no top there

### Requirement: Floes float

A floe SHALL be laid only where the water under it is deep enough for its
ice at the sea's mean. No floe SHALL stand on a
beach, a shore terrace or any ground its ice would reach.

#### Scenario: No floes on the beach

- **WHEN** the environment function calls a stretch of sand above the sea
  under-ice
- **THEN** no floe is laid on it

### Requirement: A beaver colony changes a valley through the journal

A beaver colony SHALL be a stored record, and its felled trees, dam cells and
lodge cells SHALL be journal entries authored by the colony's world process.
A proposal that would change a cell the player touched SHALL be refused
whole, and the colony SHALL move to its next site instead. The pond behind a
dam SHALL NOT be saved.

#### Scenario: A broken dam is not rebuilt on the player's cells

- **WHEN** the player breaks a colony's dam and the world is aged ten days
- **THEN** no dam cell is written on a cell the player dug, and the colony's
  record names a new site

#### Scenario: A jetty first keeps the colony away

- **WHEN** the player has built across the stream at a colony's site before
  the colony's first dam proposal
- **THEN** that proposal is refused and the dam is built at the next site
