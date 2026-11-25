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

### Requirement: A deal is paid in coins, goods or both
A trade SHALL take a basket of goods each way (`townsfolk` design 37a). The
player SHALL be able to pay with coins, with goods valued at the trader's
buying price, or with both, and the difference SHALL be settled in coins
either way. A deal SHALL be all or nothing: a purse that cannot cover the
difference changes nothing. The pack and the purse SHALL change together in
one durable line, after the town's and the world's market lines.

#### Scenario: Paying half in goods
- **WHEN** the player takes two loaves and puts fish worth part of their
  price on the other side
- **THEN** the fish leave the pack, the loaves enter it, the rest of the
  price leaves the purse, and the pack and the purse are in the same saved
  line

### Requirement: A made good is worth more than what it is made of
Raw materials SHALL carry a base value in `values.ron`. A good made by a
recipe SHALL be valued at start from its inputs and its labour (0.1 coin a
second of the recipe) times the craft margin, so the outputs of the recipe
that prices it are worth more than its inputs, and a dearer input SHALL
make the goods made from it dearer. A tool, and a good `values.ron`
overrides, SHALL keep its typed value.

#### Scenario: Planks from a log
- **WHEN** the values are worked out at start
- **THEN** every made good's recipe gives more value than it takes

#### Scenario: Dear tin
- **WHEN** tin's market index rises
- **THEN** bronze is priced higher

### Requirement: Prices follow a market per good
Each good SHALL have a market index per world, stored as the `market`
record, that falls as the player sells it and rises as the player buys it,
moves each game day by the towns' making and using, and drifts back toward
rest over game days. A trader SHALL price from the good's value with the
index folded in, times the town's need and the player's goodwill. A world
saved before the market SHALL open with every index at rest and write no
market until a trade or a day moves one.

#### Scenario: Flooding the fish market
- **WHEN** the player sells many fish
- **THEN** the fish index falls, and it rises back toward rest over the
  following days of play
