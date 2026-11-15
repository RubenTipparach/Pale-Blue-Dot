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

### Requirement: The GPS advises the best heading and sail trim
While a route is active on a sailing ship, the GPS SHALL give the heading
with the most velocity made good toward the active waypoint from the ship's
polar at the present wind. It SHALL say which way and how far to turn the
helm, when to tack, and which way and how far to trim the sail for the most
drive. It SHALL NOT steer or trim by itself.

#### Scenario: A waypoint dead upwind
- **WHEN** the active waypoint lies straight into the wind
- **THEN** the GPS advises a close-hauled heading on one tack, not the
  bearing, and later advises a tack

#### Scenario: Sail over-trimmed
- **WHEN** the sail is hauled in past the best angle for the apparent wind
- **THEN** the GPS advises easing it, by how many degrees
