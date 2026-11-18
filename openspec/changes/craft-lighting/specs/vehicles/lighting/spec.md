## ADDED Requirements

### Requirement: A craft is lit as the terrain is
Every craft drawn in the world SHALL be lit as the terrain round it is: the
terrain's own sun with its shadows and the cell light field, never the
engine's unshadowed sun. Its hull, deck, cargo and crew are lit together.

#### Scenario: A raft in shade
- **WHEN** a raft floats where the terrain's sun shadow falls on the river
- **THEN** the raft is as dark as the river and banks in that shadow
