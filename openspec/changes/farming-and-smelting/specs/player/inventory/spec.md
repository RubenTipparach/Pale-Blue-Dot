## ADDED Requirements

### Requirement: A slot can hold a good, saved by its short code
A slot SHALL be able to hold a good: a row of one append-only goods table
(seeds, produce, coal, ore, ingots), with that row's stack limit and icon. A
good SHALL be saved by the row's short code, never its position in the table,
and a code the build does not have SHALL make only its own line unreadable.

#### Scenario: A save round trip
- **WHEN** a slot holding three wheat is saved and read back
- **THEN** it holds three wheat, saved as `gwheat,3`

#### Scenario: An old save
- **WHEN** a save written before goods existed is opened
- **THEN** every slot, edit and record in it reads as it did before
