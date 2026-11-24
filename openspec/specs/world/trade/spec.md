# Trade Specification

## Purpose
Buying and selling with a town's trader: prices from what the town makes and
needs, limited stock, and prices that move with your sales
(`townsfolk` design 6, 7 and 12; `pbd_core::economy::trade`,
`assets/config/trade.ron`).

## Requirements

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
- **THEN** no sale pays more than the one before, the last pays less than
  the first, and the price is back at rest a game day later

### Requirement: Goodwill gives better prices
Each town SHALL keep a goodwill number for the player, raised by trade and
by jobs done, and each step of it SHALL take 2% off what its trader asks and
add 2% to what it pays, up to 10%.

#### Scenario: The price at each step
- **WHEN** the player's goodwill with a town is at each step from none to the
  most
- **THEN** each step's prices are no worse than the step before, and the
  bonus is 2% a step, 10% at the most
