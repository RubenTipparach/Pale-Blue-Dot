# Design: townsfolk

## Context

See `proposal.md` for the owner's words. What the design has to work with
(2026-09-27):

- **The mockup's people** (`docs/mockups/towns.html`, `person()` and
  `animateNpc()`):
  - A person is a group of boxes: two legs (0.18 × 0.8 m), a body (0.5 ×
    0.72 m), a belt, a head (0.3 m), hair, and two arms hinged at the
    shoulder. An optional spear is held in the right hand.
  - A walker ping-pongs along its path at about 1.2 m/s, facing its way,
    with arms and legs swinging at 7 rad/s.
  - A worker animates its right arm: a hammer stroke, or a wipe.
  - An idler turns its head slowly.
  - Each person's solid is a circle of radius 0.28 m from the feet to 1.9 m,
    labelled with its role. It is moved with the person every frame and
    pushed out of like a wall.
- **The game, after the changes this follows:**
  - `tenebris-towns` gives thin-solid contact and the walker's slide;
  - `lamps-and-lanterns` gives the light-field sampler for moving things;
  - `cities-in-the-world` gives settlement records and active settlement
    entities;
  - `world-persistence` gives the rule that what has no state is derived.

## Goals / Non-Goals

**Goals:**
- Parity with the mockup's people: the same roles, movement, solidity and
  numbers.
- The owner's look: hexagonal blocks that read as cubes.

**Non-Goals:**
- **Behaviour beyond the mockup:** schedules and sleeping at night. Talking,
  trading, missions and roads came in on 2026-11-21 (decisions 5 to 12);
  following the player is only for a lost villager on a mission.
- **Consequences:** stealing, a house taken apart, reputation. That is a
  later change, after the roadmap (the owner, T5 and P2).
- **Crowds.** The counts are the mockup's, per settlement.

## Decisions

**1. People are derived from the settlement record.** Each template carries
its people as roles with posts or paths in the layout's axial coordinates.
The settlement's variation (rotation and mirror) charts them onto the sphere
with the buildings. Clothes, skin and pace are rolled from the seed and the
person's index, so a town's people are the same on every visit and on every
machine.

**2. A figure is prisms, animated on the CPU, drawn instanced.**
- Each body part is a hexagonal prism, flat side forward, sized like the
  mockup's boxes. Across the flats it matches the box's width, so the
  silhouette reads as the mockup's.
- The parts' poses are computed on the CPU each frame: the swing, the stroke
  or the look. They are drawn as instances of one prism mesh with the
  terrain's atlas, so a hundred people are one draw.
- The pixel texture is the terrain's, sampled nearest (CLAUDE.md).

**3. Solidity reuses the thin-solid contact.** A person's circle is a thin
solid in the settlement's contact index. The index updates the circle's
centre each tick, as the mockup does, and the walker's push-and-slide handles
it with no special case.

**4. Lighting reuses the sampler.** Each person samples the field at the
corners of their bounds, like the player's body, so no second lighting path
exists.

**5. Coins are a counter in the purse, one rule in `pbd-core`.** The purse
is a player record (`purse`, schema 1): a whole number of coins, never
negative. It is not a pack slot and has no stack limit, so it can never be
dropped by a full pack. Every change to it goes through one function that
takes a reason (`sold`, `bought`, `mission`, `wage`) and refuses a spend the
purse cannot cover, so single player and a future server pay the same way. A
coin has no weight. The pack shows the count beside the hotbar with a coin
icon (the UI's 1 px outline is allowed there).

**6. A good's value is one number; a town's price is that number times its
need.** Every good, block, fish and dish gets a base value in coins, in data
(`assets/config/values.ron`, append-only like the goods table). A town's
buying price is `value x need x 0.6` and its selling price
`value x need x 1.0`, where `need` runs 0.5 (it makes this) to 2.0 (it lacks
it), from the town's biome and its trades (a harbour sells fish cheap, a
mountain town pays well for it). Recommendation taken (ask only with
screenshots): a 40% spread, so buying and selling back loses coins.

**7. Stock is limited and comes back with the day.** A town holds a stock of
each good it sells, sized by the town, and wants a quantity of each good it
buys. Each sale moves the price 2% toward the far end of its range, and the
town's stock and prices drift back to rest over a game day. Stock and drift
are a stored record per town (`town_market`), written only when you trade, so
an unvisited town costs nothing.

**8. A town mission is a mission row, made from the town's needs.**
- A mission has the starter missions' shape: objectives a kind, what it
  counts and how many, counted from the same `Happened` events. New kinds
  join `Kind`: `Deliver` (a parcel item to a named townsperson in another
  town), `Escort` (a townsperson walked to a place), `Repair` (a broken
  piece fixed), `Sell` and `Buy` (at a trader), `Light` (lamps lit), `Clear`
  (a road cell dug clear). Hunting waits for survival's predators; until then
  "hunt" asks for a herd animal's goods (meat, hides, wool).
- A town's board shows three to five jobs, made from its needs, its trades
  and the seed of the game day: a harbour asks for fish, a mining town for
  ore, a town short of food for a dish. A job not taken is replaced after
  three game days.
- Taking a job writes it into the missions record as an open mission with
  its town and its reward; it never fails or expires once taken (Explorer is
  calm; Survival may add time limits later). Coming back to the giver pays.
- Pay is the sum of the objectives' values times 1.5, rounded to whole coins,
  plus goodwill with the town (decision 11).

**9. The special jobs.**
- **Lost villager:** a townsperson becomes a record, standing somewhere out
  of town within 300 m (a clearing, a cave mouth, a far shore), with a marker
  on the map. Talk to them and they follow you at your pace, stopping when you
  stop; they never swim, and wait at deep water. Bring them within the town's
  wall and the job is done.
- **Deliver:** the giver hands you a parcel (a good, `parcel`, one per job,
  carried in the pack). Hand it to the named townsperson in the other town.
- **Repair:** a town building has a broken piece (a stored, world-made
  change: a missing wall cell, a broken fence). Place the material back (the
  same block, or planks), and the job counts it. The world never breaks a
  piece the player placed.
- **Escort a trader:** a trader walks the road to the next town and you walk
  with them; they wait if you fall behind.

**10. Roads: a sloped cell, dirt or gravel.**
- A road cell is a hex whose top is a plane tilted from one edge's layer to
  the opposite edge's, never more than one layer (1 m) over a cell (about 19
  degrees). Where a road climbs more than a layer between cells it zig-zags
  or cuts the hill, never steps. A flat road cell is an ordinary cell with a
  road top.
- Its collision is that plane, solid to the walker and to drops, so walking
  up it is smooth and needs no step-up. Carts, later, roll on it.
- Two surfaces: **dirt track** (packed earth, built with a shovel on any
  cell) and **gravel road** (gravel, from a gravel pit or crushed stone,
  walked 15% faster). Recommendation taken (ask only with screenshots):
  generated roads are dirt between villages and gravel between the large
  towns; the look goes into a mockup first.
- The player builds a road by placing road cells, which are edits like any
  other.

**11. Roads between towns are laid by the world.** On opening a world at a
build that knows roads, the road network is planned once from the sites and
the seed and saved as stored road records, authored by the world:
- each settlement joins its two nearest neighbours within 2 km, and every
  town of the largest size joins its nearest such town;
- the route follows the ground's cost (slope, water, cliffs), crossing a
  river at a ford where it is shallow, otherwise a timber bridge (a piece);
- a cell is laid as road only where no player edit stands; where one does
  the road goes round it. Nothing the player made is overwritten.
Saves made before roads gain them on next open, in place, without a restart,
as the town-crust fix did (survey T14). Roads are not a generator change, so
no new generator version.

**12. Goodwill per town.** Each town keeps a goodwill number for the player
(a stored record), raised by jobs done and fair trade. Higher goodwill gives
better prices (up to 10%) and more jobs on the board. Theft and damage lower
it in the consequences change, not here.

**Order.** Coins first, then trade, then missions (they pay coins and use
trade), then roads (the delivery, escort and clear jobs want them). Each is
mocked up before code where it has a look: the road surfaces and slopes in a
new tab of the towns mockup (M15) and the board and trade panel in M5.

## Risks / Trade-offs

- [Moving solids in a contact index built for static pieces] → A person's
  circle is kept in a small list apart from the static index, since there are
  at most a few dozen per town. The walker checks both.
- [Figures pop in at the edge of range] → They fade with the settlement's
  pieces (`cities-in-the-world`, decision 6).
- [Frame cost] → Not measured in the cloud (CLAUDE.md). The town scenario in
  `perf_suite.py` covers it.
- [Coins inflate] → Every source of coins is a job or a sale to a town whose
  stock and demand are limited; the values file is one place to tune.
- [A road over a player's build] → A road cell is laid only on cells with no
  player edit; a road record that cannot route round one stops at it.
- [Sloped cells and the rest of the engine] → A slope is a top shape on an
  ordinary cell: the cell's material, light and saving are unchanged, and
  only the mesher, collision and the walker's ground height read the slope.

## Migration Plan

- There is no save change, because people are derived.
- Rollback is the previous build: the towns are empty of people again.
