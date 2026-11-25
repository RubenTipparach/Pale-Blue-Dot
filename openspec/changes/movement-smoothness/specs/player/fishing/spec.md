## ADDED Requirements

### Requirement: Fish are drawn between their steps

A school's creatures SHALL be drawn between the school's last two steps, at
the fraction of a step its clock has run, never at the raw step of a clock
slower than the frame. A fish held at the lure (nibbling, biting or on the
line) SHALL be drawn with its mouth at the lure as it is this frame when it
pecks and while it is on the line (`fish-behaviour`, which draws it by its
choreography between pecks).

#### Scenario: A school swims past while the player watches

- **WHEN** a school steps at 30 Hz and the game draws at another rate
- **THEN** each fish is drawn partway between its two latest steps, so it does
  not stand still for a frame and then jump

#### Scenario: The fish on the line

- **WHEN** a fish is nibbling, biting or hooked
- **THEN** its mouth is drawn at the lure this frame (pecking or hooked), under
  the float, not where the last school step left it

### Requirement: A fish's heading is drawn smoothly and a fish at the lure lies level

A creature's drawn heading SHALL ease toward the heading of its school's
latest step over the frames between steps, never turning a whole step's turn
in one frame, and SHALL keep its last heading when its step has no velocity.
A fish held at the lure (nibbling, biting or on the line) SHALL never be
turned by the lure's movement; it stays within its pitch limit of level
(`fish-behaviour` turns it by its choreography: to face the lure, to take it,
round its mouth on the line).

#### Scenario: A fish nibbles under a bobbing float

- **WHEN** a fish nibbles and the float dips and rises
- **THEN** the fish stays within its pitch limit of level; it is never drawn
  standing on its nose or tail

#### Scenario: A school turns

- **WHEN** a fish's step heading changes by a large angle in one step
- **THEN** the drawn fish turns over several frames, not in one
