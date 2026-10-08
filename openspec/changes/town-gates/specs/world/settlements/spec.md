# Settlements: a gate passage is walled

## ADDED Requirements

### Requirement: A wall's side is drawn wherever its neighbour leaves it open

A masonry cell SHALL draw its side toward a neighbouring masonry cell over
every height the neighbour does not fill, so the passage under a gate shows
the wall on both sides from its ground to its vault.

#### Scenario: Standing in a gate passage

- **WHEN** the walled town and the tundra camp are cut
- **THEN** for each gate cell's edge against a wall cell, that wall cell's
  drawn side spans the gate's ground to the gate's vault
