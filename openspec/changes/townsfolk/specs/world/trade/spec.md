# World: trade

## Purpose

Buying and selling with a town's trader: prices from what the town makes and
needs, limited stock, and prices that move with your sales.

## ADDED Requirements

### Requirement: Each town has a trader

Every settlement SHALL have a trader the player can talk to and buy from or
sell to for coins.

#### Scenario: Buying bread

- **WHEN** the player buys a loaf from a town's trader with enough coins
- **THEN** the loaf is in the pack and its price has left the purse

### Requirement: Prices follow the town

A town's price for a good SHALL be the good's value times the town's need for
it, buying at 60% of its selling price. A town that makes a good SHALL sell it
for less than a town that lacks it.

#### Scenario: Fish by the sea and in the hills

- **WHEN** a harbour town and a mountain town both trade fish
- **THEN** the harbour sells fish for less than the mountain town buys them

#### Scenario: Buying and selling back

- **WHEN** the player buys a good and sells it straight back to the same trader
- **THEN** the player has fewer coins than before

### Requirement: Stock is limited and prices move with your trade

A trader's stock of each good SHALL be limited, each sale SHALL move its
price, and stock and prices SHALL return to rest over a game day. A town's
market SHALL be saved only once the player has traded there.

#### Scenario: Selling a lot

- **WHEN** the player sells ten of a good to one trader
- **THEN** each sale pays less than the one before, and the price is back at
  rest a game day later
