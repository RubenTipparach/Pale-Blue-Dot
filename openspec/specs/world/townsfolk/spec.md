# Townsfolk Specification

## Purpose
The people of a settlement: who stands where and what they do, and that they
are solid, all derived from the settlement record and the seed. The rules
are `pbd_core::townsfolk`, the people are in each template's `people`
(exported from the towns mockup's `person()` calls), and the app
(`pbd_app::townsfolk`) places, draws and poses them.

## Requirements

### Requirement: Every settlement has its people, as the mockup has them
Each settlement SHALL have the people its template holds, in the roles the
approved towns mockup gives them: walkers going back and forth between two
points, workers working in place (the smith hammering, the innkeeper
wiping), idlers, and a guard at each gate. Who they are, their post or path
and their look SHALL be derived from the settlement record and the seed, the
same each time the town is built, and no post or path SHALL cross a wall.

#### Scenario: The walled town
- **WHEN** the walled town is built
- **THEN** it has 19 people, among them a smith hammering and a guard at the
  gate (`townsfolk::tests::the_walled_town_has_its_nineteen_with_a_smith_and_a_gate_guard`)

#### Scenario: Built twice
- **WHEN** the same settlement is built twice from the same record
- **THEN** every person has the same role, path, look and starting point
  (`townsfolk::tests::the_same_record_gives_the_same_people`)

#### Scenario: Walls
- **WHEN** a template's people are laid
- **THEN** no post stands in a wall and no path crosses one
  (`townsfolk::tests::no_post_or_path_crosses_a_wall`)

### Requirement: People are solid
A person SHALL be a solid circle of 0.28 m, from the feet to 1.9 m, that the
walker slides round and never passes through.

#### Scenario: Walking into the smith
- **WHEN** the walker walks straight into a working smith
- **THEN** the walker slides round them, and at no tick overlaps them
  (`walking::tests::walking_into_a_working_smith_slides_round_him`)
