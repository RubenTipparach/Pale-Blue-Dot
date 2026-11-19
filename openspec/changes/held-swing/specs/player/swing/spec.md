## ADDED Requirements

### Requirement: A tool's strikes land with the dig

While a block is being broken, the tool in hand SHALL strike in strokes timed
to the dig's progress, the last impact on the break, each stroke fastest at
the impact.

#### Scenario: Chopping wood with the stone axe

- **WHEN** the stone axe breaks wood that takes 0.6 s
- **THEN** it strikes twice, the second impact when the block breaks

### Requirement: Bare hands punch

With nothing in hand, the player's fist SHALL punch in strokes timed to the
dig's progress, as a tool strikes, the last impact on the break.

#### Scenario: Punching a tree for a log

- **WHEN** a player with bare hands holds the use button on a tree, a log taking 2.4 s
- **THEN** the fist punches six times per log, the sixth when the log is taken, and the tree shakes on each punch

### Requirement: Every click swings

With a digging tool or bare hands, the use button SHALL play a stroke whether
or not anything breaks: at the air it carries through with no hold, and at a
block that refuses it, it stops and kicks back.

#### Scenario: Clicking the air

- **WHEN** a player clicks with nothing under the crosshair
- **THEN** the tool or the fist swings once and carries through

#### Scenario: A refused block

- **WHEN** a player with bare hands clicks on stone
- **THEN** the fist strikes it, stops and kicks back, and the stone does not crack
