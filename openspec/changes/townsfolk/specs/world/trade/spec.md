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

### Requirement: A deal is paid in coins, goods or both
A trade SHALL take a basket of goods each way. The player SHALL be able to
pay with coins, with goods valued at the trader's buying price, or with
both, and the difference SHALL be settled in coins. The pack, the purse and
the town's market record SHALL change together in one durable write.

#### Scenario: Paying half in goods
- **WHEN** the player buys a bronze pickaxe and puts fish worth half its
  price on the other side
- **THEN** the fish leave the pack, the pickaxe enters it, half the price
  leaves the purse, and all three are in the same saved line

### Requirement: A made good is worth more than what it is made of
Raw materials SHALL carry a base value. A good made by a recipe SHALL be
valued from its inputs and its labour, so it is always worth more than its
inputs, and a dearer input SHALL make the goods made from it dearer.

#### Scenario: Planks from a log
- **WHEN** the values are worked out at start
- **THEN** every made good is worth more than the sum of its inputs

### Requirement: Prices follow a market per good
Each good SHALL have a market index per world that rises when it is short
across the towns, falls when it floods, and drifts back toward rest over
game days. A town's price SHALL be the good's value times its index, the
town's need and the player's goodwill. A world saved before the market
SHALL open with every index at rest.

#### Scenario: Flooding the fish market
- **WHEN** the player sells many fish across the towns in one day
- **THEN** fish fetch less everywhere that day, and the price recovers over
  the following days of play
