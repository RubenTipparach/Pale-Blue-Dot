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

### Requirement: Some creatures react to the player coming close

Clams SHALL shut, tube worms SHALL pull their plumes in, octopuses SHALL ink
and return to their den, and seals SHALL slide into the water when the
player comes within each species' distance, and SHALL return to their
former state some time after the player has gone. Every such change SHALL be
seen happening: a seal slides off its floe and swims away under the surface,
a diving duck sinks and rises, a startled floater swims away from the
player. No creature SHALL appear or vanish in one frame, near or far: a
school fades in when it is spawned and out before it is dropped, and
ambient life fades with its tile.

#### Scenario: A clam shuts

- **WHEN** the player swims within 4 m of an open giant clam
- **THEN** it shuts, and opening it with a knife is refused until it opens
  again

#### Scenario: A seal is seen leaving

- **WHEN** the player walks or swims toward a seal from 30 m to 2 m
- **THEN** the seal is drawn in every frame of the approach: lying out, then
  sliding in, then swimming off under the surface

### Requirement: Ducks live on fresh and brackish water

Dabbling ducks SHALL float on ponds, pools and slow rivers, tip up to feed,
and lead ducklings in a line. A flock SHALL lift when startled and land again,
and ducks SHALL sleep on the water at night. Diving ducks SHALL dive and
surface elsewhere. Their eggs and feathers SHALL be gathered as drops.

#### Scenario: A startled flock comes back

- **WHEN** the player startles a flock on a marsh pool
- **THEN** the ducks lift, circle, and land on the water again
