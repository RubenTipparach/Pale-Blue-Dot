# Player: the hand in first person

## Purpose

How the player's own hand looks while it holds a tool.

## ADDED Requirements

### Requirement: The hand is one skin tone and lit as it faces

The first-person hand SHALL be drawn in one skin tone. Each face SHALL be lit by its own outward
normal against a light that stays fixed in eye space while the tool moves.

#### Scenario: A swing

- **WHEN** the tool in hand swings
- **THEN** the hand's lit side stays toward the light, and no finger segment is drawn darker than its neighbours by its paint
