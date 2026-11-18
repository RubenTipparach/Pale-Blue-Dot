# World: fauna

## Purpose

What lives in the water. These additions come with the seas, rivers and
marshes mockup; nothing here is built.

## ADDED Requirements

### Requirement: Water life is seen through the water it is in

Every creature and sea place under water SHALL be lit and tinted as the
seabed beside it is: by the terrain's light (its fill, its sun, the sun's
shadow and the lamps), dimmed by the daylight left at its depth under the
surface of the water it is in, and taking the same share of the water's
deep colour as the seabed at that depth. Seen from under the water it SHALL
take the murk by distance, and from above the water's transmission, as the
seabed does. Its atlas and palette SHALL be kept, with no outline.

#### Scenario: A fish near the surface is not a sticker

- **WHEN** the eye is 2 m under a shallow sea by day, looking at a school 3 m off
- **THEN** the fish are dimmed and tinted toward the water as the reef
  beside them is, not drawn in the atlas's full colours

#### Scenario: Fresh water dims too

- **WHEN** a school of perch swims 1 m under a lake's surface that stands
  over sea level
- **THEN** its depth is read from the lake's surface and it is dimmed by it

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
