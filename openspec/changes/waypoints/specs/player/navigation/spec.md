## ADDED Requirements

### Requirement: The player places and removes waypoints on the map
With the map open, a right-click on the map SHALL place a waypoint at that
point, and a click within 12 px of a waypoint's pin SHALL remove it. A
left-drag SHALL NOT place or remove one. At most four waypoints SHALL be
kept; placing a fifth SHALL remove the oldest.

#### Scenario: Place and remove
- **WHEN** the player right-clicks two points on the map, then clicks the first pin
- **THEN** waypoints W1 and W2 are placed, and then only W2 remains

#### Scenario: A fifth drops the oldest
- **WHEN** four waypoints exist and the player right-clicks empty map
- **THEN** the oldest is removed and the new one is placed

### Requirement: Waypoints are numbered in the order placed
Each waypoint SHALL carry a number, shown as "W1", "W2" and so on. A new
waypoint SHALL take the lowest number not held by another, and a waypoint
SHALL keep its number until it is removed.

#### Scenario: A removal leaves the others' numbers
- **WHEN** the player places W1, W2 and W3, removes W2, then places another
- **THEN** W1 and W3 keep their numbers and the new waypoint is W2

### Requirement: Waypoints show on the map and the compass
Every waypoint SHALL be drawn on the map as a pin with its label beside it.
It SHALL be drawn on the compass bar at its bearing from the player,
whatever its distance. The waypoint nearest the bar's middle, within the
bar's naming window, SHALL be named under the bar by its label and its
distance along the ground, ahead of a town.

#### Scenario: Facing a waypoint
- **WHEN** the player faces waypoint W1, 1.2 km away
- **THEN** its pin is at the middle of the compass bar and the line under the bar reads "W1 1.2 km"

### Requirement: Waypoints are saved with the world
Placing or removing a waypoint SHALL go through the world's durable journal
on the frame it happens, in the `map-mark` record authored by the player,
with each waypoint's number. Reopening the world SHALL restore every
waypoint with its number. A save whose waypoints were written without
numbers SHALL open with them numbered in the order they were set.

#### Scenario: Survives a reload
- **WHEN** the player places W1 and W2, removes W1, and reopens the world
- **THEN** only W2 is there, at the same place, still W2

#### Scenario: An older save
- **WHEN** a save whose waypoints have no numbers is opened
- **THEN** its waypoints are there, numbered W1, W2, ... oldest first
