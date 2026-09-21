# World: saving and loading

## ADDED Requirements

### Requirement: A world can be saved in a named slot

The game SHALL keep several independent saves, and the player SHALL be able to
create one, load one and delete one.

#### Scenario: A new slot starts a fresh world

- **WHEN** the player creates a slot
- **THEN** it has no edits, the starting kit, and the spawn pose

#### Scenario: Deleting a slot needs confirming

- **WHEN** the player presses delete on a slot
- **THEN** nothing is removed until a second, explicit confirmation

#### Scenario: The list does not reorder itself

- **WHEN** the slots are listed twice with nothing changed
- **THEN** the order is the same both times

### Requirement: Loading restores where the player was and what they carried

Loading a slot SHALL restore the player's position and view, their slots and
the selected one, and every edit made to the world.

#### Scenario: A reloaded world is where it was left

- **WHEN** a world with edits and a moved player is loaded
- **THEN** the player stands at the saved position facing the saved direction
- **AND** the slots hold what they held
- **AND** every edited cell reads as it was left

#### Scenario: A save belongs to the world it was made in

- **WHEN** a slot's recorded seed is not the running world's
- **THEN** it is not loaded, and the reason is shown

### Requirement: Saving never blocks a frame

No write, flush or sync SHALL happen on the frame that accepts a change. An
edit SHALL be acknowledged as saved only once the storage has reported the
write down, and never on being queued.

#### Scenario: An edit returns before the disk does

- **WHEN** an edit is accepted
- **THEN** it is queued and the frame continues without waiting for the disk

#### Scenario: Committed means committed

- **WHEN** an edit has been queued but not yet written
- **THEN** it is reported as pending rather than saved

#### Scenario: Quitting does not lose the last edit

- **WHEN** the game exits with writes outstanding
- **THEN** it waits for them to reach the disk before the process ends

#### Scenario: A failed write is reported

- **WHEN** the storage refuses a write
- **THEN** the failure is surfaced rather than swallowed
