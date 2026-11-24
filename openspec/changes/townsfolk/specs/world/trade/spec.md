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

### Requirement: The trade panel lists every good in a window that scrolls and filters
The trade panel SHALL list every good the trader sells and every good in
the pack it buys, each column in a window of a fixed number of rows that
scrolls on its own by whole rows with the mouse wheel and a scroll bar
shown only when the column overflows, the purse and BACK fixed under it.
Each column SHALL offer ALL and one filter per shelf (food, organics, raw,
tools, goods) that has goods in it, read from each good's group, and a
filter change SHALL show the column from its top.

#### Scenario: A long stock scrolls
- **WHEN** a trader sells more goods than its column shows and the player
  turns the wheel one notch over the BUY column
- **THEN** the BUY column shows from its second good, the SELL column is
  where it was, and the panel's height is unchanged

#### Scenario: Filtering to food
- **WHEN** the player chooses the food filter over the BUY column
- **THEN** only the trader's crops, fruit, fish and dishes are listed,
  from the first of them
