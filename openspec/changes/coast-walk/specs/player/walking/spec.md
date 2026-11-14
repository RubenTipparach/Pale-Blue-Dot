## ADDED Requirements

### Requirement: A scripted walk can follow a coast
`--coast-walk METRES` SHALL walk the walker at walking pace along the shore
with the sea on one side, cross rivers it meets, keep off cliffs and never
dig, and quit after that many metres along the ground.

#### Scenario: Two kilometres along the shore
- **WHEN** the coast walk runs 2,000 m from the chosen start
- **THEN** no step of its route rises or falls more than 2.5 m, the sea stays within 40 m on its side for most of it, and it crosses at least one river
