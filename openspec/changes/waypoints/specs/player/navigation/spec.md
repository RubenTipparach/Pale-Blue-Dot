## ADDED Requirements

### Requirement: The player places and removes waypoints on the map
With the map open, a right-click (or Ctrl+click) on the map SHALL place a
waypoint at that point, and a right-click within 12 px of a waypoint SHALL
remove it. A left-click or left-drag SHALL NOT place one. At most eight
waypoints SHALL be kept, each with the lowest free number from 1 to 8. A
placement past eight SHALL be refused with a message, and nothing SHALL be
placed.

#### Scenario: Place and remove
- **WHEN** the player right-clicks two points on the map, then right-clicks the first pin
- **THEN** waypoints 1 and 2 are placed, and then only waypoint 2 remains

#### Scenario: A ninth is refused
- **WHEN** eight waypoints exist and the player right-clicks empty map
- **THEN** no waypoint is added and the map says why

### Requirement: Waypoints show on the map and the compass
Every waypoint SHALL be drawn on the map as a numbered pin in its colour. It
SHALL be drawn on the compass bar at its bearing from the player, in its
colour, whatever its distance. The waypoint nearest the bar's middle SHALL be
named under it with its distance along the ground, ahead of a town.

#### Scenario: Facing a waypoint
- **WHEN** the player faces a waypoint 1.2 km away
- **THEN** its pin is at the middle of the compass bar, labelled with its number and "1.2 km"

### Requirement: Waypoints are saved with the world
Placing or removing a waypoint SHALL go through the world's durable journal
on the frame it happens, as a record of kind `waypoint` authored by the
player. Reopening the world SHALL restore every waypoint with its number. A
save written before waypoints existed SHALL open with none and otherwise
unchanged.

#### Scenario: Survives a reload
- **WHEN** the player places waypoints 1 and 2, removes 1, and reopens the world
- **THEN** only waypoint 2 is there, at the same place

#### Scenario: An old save
- **WHEN** a save from before this change is opened
- **THEN** it opens with no waypoints and every edit and item it had
