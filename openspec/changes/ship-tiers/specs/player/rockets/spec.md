# Rockets Specification

## ADDED Requirements

### Requirement: Rockets are stacks of hex cells
The assembly building SHALL build a rocket only from parts that each fill one
hex cell of the ground lattice (2.833 m flat to flat) and one 1 m layer: a core
column at cell (0, 0) and the six cells round it. A core cell SHALL sit on the
core cell below it or on the pad, with the command pod as the top core cell. A
ring cell SHALL sit beside a core cell of its layer or on a ring cell below it.

#### Scenario: A gap in the core column
- **WHEN** a stack has core cells at layers 0 and 2 and none at layer 1
- **THEN** the stack is reported invalid and cannot be launched

#### Scenario: Radial symmetry
- **WHEN** a ring part is placed with 3x symmetry
- **THEN** the same part fills the three ring cells 120 degrees apart on that layer

### Requirement: Decouplers split a stack into stages
A decoupler SHALL occur only in the core column and SHALL end a stage: the
cells at or below it belong to the stage that fires first. A stage's tanks
SHALL feed only that stage's liquid engines; its solid boosters SHALL burn
alongside them and separate when spent; the stage SHALL separate at its
decoupler when its propellant is gone.

#### Scenario: Boosters, then the core, then the upper stage
- **WHEN** a stack has three solid boosters and a liquid core below a decoupler and a nuclear thermal engine above it
- **THEN** the boosters and the core light together, the boosters drop when spent, and the upper stage lights after the core stage drops

### Requirement: Thrust-to-weight decides lift-off
A stage's thrust-to-weight ratio SHALL be its lit thrust divided by its mass at
ignition (every stage above included) times the launch body's surface gravity.
A rocket whose first stage is under 1.0 against the launch body SHALL NOT leave
the pad.

#### Scenario: Too heavy
- **WHEN** a first stage's thrust is 0.55 of its weight at 1 g
- **THEN** its engines light and the rocket stays on the pad

### Requirement: Delta-v follows from the parts
Each stage's delta-v SHALL be computed by the rocket equation from its mass at
ignition, its mass at burnout and its engines' exhaust speed (thrust-weighted
while boosters burn with the core), and SHALL be the same number the flight
spends: a rocket that shows enough delta-v for a burn SHALL be able to fly it.

#### Scenario: The assembly building and the flight agree
- **WHEN** a stack shows 2,122 m/s in the assembly building and flies to low orbit spending 535 m/s
- **THEN** the flight reports 1,587 m/s left, within rounding

### Requirement: Tiers unlock parts by the power built
Parts SHALL belong to a tier: chemical and nuclear thermal parts to tier 1,
fusion drives and the nav computer to tier 2, antimatter cores to tier 3. A part of a tier the player has not unlocked SHALL be shown locked
and SHALL NOT be placed. Unlocking SHALL follow from what the player has built
on the power ladder.

#### Scenario: A locked part
- **WHEN** a player without a fusion plant selects the fusion drive
- **THEN** the assembly building names what unlocks it and places nothing

### Requirement: A large rocket has decks to walk
The assembly grid SHALL reach four rings of cells out from the core column.
Crew decks, cabins, ladders, hatches and airlocks SHALL be cells of the
stack. A walkable floor SHALL have two clear layers above it. The assembly
building SHALL refuse a stack whose command pod cannot be walked to from a
hatch over decks and ladders, and SHALL name the check. A walker SHALL stand
on the decks in the rocket's frame, on the pad and in flight.

#### Scenario: An unreachable pod
- **WHEN** a stack has a crew deck under its command pod with no ladder up
- **THEN** the assembly building refuses it with "the pod cannot be walked
  to from the hatch"

#### Scenario: Standing on a deck at lift-off
- **WHEN** a walker stands on a crew deck as the rocket lifts off
- **THEN** the walker stays on the same deck cell as the rocket climbs
