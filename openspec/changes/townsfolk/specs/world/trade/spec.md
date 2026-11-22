# World: trade

## Purpose

Buying and selling with a town's trader. Its prices, stock and goodwill are
built and in `openspec/specs/world/trade` (2026-11-21); what is left here
waits for the townsfolk standing in the game to talk to.

## ADDED Requirements

### Requirement: Each town has a trader

Every settlement SHALL have a trader the player can talk to and buy from or
sell to for coins.

#### Scenario: Buying bread

- **WHEN** the player buys a loaf from a town's trader with enough coins
- **THEN** the loaf is in the pack and its price has left the purse
