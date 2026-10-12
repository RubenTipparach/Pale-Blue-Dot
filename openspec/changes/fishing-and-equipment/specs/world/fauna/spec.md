# Fauna Specification

## ADDED Requirements

### Requirement: The fish wiki page is generated from the shipped roster
A wiki page of the fish (each species' thumbnail, entry, tip, water and
numbers) SHALL be written by a generator from `assets/config/fauna.ron`, and
the committed page SHALL equal that generator's output, so the page cannot
disagree with the game.

#### Scenario: A species is retuned
- **WHEN** a species' numbers change in `fauna.ron` and the page is not
  regenerated
- **THEN** a test fails
