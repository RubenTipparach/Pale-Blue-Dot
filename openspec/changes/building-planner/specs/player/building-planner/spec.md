# Player: building planner

## Purpose

How a player designs a structure before building it: a planner checked by
build mode's own rules, house and building presets and the player's own
blueprints, a blueprint set out in the world as a construction site, built
piece by piece from the pack and changed mid-build, all kept as stored
records.

## ADDED Requirements

### Requirement: The planner designs a structure on the hex grid

The game SHALL have a planner screen, opened at a drafting table or from
build mode, in which the player designs a structure on the hex grid one
storey at a time: rooms painted cell by cell, whose outside edges are walls,
inner walls, doors and windows on any wall, stairs, lanterns, the roof's
shape, the kit, and furniture slots.

#### Scenario: Painting a room

- **WHEN** the player paints three adjacent cells on the ground storey
- **THEN** the plan holds three floors and a wall on every outside edge of
  the three, and no wall between them

#### Scenario: Opening the planner from build mode

- **WHEN** the player presses O in build mode, aimed at nothing
- **THEN** the planner opens on an empty plan

### Requirement: The planner checks a plan with build mode's rules

The planner SHALL check a plan with the same rule functions build mode uses
for a single piece, on the plan charted onto a flat patch of its own. A
broken rule SHALL be shown on the piece that breaks it, red, with the rule's
own reason. A plan larger than the planner's limits SHALL be refused with
its own reason. A plan with a refusal SHALL NOT be set out.

#### Scenario: An upper floor over nothing

- **WHEN** a floor is painted on the upper storey over a cell with no room
  and no wall below it
- **THEN** that floor is red and the checks list names the held-up rule's
  reason

#### Scenario: A stair with no landing

- **WHEN** a stair's head leads into a cell with no floor on the storey
  above
- **THEN** the stair is red with the landing rule's reason

### Requirement: Presets can be picked, edited and saved as the player's own

The planner SHALL offer a library of presets: a straw hut, a cottage, a
two-storey house, a townhouse, a workshop, a shop, a barn, a stable, a
warehouse and a tower, each in a small, standard and large size, and every
plain building of the settlement templates. Picking a preset SHALL load it
as a plan; saving an edited plan SHALL make it the player's own blueprint
under a name the player gives, and SHALL NOT change the preset.

#### Scenario: Saving an edited cottage

- **WHEN** the player loads the cottage, adds a window and saves it as
  "My cottage"
- **THEN** the library lists "My cottage" with the window, and the cottage
  preset is as it was

### Requirement: A blueprint is set out whole as a construction site

A blueprint SHALL be placed in the world as a ghost of the whole plan,
turned in sixths, and checked whole against the world before it is set out:
a five-sided cell, ground too steep for a footing, something in the way, a
piece already in a slot, and build mode's rules against the world's own
pieces. Setting out SHALL cost nothing, SHALL write a site record, and SHALL
mark the site with survey stakes and lime lines in the ground's own light.

#### Scenario: Turning a ghost

- **WHEN** the player turns a blueprint's ghost with shift and the wheel
- **THEN** it turns by a sixth of a turn about its origin cell, and is
  checked again whole

#### Scenario: Something in the way

- **WHEN** a tree stands in a cell under the ghost's ground floor
- **THEN** the ghost is red, says "Something is in the way", and the site
  is not set out

### Requirement: A site is built piece by piece from the pack

In a planned world, each ghost piece of a site that is held up by what is
built SHALL be buildable with the mallet: interacting with it within reach
SHALL strike it on the mallet's beat, and its last stroke SHALL take the
piece's goods from the pack and place it as an ordinary piece record in the
same journal line. A piece whose goods are not in the pack SHALL be refused
with what it needs, taking nothing. The site SHALL show how many pieces are
left and the bill of materials still needed.

#### Scenario: Building a wall

- **WHEN** the player holds F on a buildable timber wall ghost with the
  mallet and 4 planks in the pack
- **THEN** after the wall's strokes the wall stands, the pack has 4 fewer
  planks and the site's count of pieces left is one less

#### Scenario: Not enough planks

- **WHEN** the player strikes a timber wall ghost with 2 planks in the pack
- **THEN** nothing is taken and the line under the crosshair says
  "Needs 4 planks"

### Requirement: Scaffolding reaches an upper storey

When scaffolding is on, a site with a buildable piece on a storey above the
ground SHALL put up scaffolding round that storey's outer walls, solid to
the walker, taking its goods from the pack, and SHALL take it down and give
the goods back when that storey's walls are built.

#### Scenario: The second storey

- **WHEN** the ground storey's walls and the upper floor of a two-storey
  site are built
- **THEN** scaffolding stands round the upper storey's outer walls

### Requirement: A site can be changed mid-build

A site SHALL open in the planner, and saving a changed plan SHALL update the
site: pieces only in the new plan become ghosts, unbuilt pieces the new plan
drops go, and built pieces the new plan drops are marked for removal and
give their goods back when taken down. The planner SHALL list what the
change does, and the goods that come back, before it is saved. A changed
plan the world refuses SHALL NOT be saved.

#### Scenario: Dropping a room half built

- **WHEN** a site's east room has two walls built and its plan is saved
  without the east room
- **THEN** the room's unbuilt ghosts are gone, its two built walls are
  marked for removal, and the planner listed the planks they give back

#### Scenario: Adding a storey

- **WHEN** a finished one-storey site's plan gains an upper storey
- **THEN** the upper storey's pieces are ghosts and the old roof is marked
  for removal

### Requirement: Blueprints and sites are stored records

A player's blueprint and a construction site SHALL each be a stored record
written through the durable path, with the pack in the same line when goods
move. Whether a site's piece is built SHALL be derived from the piece
records, never stored twice. A world saved before this change SHALL open
unchanged, and SHALL build as free building.

#### Scenario: An old save

- **WHEN** a world saved before this change is opened
- **THEN** its pieces are as they were, it has no sites or blueprints, and
  its building setting reads as Free

#### Scenario: A site across a restart

- **WHEN** a site with five pieces built is saved and the world reopened
- **THEN** the same five pieces are built and the rest are ghosts
