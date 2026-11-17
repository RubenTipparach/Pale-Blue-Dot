# Player: the rod in hand moves with the line

## Purpose

What the rod in hand does while fishing, and how the hand holding it is shaded.

## ADDED Requirements

### Requirement: The rod in hand moves with the line

The rod in hand SHALL:

- draw back while a cast charges;
- flick forward on the cast;
- twitch on a nibble and dip on a bite;
- shake with the line's tension while a fish is on.

The motion SHALL be drawn only. No catch rule SHALL read it.

#### Scenario: A bite dips the rod

- **WHEN** a fish bites
- **THEN** the rod's tip dips toward the water within one frame and shakes until the window ends

### Requirement: The line leaves the rod tip as drawn

The fishing line SHALL leave the rod tip where the rod is drawn that frame.

#### Scenario: The line follows a drawn-back rod

- **WHEN** the rod is drawn back while charging
- **THEN** the line hangs from the drawn-back tip, not from the tip's rest position

### Requirement: A held hex prism is shaded by its true face normals

Each face of a hand's prism SHALL be shaded by the normal of that face, so a
flattened prism's slanted faces are lit as they point.

#### Scenario: The back of the hand

- **WHEN** the back of the hand, a flattened prism, is built
- **THEN** each of its side faces is shaded by the cross product of its edge and the prism's axis
