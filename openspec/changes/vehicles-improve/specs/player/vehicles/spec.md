# Vehicles improvement delta

## ADDED Requirements

### Requirement: Chase views frame the craft and avoid terrain
The Tern's default chase view SHALL include its masthead. A chase camera
SHALL shorten its boom when terrain obstructs it, with a positive clearance,
and retain the requested zoom distance for when the obstruction clears.
This correction SHALL NOT ease or delay raw mouse look.

#### Scenario: Canoe beside a steep shore
- **WHEN** the Loon's desired chase camera lies behind shoreline terrain
- **THEN** the camera is placed on the clear part of the boom instead of inside the ground

#### Scenario: Tern rig in view
- **WHEN** the Tern is boarded in its default chase view
- **THEN** the masthead and hull fit within the vertical field of view

### Requirement: Boat instruments distinguish water motion from ground motion
Boat telemetry SHALL report motion through water separately from motion over
ground. Tern leeway SHALL measure water-relative motion, and its instruments
SHALL show apparent-wind side and angle, windward speed made good over ground,
and a labeled hull-speed percentage.

#### Scenario: Drifting with a cross-current
- **WHEN** a boat moves with the water in a cross-current
- **THEN** its speed through water and leeway are near zero even though its ground speed is nonzero

#### Scenario: Trimming on either tack
- **WHEN** apparent wind comes from either side of the Tern
- **THEN** the panel identifies port or starboard and shows apparent-wind angle and windward VMG

### Requirement: Vehicle visuals follow configured surfaces and blade state
The Kestrel's drawn wing panels SHALL match their configured area, incidence
and dihedral. The Loon's drawn blade SHALL match its configured area and
its working stroke or stern-rudder position, including at zero water speed.

#### Scenario: Paddle held as a rudder
- **WHEN** the occupied Loon holds a stern rudder with no power stroke active
- **THEN** the blade is shown at the working stern position on the selected side

#### Scenario: Wing configuration
- **WHEN** the Kestrel mesh is built from its vehicle configuration
- **THEN** both panel areas and foil axes agree with the simulation configuration
