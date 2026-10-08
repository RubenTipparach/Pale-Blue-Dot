# Player: compass

## Purpose

Which way the player is facing, and which way the places they know lie, read
the way a compass on Earth reads them.

## ADDED Requirements

### Requirement: North is the pole the planet turns counter-clockwise about

The compass SHALL call north the pole about which the planet turns
counter-clockwise, which is the body frame's -Y. East SHALL be on the right
of a player facing north, and the sun SHALL rise toward east. Every heading,
latitude and compass word a player reads SHALL come from `geo`'s compass
functions. The frame's own functions, which layouts and saved records are
built on, SHALL keep their meaning.

#### Scenario: Facing north at sunrise

- **WHEN** a player at any place faces compass north
- **THEN** compass east is on the right of the screen
- **AND** the sun, as it rises there, is toward compass east

#### Scenario: Latitude a player reads

- **WHEN** a place's compass latitude is read
- **THEN** it is the frame latitude negated and the longitude is unchanged

### Requirement: A compass bar shows the heading while playing

While the player is in the world, a bar across the top of the screen SHALL
show 180 degrees of heading centred on the view's: N, E, S and W, the four
intercardinals, and a tick every 15 degrees. Marks SHALL fade toward the bar's
ends. The heading SHALL hold steady when the view pitches toward straight up
or down. The bar SHALL hide while a menu, the pack or the map is open, and
SHALL fade out between 2 and 3 km above the ground.

#### Scenario: Turning right

- **WHEN** the player turns right from north through a full turn
- **THEN** the bar's centre passes N, NE, E, SE, S, SW, W, NW and N in that order

#### Scenario: Looking at their feet

- **WHEN** the player pitches the view from level to straight down without turning
- **THEN** the heading the bar shows does not change

### Requirement: Places within reach show on the bar

Each of the world's sites within 3 km of the player SHALL show on the bar as a
marker at its bearing. The site nearest the bar's centre, within 7 degrees,
SHALL show its name and its distance.

#### Scenario: Turning toward a town

- **WHEN** a town lies 1.2 km away and the player turns to face it
- **THEN** its marker moves to the centre of the bar
- **AND** its name and "1.2 km" show under it
