## ADDED Requirements

### Requirement: A townsperson stops to talk

A townsperson the player talks to SHALL stand where they were and face the
player until the talk ends, then SHALL carry on from that place.

#### Scenario: A walker spoken to

- **WHEN** the player talks to a townsperson walking their route
- **THEN** they stop and face the player, and when the talk ends they walk on from where they stopped

### Requirement: A taken job can be the active mission

The Missions screen SHALL list the player's taken jobs, and SHALL let one
be tracked. The HUD tracker SHALL show the tracked job.

#### Scenario: Tracking a job

- **WHEN** the player takes a job and presses TRACK on it
- **THEN** the HUD tracker shows that job and its objectives

### Requirement: Picked wild plants grow back

A picked wild crop or glowing mushroom SHALL grow back two game days after
it was picked, out of the player's sight.

#### Scenario: Two days on

- **WHEN** the player picks a wild plant, walks away, and returns more than two game days later
- **THEN** it stands there again

### Requirement: A placed torch looks like the held torch

A torch placed in the world SHALL be drawn with the held torch's haft, head
and flame.

#### Scenario: Setting a torch down

- **WHEN** the player places the torch they carry
- **THEN** the torch on the ground has the same haft, head and flame

### Requirement: An old town gains its forge where the player built nothing

A town laid before forges were added (layout 1 or 2) SHALL be given its
smithy (the forge, the anvil and the shed) the first time it is stood,
by the world through the durable path, keeping the rest of the town where
it stands; and SHALL NOT be given it while any of the smithy's cells holds
anything the player made.

#### Scenario: An old village

- **WHEN** a world saved with a village at layout 2 is opened
- **THEN** the village's smith works at a forge, its record names layout 4,
  and its ground and houses stand where they stood

#### Scenario: The player built there

- **WHEN** the player had placed a block on one of the smithy's cells
- **THEN** the village stays at its old layout, with no smithy
