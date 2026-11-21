# Proposal: townsfolk

## Why

**The owner (2026-09-27): "can you also make sure all features in the mockup
are implemented too? make sure its in the roadmap".** And on what they look
like (survey T2): **"this is fine, instead of cubes I prefer hexagonal blocks
that look like cubes, keeps the spirit of the game".**

The towns mockup has people. Each settlement has its own, 16 in the walled
town:
- some walk a path between two points, swinging arms and legs;
- some work in place: the smith hammers, the innkeeper wipes the bar;
- the rest stand and look about;
- a guard on the gate carries a spear;
- every one of them is solid to the walker.

`tenebris-towns` and `cities-in-the-world` left them out as "a separate
change". This is that change, so every feature of the approved mockup has a
place on the roadmap.

## What Changes

- **People in every settlement, as the mockup has them.**
  - Walkers go back and forth along a path between two points.
  - Workers work in place (hammering, wiping, stirring).
  - Idlers look about.
  - A guard stands at each gate.
- **Built from hexagonal blocks that look like cubes** (the owner, T2). Each
  part of the body (legs, body, belt, head, hair, arms) is a hexagonal prism
  in the terrain's pixel style, and clothes and skin take their colours from
  the settlement's people and biome.
- **Solid to the walker.** Each person is a circle of 0.28 m from the feet to
  1.9 m, pushed out of like any thin solid. The walker slides round them and
  never passes through.
- **Lit like everything else that moves.** They take the light field through
  `lamps-and-lanterns`' sampler, so a smith by the forge is lit by it at night.
- **Derived, not stored.** A townsperson with no state is a law, following
  `world-persistence`'s rule. Who stands where, and what they do, comes from
  the settlement record and the seed. When people gain state (the owner's
  "you can steal from people lol, but there will be consequences"), they
  become records in a later change.
- **Out of scope:** schedules, reactions to theft or to a house taken apart,
  and walking off their paths outside a mission. Those are the consequences
  change after the roadmap. Talk, trade, coins, town missions and roads came
  in on 2026-11-21 (below).

## Added 2026-11-21: coins, trade, town missions and roads

**The owner (2026-11-21):** "Townsfolk: add reqs for trading, picking up
custom missions, building roads (slope hexagon dirt + collision or gravel
roads) and generate road segments between some towns." On the missions:
"trade something, hunt or gather some resource for a town, help find a lost
villager guide them back, deliver a package, cook some food, repair a
building, do some fishing, mine some resources, idk whatever else you think
is appropriate". And: "doing missions and rtrading should get you money, so
we need currency".

- **Coins.** One currency, coins, held in the player's purse (a counter, not a
  pack slot), saved as a player record. Missions and selling pay coins;
  buying spends them. The starter missions said "coins come with the
  townsfolk phase" (`starter-missions` design); this is that.
- **Trading.** Each town has a trader (the market stall, or the shop in a
  village). You buy and sell goods for coins. A town's prices follow what it
  makes and what it needs (its biome, its trades, its size), it buys for less
  than it sells, its stock is limited and comes back each game day, and your
  own sales move its price. The trade panel is M5's.
- **Town missions.** Each town posts jobs on its board, and some townsfolk
  offer one when you talk to them. You pick one up, do it, and come back for
  coins and the town's goodwill. The kinds: trade, hunt or gather for the
  town, find a lost villager and guide them home, deliver a package to another
  town, cook a dish, repair a building, fish, mine, and (recommended here)
  tend a herd, bring in a harvest, clear a rockfall off a road, escort a
  trader along a road, and light the town's lamps at dusk. They count with the
  starter missions' rule, so a mission is data and one counting function.
- **Roads.** A road cell is a hex whose top slopes from one layer to the next
  where the road climbs, so a road has no steps, and its slope is solid to the
  walker. A road is dirt (packed, built with a shovel) or gravel (needs
  gravel, faster underfoot). Towns are joined by generated road segments: not
  every pair, a few, the way roads grow (each town to its nearest neighbours,
  the large towns to each other). Roads are laid by the world as stored
  records, so old saves gain them and nothing a player made is overwritten.
- **Townsfolk with state.** The people stay derived, except where the player
  has done something with one: a lost villager, a mission given, a town's
  goodwill. Those are stored records (`world-persistence`).

## Added 2026-11-21: friendly faces

**The owner (2026-11-21):** "Design new and friendly looking NPCs ... do
research on how we can generate and animate interesting looking cartoony low
poly people withe expressive faces. Think of like kerbal space program".
The research and the plan are design 13 to 18: KSP's proportions (a big head
on a short round body) built from six-sided pieces, a face of layered pieces
that are swapped and slid rather than deformed (Wind Waker's eyes, High
Frontier's swap rule), expressions driven by what happens to the person,
blink and gaze running on their own layers, and people generated from the
town and the seed. The look is decided in M5's new Faces tab first.

## Capabilities

### New Capabilities
- `world/townsfolk`: the people of a settlement:
  - who they are and where they stand;
  - how they move;
  - how they look;
  - that they are solid and lit;
  - that they follow from the settlement and the seed.

- `player/coins`: the purse, what pays coins and what spends them.
- `world/trade`: a town's trader, its prices, its stock.
- `world/town-missions`: jobs from a town's board and its people, taken,
  done and paid.
- `world/roads`: sloped road cells, dirt and gravel, and the roads laid
  between towns.

### Modified Capabilities
- None.

## Impact

- **`pbd-core`:** a `townsfolk` module that derives each settlement's people
  from its record and the seed: role, path or post, clothing and pace.
- **`pbd-app`:**
  - people drawn as hexagonal-prism figures with simple animation (walk swing,
    work loop, idle look);
  - their circles joined to the walker's thin-solid pushes;
  - their lighting from the field's sampler.
- **Order:** after `cities-in-the-world`, because they live in its settlements
  and use its thin-solid contact. Before `player-building`.
- **Performance:** a few hundred figures across the towns in range, each a
  handful of prisms. It is not measured in a cloud session (CLAUDE.md). The
  owner's `perf_suite.py` town scenario covers it.
