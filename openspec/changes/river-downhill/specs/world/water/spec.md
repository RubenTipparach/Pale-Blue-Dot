## ADDED Requirements

### Requirement: A river never runs uphill (generator version 11)
In a world of generator version 11 or later, a river's water surface SHALL
NOT rise anywhere downstream along its course: it falls or stays level, down
every reach and through every confluence. Worlds of earlier versions keep
their rivers as they were made.

#### Scenario: Down every river's line
- **WHEN** the water's top is read every 2 m down the line of every river of a version 11 world
- **THEN** no reading is higher than the one before it
