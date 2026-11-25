# Platform: bot control

## ADDED Requirements

### Requirement: A showcase run can be driven live

A `--showcase` run SHALL accept commands on a local socket (127.0.0.1 only)
that report the bot's state and queue, skip, pause and resume its acts and
reload its shot list, without restarting the run or losing the bot's memory.
A plain run SHALL open no socket.

#### Scenario: An act run live

- **WHEN** a client sends `{"cmd": "do", "act": "Hand(\"shovel\")"}` to a running showcase
- **THEN** the bot picks the shovel with G and the wheel before going on with its shot

### Requirement: The channel changes nothing but the bot's inputs

A command SHALL act only through the bot, which presses the player's own
keys and buttons; no command sets an item, a count, a mission or the world.

#### Scenario: Skipping an act

- **WHEN** a client sends `skip`
- **THEN** the bot's current act ends and no mission counts
