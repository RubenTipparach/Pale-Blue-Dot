## ADDED Requirements

### Requirement: A block placed under the player's own feet lifts the player onto it
A block aimed at the layer the player's feet stand in, in the player's own cell, SHALL be placed when the two layers above the feet's new height are open, and the player SHALL be lifted one block onto it. With no head room above, the block SHALL be refused.

#### Scenario: Pillar up out of a hole
- **WHEN** a player at the bottom of a hole with open sky above looks down and places a dirt block
- **THEN** the block is placed where the feet were and the player stands on it, one block higher

#### Scenario: No head room
- **WHEN** a solid block is right above the player's head
- **THEN** a block placed under the feet is refused and nothing moves
