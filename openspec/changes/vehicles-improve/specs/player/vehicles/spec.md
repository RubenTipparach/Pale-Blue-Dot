# Vehicles improvement delta

## ADDED Requirements

### Requirement: Foil forces remain oriented through reverse flow
Every vehicle foil SHALL orient lift perpendicular to relative flow with a
consistent span direction, including reverse flow. Drag SHALL oppose motion
relative to the fluid; changing flow direction SHALL NOT flip lift at
normal incidence through an artificial normal-vector sign change.

#### Scenario: Reverse flow from below
- **WHEN** a symmetric horizontal foil meets reverse flow from below
- **THEN** its vertical force is upward, and its net force does nonpositive work against its motion through the fluid

### Requirement: Rotor steering requires rotor power
The Kestrel's rotor steering SHALL be bounded by delivered rotor power.
Zero throttle with assist off SHALL produce no powered rotor force or
cyclic/yaw torque. Assisted controls SHALL stay finite when local gravity
falls to zero.

#### Scenario: Unpowered controls
- **WHEN** an airborne stationary Kestrel has zero throttle and assist off and the pilot requests rotation
- **THEN** its rotors produce no thrust or steering torque

#### Scenario: Gravity fades away
- **WHEN** the assisted Kestrel is stepped with zero local gravity and centred input
- **THEN** its pose, velocity and controls remain finite

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

### Requirement: Fleet placement respects the planet frame
A new fleet SHALL be placed from the player's body-local position regardless
of the planet's render-frame translation. Boarding and vehicle camera poses
SHALL use the same frame conversion.

#### Scenario: Translated planet
- **WHEN** the planet and walker are translated together within the render frame
- **THEN** the new craft retain equivalent body-local berths and are boardable with correctly translated cameras

### Requirement: Vehicle input is immediate and respects menus
Vehicle mouse look SHALL consume the current frame's raw displacement and
change the camera on that update independently of fixed physics ticks.
An open menu SHALL suppress vehicle controls and mouse look.

#### Scenario: Looking between physics ticks
- **WHEN** mouse displacement arrives while aboard without a physics tick
- **THEN** the camera turns on that update while the craft's physical orientation stays unchanged

#### Scenario: Menu owns input
- **WHEN** the menu is open while vehicle keys and mouse motion arrive
- **THEN** the vehicle control request and look do not change the craft or camera
