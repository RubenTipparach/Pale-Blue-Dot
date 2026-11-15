## ADDED Requirements

### Requirement: A pier's planks are never inside the ground
A harbour pier SHALL be cut so that no stretch of its planks lies under the
ground beneath it: a stretch whose ground reaches the template's deck height
SHALL lie on that ground, the underside of its planks on the highest ground
under the stretch, and every other stretch SHALL stand at the template's deck
height on its piles. What stands on a pier SHALL stand on the same planks.
This SHALL change no stored record and no ground.

#### Scenario: Across the beach from the quay
- **WHEN** each pier of each harbour on the shipped seed is cut
- **THEN** the underside of every stretch's planks is at or over the highest
  ground under that stretch
- **AND** every stretch whose ground is under the template's deck stands at
  the template's deck height

#### Scenario: A saved harbour
- **WHEN** a world saved before this change is opened
- **THEN** its harbour's town record, footprint and ground are as they were
- **AND** only the planks of the stretches that were inside the sand stand
  higher
