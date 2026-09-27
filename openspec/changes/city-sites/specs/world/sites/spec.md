# World: sites

## Purpose

Where the planet's settlements stand, before anything is built on them: a
deterministic list of sites generated from the seed and the generator version,
each with a kind chosen by its biome, a footprint, and a name, plus the owner's
authored overrides.

## ADDED Requirements

### Requirement: Sites follow from the seed and nothing else

The list of sites SHALL be a pure function of the seed, the generator version,
the sites version and the authored override list. Computing it twice, in any
order and on any thread count, SHALL give the same sites in the same order
with the same ids. A site's id SHALL be its anchor cell at the site level, so
changing how many sites are kept does not rename the ones that stay.

#### Scenario: Two machines

- **WHEN** the site list is computed twice for one seed, once on one thread and
  once on all of them
- **THEN** both lists are identical, byte for byte

#### Scenario: Fewer sites keeps the same ones

- **WHEN** the configured town count is lowered by one
- **THEN** every site still in the list keeps its id, kind and place

### Requirement: A site stands where its settlement can be built

A generated site SHALL have its whole footprint on dry land, with its surface
range inside its kind's flatness limit. It SHALL have no pentagon cell, and no
neighbour of a pentagon, under the footprint. A harbour SHALL touch the sea,
with shallows within its footprint and a shelf within reach of its quay. No
site SHALL stand in the Ocean biome, and only a cliff village or a cave town
SHALL stand in Mountains. A cliff village's ground SHALL rise between 9 and
25 m across its footprint with no sheer drop, and a cave town SHALL have at
least 16 m of rock over its chamber, above the level of the face its tunnel
enters from.

#### Scenario: Flat, dry ground

- **WHEN** every generated site's footprint is sampled at the terrain's finest
  cell level
- **THEN** no sample is below sea level (a harbour's water rows aside), and the
  surface range is within the kind's limit

#### Scenario: No pentagons

- **WHEN** the footprints are checked against the twelve pentagons
- **THEN** none contains a pentagon or a pentagon's neighbour

#### Scenario: A harbour is on the water

- **WHEN** a harbour's footprint is sampled
- **THEN** part of it is shallows and a shelf lies within its reach

#### Scenario: A cliff village climbs

- **WHEN** a cliff village's footprint is sampled along its downhill direction
- **THEN** the surface rises between 9 and 25 m across it, and no 10 m step
  rises more than 6 m

#### Scenario: A cave town has rock over it

- **WHEN** a cave town's chamber outline is sampled
- **THEN** the surface is at least 16 m above its entrance's level everywhere
  over it, and the ground falls to that level within 30 m of its edge

### Requirement: A site's kind is chosen by its biome

A site's kind SHALL be one of the settlements `tenebris-towns` designs, chosen
by the biome at its anchor:
- walled town or village in fields;
- desert town in desert;
- tundra camp in tundra;
- jungle village in jungle;
- swamp village in swamp;
- harbour where beach meets fields;
- cliff village or cave town in mountains.

A site SHALL NOT take a kind whose biome it does not stand in.

#### Scenario: Every kind has its biome

- **WHEN** every generated site's anchor biome is read
- **THEN** it is the biome its kind belongs to

#### Scenario: Every kind occurs

- **WHEN** the shipped seed's sites are listed
- **THEN** every kind whose biome is on the planet occurs at least once

### Requirement: Sites are spaced apart

Two sites SHALL NOT stand closer than the configured spacing for the larger of
the two, measured on the sphere between their footprints' edges. Towns SHALL
be spaced further apart than villages.

#### Scenario: Measuring every pair

- **WHEN** every pair of sites is measured
- **THEN** no pair is closer than its configured spacing

### Requirement: Every site has a name of its own

Each site SHALL carry a name drawn from its biome's name table, seeded by its
id. No two sites on a planet SHALL share a name. A pinned site SHALL keep the
name the owner gave it.

#### Scenario: Unique names

- **WHEN** the shipped seed's sites are listed
- **THEN** no two names are equal

#### Scenario: A pinned name

- **WHEN** the override list pins a site with a name
- **THEN** that site carries that name, and no generated site takes it

### Requirement: The owner's overrides win

The authored override list SHALL be able to pin a site at a place with a kind
and a name, and to strike a generated site out by its id. A pinned site SHALL
be kept even where the rules would refuse it, and SHALL push generated sites
out of its spacing. The list SHALL be validated at load, and an override on a
pentagon or at sea SHALL be refused with its line named.

#### Scenario: Pinning a town

- **WHEN** the override list pins a walled town at a place
- **THEN** the site list holds that town there, and no generated site is within
  its spacing

#### Scenario: Striking a site out

- **WHEN** the override list strikes a generated site's id
- **THEN** that site is gone, and the rest keep their ids and names

### Requirement: A world keeps the sites it was made with

The first time a world is opened with sites, its resolved site list SHALL be
written to its save through the durable transaction path, with the sites
version that made it. From then on the world SHALL read its sites from the
save. Changing the rules, the counts or the override list SHALL NOT move,
rename or remove a site in a world that already has its list.

#### Scenario: Retuning the counts

- **WHEN** the shipped counts change after a world was first opened
- **THEN** that world still lists exactly the sites it was saved with

#### Scenario: A new world

- **WHEN** a new world is made
- **THEN** its sites are generated from the current rules and overrides, and
  the save records them and the sites version before any is shown

### Requirement: A new player starts near a small town, and the capital is elsewhere

The site list SHALL hold a small settlement (a village or a small walled town)
within about 500 m of the spawn. It SHALL mark one walled town as the capital,
placed away from the spawn and on another land mass where one can hold it. The
override list SHALL be able to move the capital.

#### Scenario: A new world

- **WHEN** a new world's site list is made
- **THEN** a village or small walled town stands within 500 m of the spawn, and
  the capital is on another land mass
