## ADDED Requirements

### Requirement: Fish are drawn between their steps

A school's creatures SHALL be drawn between the school's last two steps, at
the fraction of a step its clock has run, never at the raw step of a clock
slower than the frame. A fish held at the lure (nibbling, biting or on the
line) SHALL be drawn at the lure as it is this frame.

#### Scenario: A school swims past while the player watches

- **WHEN** a school steps at 30 Hz and the game draws at another rate
- **THEN** each fish is drawn partway between its two latest steps, so it does
  not stand still for a frame and then jump

#### Scenario: The fish on the line

- **WHEN** a fish is nibbling, biting or hooked
- **THEN** it is drawn at the lure this frame, under the float, not where the
  last school step left it
