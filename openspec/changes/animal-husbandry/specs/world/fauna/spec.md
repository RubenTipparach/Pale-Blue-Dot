# World: fauna

## Purpose

Every animal on every world in one taxonomy, and what each one may become.
These additions come with the husbandry mockup; nothing here is built.

## ADDED Requirements

### Requirement: Every species has a class and a status

Each species on every body's roster SHALL name its class (mammal, bird,
reptile or amphibian, fish, arthropod, mollusc or soft-bodied, or an alien
branch) and its status (wild only, tameable, domestic, or protected), and
validation SHALL refuse a species without both.

#### Scenario: A species without a status

- **WHEN** a species is added to a roster without a status
- **THEN** validation refuses the roster

### Requirement: Land and air species join their body's append-only roster

Land and air species SHALL be listed in their body's roster beside its fish,
appended after them, so that every saved index keeps its species. An airless
body's roster SHALL stay empty.

#### Scenario: The fish keep their indices

- **WHEN** the land species are appended to Tenebris's roster
- **THEN** every fish already on it keeps its index

#### Scenario: A barren moon

- **WHEN** the player walks on an airless moon
- **THEN** no animal of any kind spawns there
