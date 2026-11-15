# Player: aircraft building

## Purpose

How a player builds chibi planes, helicopters and VTOLs from parts at a
hangar, with every flying number following from the parts. Proposed with the
M24 mockup; nothing here is built.

## ADDED Requirements

### Requirement: Aircraft are built and kept in a hangar

The aircraft builder SHALL open only at a hangar's workbench. A built
aircraft SHALL roll out of the hangar's doors, and its span SHALL fit the
door it rolls out of.

#### Scenario: Too wide for the door

- **WHEN** a design's span is wider than the hangar's door
- **THEN** the builder refuses it with both widths

### Requirement: Flying numbers follow from the parts

The builder SHALL work out each design's flying numbers from its parts at the
body's gravity and air density:
- mass;
- wing area, 1 m^2 per wing cell;
- stall speed;
- hover margin, rotor thrust over weight;
- top speed;
- balance, the centre of mass against the centre of lift;
- range.

The built aircraft SHALL fly with the same parts through the foil model that
flies the Kestrel.

#### Scenario: More wing, slower stall

- **WHEN** two wing cells are added to a plane
- **THEN** its stall speed falls by `sqrt(S / (S + 2))`, and nothing else
  about it changes except its mass, its balance and its range

#### Scenario: A helicopter hovers on its margin

- **WHEN** a helicopter's hover margin is under 1.0
- **THEN** it cannot leave the ground at full power, and the builder says so

### Requirement: The builder's checks are the ones a pilot would feel

The builder SHALL refuse to roll out a design unless every check passes, and
it SHALL name the failing one:
- a single main rotor has a tail rotor at least 3 m behind its mast;
- a plane stalls under the airfield's take-off speed;
- tilt nacelles come in mirror pairs;
- it has wheels, skids or floats;
- the cockpit has a clear view forward;
- the centre of mass is ahead of the centre of lift by no more than 1.5 m.

#### Scenario: No tail rotor

- **WHEN** a helicopter has a main rotor and no tail rotor
- **THEN** the check fails with "the main rotor's torque spins it: add a tail
  rotor"

### Requirement: An aircraft is a stored record with its livery

An aircraft SHALL be a stored record. The record SHALL hold:
- its stable ID;
- its parts grid and the part set's version;
- its livery (colours, a shark mouth, a nose number and the player's
  roundel);
- its pose in a named frame;
- its fuel and its damage.

Leaving it SHALL change its occupancy, never its existence.

#### Scenario: The livery survives a reload

- **WHEN** a player paints a nose number, leaves the aircraft on the apron and
  reloads
- **THEN** it stands where it was left, with the same number
