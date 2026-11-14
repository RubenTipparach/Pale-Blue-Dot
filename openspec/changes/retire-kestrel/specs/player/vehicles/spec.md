## ADDED Requirements

### Requirement: A retired craft in a save is kept and not flown
A saved craft record whose kind the game has retired SHALL be held aside on
load and SHALL NOT be drawn, boarded or simulated. Every rewrite of the
vehicle file SHALL write it back unchanged, and no new craft SHALL take its ID.

#### Scenario: Opening a world saved with a Kestrel
- **WHEN** a world whose vehicle file holds a Kestrel, a Tern and a Loon is
  opened, a boat is moved, and the world is saved
- **THEN** the world has the Tern and the Loon, no Kestrel is drawn, and the
  Kestrel's record in the saved file is byte-identical to the one read

### Requirement: A new world's fleet is the boats
A new world's fleet SHALL be the Tern and the Loon, placed from the player's
start as before, and the start SHALL be found by the same rule as before.

#### Scenario: A new world
- **WHEN** a new world is made
- **THEN** its fleet is a Tern and a Loon, no aircraft is placed, and the
  player's start is the same place it was before this change

### Requirement: The Loon's blade is drawn as configured and where it works
The Loon's drawn blade SHALL match its configured area and its working stroke
or stern-rudder position, including at zero water speed.

#### Scenario: Paddle held as a rudder
- **WHEN** the occupied Loon holds a stern rudder with no power stroke active
- **THEN** the blade is shown at the working stern position on the selected side

## MODIFIED Requirements

### Requirement: Leaving a craft changes its occupancy, never its existence
Boarding and leaving SHALL change only a craft's occupancy. A left craft SHALL
remain in the world, simulated or asleep under the documented unattended
policy, and boardable with the same ID.

#### Scenario: Leave in the air, reload, reboard
- **WHEN** the sailor leaves the Tern under way (the name is kept from when
  this was the Kestrel in the air), the world is saved and
  reloaded
- **THEN** the Tern with the same ID is in the world where the unattended
  policy put it, and F boards it

## REMOVED Requirements

### Requirement: The VTOL converts from rotor lift to wing lift
**Reason**: The Kestrel is retired (owner, 2026-11-14: "can we remove it,
since we are building better plane systems?").
**Migration**: None in play. A saved Kestrel is kept by "A retired craft in a
save is kept and not flown"; aircraft return with `vehicle-yards`.

### Requirement: Assist in the hover is a dampener
**Reason**: The Kestrel is retired; it was the only craft that hovered.
**Migration**: None; `vehicle-yards` designs its own assist.

### Requirement: Rotor steering requires rotor power
**Reason**: The Kestrel is retired; it was the only craft with rotors.
**Migration**: None.

### Requirement: Vehicle visuals follow configured surfaces and blade state
**Reason**: Its Kestrel half (the wing panels) goes with the Kestrel; the
Loon half is kept, unchanged, as "The Loon's blade is drawn as configured and
where it works".
**Migration**: The Loon's test moves to the new requirement's name.
