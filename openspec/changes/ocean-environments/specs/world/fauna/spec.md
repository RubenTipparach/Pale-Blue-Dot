# World: fauna

## Purpose

What lives in the water. These additions come with the seas, rivers and
marshes mockup; nothing here is built.

## ADDED Requirements

### Requirement: A species lives in environments as well as water classes

Each species SHALL list the environments it lives in (reef, kelp, vent,
under-ice, pool, riffle, marsh, estuary, mudflat, open water) beside its water
classes and temperature window. A creature SHALL spawn only where the
environment function gives a listed environment, the water class fits and
the temperature is inside its window. New species SHALL be appended to their
body's roster.

#### Scenario: Tube worms only at vents

- **WHEN** a spawn candidate is deep water away from a vent field
- **THEN** no plume worm spawns there

#### Scenario: The roster only grows

- **WHEN** the new species are added to Tenebris's roster (`home` in `fauna.ron`)
- **THEN** every species already on it keeps its index

### Requirement: Bed crawlers and surface floaters move with the world

Besides schools, the fauna SHALL have bed crawlers that walk the tops of the
cells and surface floaters that ride the sea function. Both SHALL be
simulated on the CPU and deterministically, as schools are.

#### Scenario: A floater rides the drawn sea

- **WHEN** a floating otter and the sea function are sampled at the same
  point and time
- **THEN** the otter's height is the sea's height plus its float offset

### Requirement: Some creatures react to the player coming close

Clams SHALL shut, tube worms SHALL pull their plumes in, octopuses SHALL ink
and return to their den, and seals SHALL slide into the water when the
player comes within each species' distance, and SHALL return to their
former state some time after the player has gone.

#### Scenario: A clam shuts

- **WHEN** the player swims within 4 m of an open giant clam
- **THEN** it shuts, and opening it with a knife is refused until it opens
  again

### Requirement: Ducks live on fresh and brackish water

Dabbling ducks SHALL float on ponds, pools and slow rivers, tip up to feed,
and lead ducklings in a line. A flock SHALL lift when startled and land again,
and ducks SHALL sleep on the water at night. Diving ducks SHALL dive and
surface elsewhere. Their eggs and feathers SHALL be gathered as drops.

#### Scenario: A startled flock comes back

- **WHEN** the player startles a flock on a marsh pool
- **THEN** the ducks lift, circle, and land on the water again

### Requirement: Protected creatures cannot be caught

Reef turtles, sea otters, seals and ducks SHALL NOT be caught: the action
SHALL be refused with the reason.

#### Scenario: An otter on the line

- **WHEN** the player tries to catch a sea otter
- **THEN** the catch is refused and nothing enters the slots
