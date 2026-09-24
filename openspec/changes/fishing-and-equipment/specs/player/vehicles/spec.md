# Vehicles Specification

## MODIFIED Requirements

### Requirement: One interaction key boards and leaves
One key SHALL board the craft in reach from on foot and leave the craft the
player is in, and it SHALL appear in the binding table that the settings page
and `--help` print. On foot, the board SHALL happen on a tap of that key: a
release before the tool picker's hold threshold. Holding it past the
threshold SHALL open the tool picker and board nothing.

#### Scenario: Boarding the canoe from the pier
- **WHEN** a walker stands within reach of the Loon's boarding point and
  taps the interaction key
- **THEN** the walker is aboard as the paddler and the vehicle camera is active

#### Scenario: Holding the key beside a craft
- **WHEN** a walker within reach of a craft holds the interaction key past the
  hold threshold
- **THEN** the tool picker opens and the walker is not aboard
