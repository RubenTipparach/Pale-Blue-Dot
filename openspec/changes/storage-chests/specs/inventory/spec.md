## ADDED Requirements

### Requirement: Storage chests keep items
A player SHALL be able to make a chest at the workbench, set it down, open it
with F, and move stacks between it and the pack. What a chest holds SHALL be
saved durably and SHALL survive closing and reopening the world. Breaking a
chest SHALL drop everything it holds.

#### Scenario: A stack stored and kept
- **WHEN** the player puts a stack in a chest and the world is closed and reopened
- **THEN** the chest still holds that stack

#### Scenario: A broken chest drops its contents
- **WHEN** the player digs a chest that holds stacks
- **THEN** each stack drops as a pickup at the chest, and the chest's item drops too

## REMOVED Requirements

### Requirement: The trash slot destroys a stack
**Reason**: The owner, 2026-11-22: "I would remove the trash icon and opt for a storage system instead, since excess items can be refined in future updates, or traded for money".
**Migration**: None; the trash held nothing. Excess goes in a chest or is dropped.
