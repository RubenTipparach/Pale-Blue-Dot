# World: water environments

## Purpose

Where each water place occurs and what it is made of, how light behaves under
water, tides and depth zones, and the beaver dams that change a valley.
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

### Requirement: Daylight under water fades with depth

The light reaching a point under water SHALL fall with its depth below the
water's surface by a downwelling coefficient per colour, separate from the
view's absorption per metre of path. The view SHALL also scatter toward the
water's own colour by a turbidity per environment.

#### Scenario: The zones fall inside the column

- **WHEN** the blue light left in ocean water is evaluated at 40 m and at
  90 m
- **THEN** it is 15% or more at 40 m (sunlit above) and 1.7% or less at
  90 m (midnight below), so every zone lies above `column::BASE_M`

#### Scenario: Red goes first

- **WHEN** the light left at 10 m is evaluated
- **THEN** red is under 2% while green and blue are above 50%

### Requirement: The tide is an offset of the one sea function

A tide SHALL be added to the sea function's height, so the drawn sea, a
floating hull and a swimmer agree. Its range SHALL be a body's number and its
phase SHALL follow the world clock. It SHALL NOT be saved.

#### Scenario: Shader and core agree with the tide on

- **WHEN** the shader's height and the core function are evaluated at the
  same point and time with a 3 m tide
- **THEN** they agree within 1 mm

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
