# Coins Specification

## Purpose
The one currency: the purse that holds it, the one rule that changes it, and
the value in coins of everything the game knows (`townsfolk` design 5 and 6;
`pbd_core::economy::coins` and `values`, `assets/config/values.ron`).

## Requirements

### Requirement: The purse holds whole coins and is saved
The player SHALL have a purse holding a whole number of coins, never below
zero, saved as a player record. Coins SHALL NOT take a pack slot and SHALL
NOT be dropped by a full pack.

#### Scenario: Saved and loaded
- **WHEN** the player has 37 coins, saves and loads
- **THEN** the purse holds 37 coins

#### Scenario: A full pack
- **WHEN** the pack is full and a job pays 20 coins
- **THEN** the purse gains 20 coins and nothing is dropped

### Requirement: Every change to the purse goes through one rule
Coins SHALL be gained and spent only through one function in `pbd-core` that
names its reason (sold, bought, mission, wage) and refuses a spend greater
than the purse holds.

#### Scenario: Too little to buy
- **WHEN** the purse holds 4 coins and the player tries to buy a good priced 6
- **THEN** the purchase is refused, and the purse and the pack are unchanged

### Requirement: Every item has a value
Every good, block, fish and dish SHALL have a value in coins, kept in one
append-only data file.

#### Scenario: A new good
- **WHEN** a good is added to the game
- **THEN** a test fails until it has a value
