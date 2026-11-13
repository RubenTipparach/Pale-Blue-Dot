# Player: navigation

## Purpose

Waypoints on the planet map and the GPS readout a player steers by.
Proposed with the mockups; nothing here is built.

## ADDED Requirements

### Requirement: A route is waypoints placed on the map
A player SHALL add, move and remove waypoints on the map screen, in order.
The route SHALL be a stored record on the craft, saved through the durable
path, naming its author. A waypoint SHALL be reached within 30 m, and the
next SHALL then become active.

#### Scenario: A route survives a reload
- **WHEN** a player places three waypoints, closes the world and opens it
- **THEN** the craft has the same three waypoints and the same active one

### Requirement: The GPS readout gives what a helmsman needs
While a route is active, the readout SHALL show the bearing and the distance
to the active waypoint, the course and the speed over the ground, the
cross-track error and the time to the waypoint. Bearings SHALL be compass
bearings, as the compass bar reads them, and the compass bar SHALL mark the
waypoint.

#### Scenario: The bar and the readout agree
- **WHEN** the readout gives the waypoint at bearing 090
- **THEN** the compass bar's waypoint marker sits under E
