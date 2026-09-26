# Walking Specification

## ADDED Requirements

### Requirement: A stair is walked on its pitch line
The walking surface of a stair SHALL be its pitch line, from the foot of its
first riser to the nosing of its landing, while its treads are drawn. The
camera SHALL stay at eye height over the feet with no easing.

#### Scenario: Up a straight flight at walking speed
- **WHEN** the walker walks up a straight flight at 8 m/s
- **THEN** the eye never rises more than 0.1 m within one 0.1 m piece of the sweep
- **AND** the walker reaches the landing

#### Scenario: Up a newel stair
- **WHEN** the walker climbs a newel stair along its walk line
- **THEN** the eye never rises more than 0.1 m within one 0.1 m piece of the sweep

### Requirement: A grounded walker is held to the floor below
A grounded walker whose floor falls away by no more than 0.35 m SHALL be held
to it rather than falling, unless it is rising from a jump. A larger drop SHALL
still be a fall.

#### Scenario: Down a straight flight
- **WHEN** the walker walks down a straight flight at 8 m/s
- **THEN** it is grounded on every tick

#### Scenario: Off a terrace
- **WHEN** the walker walks off a 1 m terrace
- **THEN** it falls

### Requirement: A refused move slides
When a thin solid, or a column face too tall to step, refuses a move, the
walker SHALL keep the part of the motion along the face and lose only the part
into it. Headroom refusals SHALL still stop the move.

#### Scenario: Brushing a wall
- **WHEN** the walker walks at a shallow angle into a building's wall
- **THEN** it moves on along the wall at close to its walking speed

#### Scenario: Through a doorway off-square
- **WHEN** the walker walks through an open doorway not square to it
- **THEN** it passes through instead of stopping on the jamb

### Requirement: Thin geometry blocks the body
Walls on edges, jambs, lintels, closed doors, rails, furniture, posts and
people SHALL block the walker's body, and SHALL NOT be climbed with the terrace
step height.

#### Scenario: A table
- **WHEN** the walker walks into a table
- **THEN** it is stopped at the table's edge and does not step onto it

#### Scenario: Jumping indoors
- **WHEN** the walker jumps under a 2.8 m ceiling
- **THEN** the head stops at the boards
