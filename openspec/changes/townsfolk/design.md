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

**37. Paying with coins or with goods, and prices from a market (owner,
2026-11-22).** The owner: "you can either pay with cash or sell goods (will
need stock market to determine goods prices) raw materials dont go a smuch ,
butthey do relate to the goods porices in someway". Three rules, each a
recommendation taken (ask only with screenshots) where the owner left the
detail open:

- **A deal is a basket, settled in coins or in goods.** The trade panel puts
  what you take on one side and what you give on the other; what you give
  can be coins, goods or both. The trader values your goods at its buying
  price and its goods at its selling price (design 6's spread holds), and
  the difference is paid in coins, either way. A deal with no coins at all
  is barter. Each deal is one durable line: the pack, the purse and the
  town's `town_market` record together.
- **A made good's value comes from what it is made of.** Only raw materials
  (gathered, mined, caught, grown or herded: stone, log, fibre, ore, wheat,
  fish, egg) carry a base value in `values.ron`, and it is low. A good with a
  recipe is worth the sum of its inputs' values plus its labour (the
  recipe's seconds at a station rate) times a craft margin (recommended
  1.25), worked out from `recipes.ron` and the cooking recipes when the
  game starts, never typed in. So a plank is worth a little more than half a
  log, a bronze pickaxe much more than its ingots, and a raw material is
  always worth less than the goods made from it. A raw material's market
  price moves the goods made from it: dear tin makes dear bronze. A test
  holds every made good above its inputs. `values.ron` stays append-only:
  its rows for made goods stay readable and become overrides only where a
  row says so (none at first), so no shipped row is dropped.
- **Prices come from a market, per good, per world.** Each good has a market
  index, 1.0 at rest. Towns make and use goods each game day by their
  trades (design 6's need), and the player's sales and buys add supply and
  demand. The index rises when a good is short across the towns and falls
  when it floods, and drifts back toward 1.0 over days. A town's price is
  the good's derived value, times the market index, times the town's need,
  times goodwill (design 12). The market is a stored record per world
  (`market`, one row per good touched), written only on a trade or at a day
  boundary that moved a row, and world time passes only while the world is
  played. It is the price ledger the phase 6 stock market (M12) shows,
  charts and lets the player trade on later; phase 3 builds only the index
  and the rules, not the exchange's screens.

Saves: a world from before this decision has no `market` record and opens
with every index at 1.0; its `town_market` and `purse` records keep their
meaning. Prices shown in the trade panel will move from today's hand-set
values to derived ones, which changes no stored amount.

**37a. Design 37 as it is built (2026-11-24, on "cool implement all the
phase 3 stuff!").** Where design 37 left a detail open, each choice below
is a recommendation taken (ask only with screenshots):

- **Derivation** (`pbd_core::economy::derive`). A code that is an output
  of no recipe in the one book (`recipes.ron` then `cooking.ron`) is raw
  and keeps its `values.ron` row. Every other code's value is worked out
  at start, in passes until nothing moves:
  - a recipe's cost is the sum of its inputs' values times their counts,
    plus its labour, `labour_per_s` (0.1 coin, task 12.3) times its `secs`,
    times `craft_margin` (1.25); both live in `trade.ron`;
  - a recipe with several outputs splits its cost between them in
    proportion to their `values.ron` rows (the mixed-ore smelt's copper,
    iron and tin);
  - an output's value is its share over its count, rounded up to a whole
    coin and never under 1; so the outputs of a recipe are always worth
    more than its inputs, as rounded;
  - a good two recipes make (a fried egg from an egg or a fertile egg)
    takes the cheaper;
  - a recipe's input "fish" (any fish) is valued as the cheapest fish row;
  - **tools are not derived**: a recipe's `tool:` output is one tier of a
    tool whose value row is tier-less, and tools are `untraded`, so a tool
    keeps its row;
  - a row in `values.ron` stays an override only where `overrides` names
    its code (a new optional list in the file, empty today), so no shipped
    row is dropped and the file stays append-only.
  The book is fixed at start, so this is done once, then once a day with
  the market's index folded in (below).
- **The market** (`pbd_core::economy::market`, the stored record `market`,
  id 0, schema 1: the day it was last settled and an index per good that
  has moved, in a `BTreeMap` so it is written in one order).
  - Each unit the player sells to any trader lowers that good's index by
    `market.step` (0.005) as a share; each unit bought raises it; within
    `market.low` and `market.high` (0.5 and 2.0).
  - At each day boundary, every index moves back toward 1.0 by
    `1 / market.rest_days` (8 days) of its distance, then by the towns'
    making and using: `market.town_pull` (0.02) times the mean over every
    town the world holds of its need for the good's group less one. So a
    good most towns lack runs a little dear and one most towns make a
    little cheap, at a steady point near 1.0, and a flood the player
    sells recovers over days.
  - Settling is lazy and deterministic: reading the record steps it day by
    day from its last day to today (at most `market.rest_days` times 8
    days are stepped; past that every index is at its steady point within
    rounding). The game writes the record on a trade, and at a day
    boundary only if a row moved by more than a thousandth.
  - **Dear tin makes dear bronze**: the priced values a trader reads are
    the derivation run again with each raw value times its index and each
    made value's result times its own index, so an input's price moves the
    goods made from it. A town's price is then that, times its need, times
    goodwill, as design 6 and 12 already say; the town's own `town_market`
    factor (a day's local swing) stays on top.
  - A world with no `market` record reads every index as 1.0 and writes
    none until a trade or a moved day.
- **Barter in the trade panel** (task 11.3), as M5's Talk tab Basket view
  draws it: each row's button puts one in a basket (BUY rows into YOU
  TAKE, SELL rows into YOU GIVE), a basket's well takes one back out, the
  line under them says YOU PAY, THEY PAY or EVEN and the coins, DEAL
  settles and CLEAR empties both. The baskets replace the one-click buy and
  sell: one good for coins is a basket of one. A deal is the core's
  `Trader::deal` (all of it or none), counted in the town market, the
  world market, goodwill (one step per deal) and the taken jobs as the
  single trades were (each unit a buy or a sell). Its lines are the
  existing trade's: the town market, goodwill, the world market and the
  jobs, then the purse with the pack in one `with` line, so the pack and
  the purse never part. The words are locale keys (`trade.you_take`,
  `trade.you_give`, `trade.you_pay`, `trade.they_pay`, `trade.even`,
  `trade.deal`, `trade.clear`).
- **Fibre stands on Organics** (task 12.1): `values::shelf` puts fibre
  there by code (`ORGANIC_CROPS`), its `crop` group, and so its towns'
  need, unchanged.
- **The scroll thumb** (task 12.2) is 6 px wide inside the 8 px track (it
  was 4), so it can be caught at 1x.

**37a, framing (owner, survey P1, 2026-11-24).** The owner kept the
baskets as built ("Baskets only, as built") and added, in their words:
"One thing id improve is framing. Move the camera so the merchant is on
left, and the trade UI is on the right." The plan, built as written:

- **The panel sits on the right.** While Talk, Trade or Board is open, its
  layer right-aligns the panel (`justify_content: FlexEnd`) with a margin
  of `panel_margin_px` (24 native px, times the UI scale), still centred
  top to bottom. The panel keeps its native width at every window size, so
  it stays as readable as it was centred; on a window narrower than the
  panel it overflows to the left as a centred one overflowed both ways.
- **The view turns to the townsperson.** When the talk opens, the
  walker's own look (heading and pitch, `WalkingState`) is turned so the
  townsperson's aim point, `aim_height_m` (1.2 m, between chest and face)
  over their feet, lands at `frame` in normalised screen space: (-2/3, 0),
  the centre of the left third, level with the middle of the screen. The
  turn is solved exactly for a camera with no roll, from the camera's own
  vertical field of view and aspect, about the up the walker already
  carries (the origin's `up_at` in f64); the eye-to-townsperson vector is
  a difference of two render-space points, never a planet-centred
  position. The look stays where it was turned when the talk closes: the
  player has turned to face the merchant, nothing more, and no camera of
  the talk's own exists.
- **Eased, not cut.** The turn runs over `turn_secs` (0.25 s), smoothstep,
  heading about up and pitch together. This does not touch the raw-look
  rule: while a talk panel is open the menu holds the pointer and the
  walker reads no mouse at all, so nothing smooths a mouse; once the panel
  closes the look is raw again from wherever the turn left it. A
  `turn_secs` of 0 turns at once.
- **The framing is the one from when the talk opened.** A townsperson who
  walks on while the panel is open is not followed: the aim is taken once,
  at the press, and the turn runs to it.
- **Tunables** are in `assets/config/talk.ron` (`TalkView`), read at start
  and validated: `turn_secs` 0 to 2, `frame` inside the screen, the aim
  height 0 to 3 m, the margin 0 to 200 px.
- **Proof.** A unit test (`the_talker_lands_in_the_left_third`) solves the
  turn for eyes and townspeople above, below and level, on a tilted up and
  at three aspects, builds the camera the walker builds, projects the aim
  point through Bevy's own perspective, and finds it in the left third at
  the screen's middle height. The capture is
  `docs/screenshots/townsfolk-economy/trade-framed.jpg`.

**37a, framing, second pass (owner, 2026-11-24, on `trade-framed.jpg`).**
In the owner's words: "Kind of center the character between the left most
edge of panel and left edge of the screen. Also if you can add some lights
that fade in light the character a bit, thatd be nice". The plan:

- **Centred in the open space, not a fixed third.** The townsperson's aim
  point lands half way between the screen's left edge and the open panel's
  left edge. With the window `W` logical px wide, the UI scale `s` and the
  panel `w` native px wide at `panel_margin_px` `m` from the right, the
  panel's left edge is at `W - (m + w) s`, the open space's middle at
  `(W - (m + w) s) / 2`, and in normalised screen space
  `x = -(m + w) s / W`. It is worked out from the real numbers at the
  moment of framing: the panel's own width (Talk 300, Trade 490, Board 450
  native px, one table the panels are spawned from), the camera's viewport
  and `UiScale`. A panel wider than the screen puts the townsperson no
  further left than -0.95. The height stays `frame_y` (0, the middle) in
  `talk.ron`, which replaces `frame`.
- **Re-aimed when the panel changes.** TRADE or WORK from Talk opens a
  wider panel, so the open space's middle moves left; the look turns again
  to the new middle with the same `turn_secs` ease, from wherever it is,
  toward the same aim taken when the talk opened (the townsperson is still
  not followed). BACK re-aims to Talk's middle.
- **A light on the townsperson.** While a talk is open a soft warm light
  burns in front of and a little above the townsperson's face, between
  them and the camera: from their feet, `light_rise_m` (1.75 m) up and
  `light_toward_m` (0.6 m) along the level line to the eye. It is not a
  Bevy light: it is a carried light (`glowing-mushrooms` part B), the short
  list the terrain, the water and the field-lit shaders already evaluate
  per pixel with the falloff every baked lamp uses, so it lights the
  townsperson (who is `LitByField`) and the ground round their feet exactly
  as a torch in hand would, at night and in shade alike, with no shadow, as
  every carried light. The list gains a reach per light (the shaders
  already read it from `w`; every other light keeps the lamps' 16 m), so
  this one reaches `light_reach_m` (4 m) and goes no further.
- **Fades in and out.** Its strength rises over `light_fade_secs` (0.4 s)
  when the talk opens and falls over the same when it closes, linear in
  the light's colour, which the shaders take as a uniform every frame, so
  the fade is smooth (a baked lamp could not do this; a carried light can).
  It is placed from the townsperson's own transform each frame, after the
  transforms are propagated, in render space, and handed to the list in the
  planet's frame by `render_origin::local_of` (f64 before the cast); up is
  the walker's camera's up, never a normalised planet-centred position.
  Nothing of it is stored or saved.
- **Modest.** `light_level` 7 on the field's 0..15 scale (a torch is 14),
  `light_tint` a candle's warm white: it lights the face a bit; it is not a
  spotlight. It does not count as the player's own light, so it never
  warms the held hand.
- **Tunables** in `talk.ron`, validated: `frame_y` inside the screen,
  `light_tint` each channel 0 to 1, `light_level` 0 to 15, `light_reach_m`
  0.5 to 16, `light_fade_secs` 0 to 2, `light_rise_m` 0 to 3,
  `light_toward_m` 0 to 2.
- **Every NPC dialog, one presentation (owner, 2026-11-24).** In the
  owner's words: "same presentation style for dialog boxes of any kind of
  interaction with npcs". The panel on the right, the townsperson centred
  in the open space left of it, and the talk light on them are one
  presentation a screen opts into, not a copy per screen: `talk.rs`'s
  `panel_width` is the table of NPC dialogs and their widths, and a screen
  in it is spawned by `npc_dialog` (the right-aligned layer and its framed
  panel), framed by `turn_to_talker` and lit by `talk_light`, all three
  reading `is_npc_dialog`. A searched list of what an NPC opens today: the
  interact key's only NPC act is Talk (`Act::Talk`, `interact.rs`), and
  every other NPC exchange lives inside its three panels: the trader's
  goods and the horse for sale (Trade), the town's jobs, taking, handing in
  and abandoning them, the town missions' board jobs and the lost villager
  (Board, and the line said in Talk when the villager is found), the
  parcel handed over on talking (Talk's line). The starter missions and
  the mission tree (O) open from the keyboard with no one to talk to, so
  they are not NPC dialogs. A new NPC dialog joins `panel_width` and gets
  the whole presentation.
- **Proof.** `the_talker_lands_in_the_open_space` solves the turn for each
  panel's width at three window sizes and scales and projects the aim point
  to the open space's middle; `the_talk_light_burns_only_while_a_talk_is_open`
  runs the light's system in an app and finds the light in the carried list
  while a talk is open, rising, and gone once it has faded after the close.
  Captures (lavapipe, no GPU): `trade-framed.jpg` retaken at a talking
  distance of about 2.7 m, `trade-framed-night.jpg` at nightfall, and the
  talk panel itself, a non-trade dialog, as `talk-framed-night.jpg`. Not at
  22:30 as first planned: from daylight 0.12 (about 20:32 at Linenleigh on
  day 0) every townsperson but a watch goes to bed (`furniture` design 11a),
  so at 22:30 there is no merchant at the stall to talk to. The night shots
  are at 20:27 (`--time 20.45`), the sun about two degrees under the
  horizon and the dusk lamps lit, the latest the merchant keeps the stall.
  The reeve's board was tried from 2.7 m too, but stepping back from the
  reeve puts a wall between them and the eye on today's town, so the
  non-trade capture is the merchant's own talk panel.

**10a. Roads as they are built (2026-11-21, on "People approved. Build.phase
3").** The look was built in the towns mockup first (M15's Roads tab,
`towns.html#roads`, task 8.1): a dirt track, a gravel road, a hill road
climbing ten layers a layer a cell with one bend, a ford and a timber
bridge, by day and lit at night by lanterns on posts; the mockup's walker
climbs the hill road with no eye jump. The owner asked for the game to
follow straight on ("Build phase 3"), so the game was built from the tab
without waiting for its approval; the tab is on the tracker for it. The
owner approved it on 2026-11-22: "roads look great, can you also show
roads of width 2 or 3?" (wide roads: decision 10b).
- **A road is a piece on an ordinary cell, not a ground edit.** The cell's
  column is the generator's, untouched; the road top is cut like a town's
  street step: its own mesh 2 cm over the ground (`roads::piece::LIFT_M`),
  its side faces down to the cell's ground, and a floor for the walker
  (`settlement::pieces::Surface::Steps` for a slope, `Floor` for a flat or
  raised cell or a deck) in a frame of its own, met through the same
  `walking` floors a town is. So `planet/contact`'s "steps are reported as
  steps" holds for the terrain, and a road's slope is a separate, explicit
  shape (a new requirement in `world/roads`). No save format changes and no
  generator version moves.
- **The slope's plane** runs from the middle of the side the road comes in
  by, at the cell's ground, to the middle of the side it climbs out of, a
  layer up, held level past each: one layer over 2.83 m straight through
  (19 degrees), over 2.45 m round a 60-degree bend (22 degrees). It goes on
  the lower cell of each one-layer pair. A cell lower than both its
  neighbours in the road (a dip one cell wide) would need two slopes, so it
  is raised a layer on a bed of packed earth instead (`RoadCell::raised`).
- **The route** is A* over the finest cells by their exact keys
  (`roads::neighbours`, on every face a key's point lies on), costed in
  millimetres so the order is total: a cell's length, plus
  `slope_cost_m` for a layer up or down; never two layers; shallow river
  water (`ford_depth_m`, 1 m) forded at `ford_cost` times the length on its
  bed; deeper water bridged at the bank's height for at most
  `max_bridge_cells`, at `bridge_cost`; never the sea; never a cell holding a
  player's edit or road cell. A town's footprint is crossed and not laid
  (its lanes are the road there). Tunables in `assets/config/roads.ron`.
  The search is weighted A* (`search_weight` 1.5: a road at most half as
  long again as the cheapest, for far fewer cells looked at), capped at
  `max_expanded` cells a link, and the links are routed side by side on all
  but one of the machine's threads, given back in link order. Measured on
  the shipped seed in this container (4 cores, `--profile fast`,
  `roads::tests::the_shipped_seed_network`): 53 settlements (the cave
  towns left out: their anchor is under the mountain), 57 links, 27 routed,
  11,730 cells, in 3.2 s; with the plain cheapest search and 600,000 cells a
  link it was 35 routed in 59 s, too long for a loading screen. The links
  not routed have no way that climbs at most a layer a cell, keeps off the
  sea and bridges at most ten cells within the search's reach; a later
  change may cut and fill (world edits through the yield check) to reach
  them.
- **Stored records**, authored by the world (`@wroads:road-net/0`) and
  passed through the yield check: one `road` record a road (id: its two
  sites, the cells compact), and `road-net` id 0 once the plan is made,
  which is what keeps a world from being planned twice. Planned in the
  loading thread after the towns' ground is installed, so an old save gains
  its roads on its next open, in place, with no restart. A player's road
  cell is a `road-cell` record of the player's, by its cell key.
- **No tree grows on a road**: the road cells are installed beside the
  towns' ground before the planet is built (`roads::Cleared`), and the
  foliage pass's cleared bit is set on them as on a town's footprint. The
  first in-game capture had trees standing in the track. Grass tufts are
  still drawn on a road (not yet cleared).
- **Laid only where no player edit stands, at every build**: a stored
  road's cell that the player has since edited is not drawn or stood on,
  so the road yields however late the edit came.
- **Gravel is walked `gravel_speed` (1.15) times the ground's**, dirt
  `dirt_speed` (1.0), read off the road piece under the walker's feet.
- Frame cost not measured (cloud session, lavapipe; CLAUDE.md).

**10b. Wide roads (2026-11-22, on the owner's "roads look great, can you
also show roads of width 2 or 3?").** Shown first in the towns mockup
(M15 version 3, the Wide roads scene, `towns.html#wideroads`); not built in
the game (task 8.6). A road two or three cells wide is two or three lanes of
road cells side by side, each lane an ordinary road cell of design 10. On
the level that is all there is to it. On a slope it is not, and the reason
is the grid:
- **Why the one-wide rule cannot be copied lane by lane.** A road running
  along a row climbs a layer a cell, 1 m over 2.833 m. The next lane is the
  next row, shifted half a cell along the road, so at the same place along
  the road it is half a layer higher or lower. Lane by lane, each cell
  running from its own entry edge at its ground to its exit edge a layer up,
  the two lanes' tops meet at their shared slanted edges with a step of up
  to half a layer, and a walker crossing from one lane to the other gets an
  eye jump. Turning the slope off the row's axis does not help: a plane that
  puts every cell of a hex band at the same height above its column spans
  4/3 of a layer over a cell, never one (the hex's corners, in the lattice's
  own terms, sit a third and two thirds of a cell along both axes).
- **The rule: one shared plane for the whole width.** Every cell of a wide
  road on a slope takes its top from one plane along the road, rising a
  layer per cell pitch (2.833 m, 19 degrees, the one-wide road's own grade),
  held level past the foot and the top of the climb (a crease across the
  cells that straddle them, which is drawn as two faces). Its column is cut
  or filled to the whole layer `low` under the plane's lowest point on the
  cell, and the road piece's bed of packed earth fills from `low` to the
  plane. The surface is the same function on both sides of every edge, so
  there is no step between lanes, across or along, by construction.
- **What it costs, measured in the mockup.** Every other lane is half a
  layer off its cells. A lane in step with the plane (its cells' middles at
  a half layer) is exactly the one-wide road cell: the plane spans
  `[low, low + 1]`, no cut, no fill. A lane half a cell along spans
  `[low + 0.5, low + 1.5]`: its natural ground is `low + 1`, so its column is
  cut a layer and the bed is 0.5 to 1.5 m deep; against the natural ground
  the road there is cut half a metre at the cell's downhill end and filled
  half a metre at its uphill end. A two-wide road has one such lane, a
  three-wide road two (the outer lanes; the middle lane is in step). Beside
  the road the hillside keeps its own steps: where it stands higher (a
  side-hill's uphill side) its cut bank shows as earth faces down to the
  road; where it stands lower the road's bed shows as an embankment.
  Walking across all three lanes diagonally up the climb and back gives no
  eye jump in the mockup's walker (numbers in `mockup-work.md`'s M15 row).
- **Bends only on the level** (recommendation taken, ask only with
  screenshots). A 60-degree bend of a wide road is a rhombus of cells where
  the lanes cross over: the inner lane turns in one cell, the outer lane in
  `w` cells, so a turn of the whole width on a slope would need a warped
  surface (each lane a different grade) rather than a plane, and the inner
  lane would climb at up to `w` times the grade. A wide road therefore climbs
  straight and bends on a level stretch: at the foot, on a landing, or at
  the top. The one-wide road keeps its bends on the slope (22 degrees round
  a bend, design 10a). The mockup shows the two- and three-wide roads
  climbing straight and bending on the plateau, one each way.
- **Junctions are level.** Where a narrow road meets a wide one the cells
  where they touch are level road cells: the narrow road meets the wide
  road's edge lane, and its last cell before the wide road is flat. A road
  never joins another on a slope, for the same reason a wide road does not
  bend on one. The mockup shows a one-wide road leaving a three-wide road in
  a T and crossing a two-wide road.
- **Bridges.** A wide bridge is a plank deck over every cell of the width at
  the banks' height, on posts to the bed, with rails only on the outer edges
  of the outer lanes; the lanes between are open to each other.
- **Lamps.** Lanterns on posts off the road every sixth cell: one side of a
  one- or two-wide road, both sides of a three-wide road (staggered), at
  both ends of a bridge and at a junction. Lit at night as every road is.
- **Which world roads are wide** (recommendation taken, ask only with
  screenshots): the gravel roads between the largest walled towns are two
  wide; inside a large town's gate, to its market, three wide; village
  tracks and every other road stay one wide. The planner (design 11) routes
  a wide road's middle line as today and lays its lanes either side, with a
  wide road's slopes allowed only where its line runs straight along a row
  and its bends only where the ground is level.
- **The player** lays any width by laying road cells side by side (task
  8.3): a player's lane on a slope beside another takes its top from the
  same rule, the shared plane of the cells it adjoins, so a road the player
  widens has no step either.

**Order.** Coins first, then trade, then missions (they pay coins and use
trade), then roads (the delivery, escort and clear jobs want them). Each is
mocked up before code where it has a look: the road surfaces and slopes in a
new tab of the towns mockup (M15) and the board and trade panel in M5.


**12a. As built (2026-11-21, on the owner's "People approved. Build.phase 3").**
The decisions above, made concrete where they left a choice; each is a
recommendation taken (ask only with screenshots):
- **One authority.** `pbd_core::economy` holds every rule: `coins` (the
  purse and `Purse::change`, the one function, with its reason), `values`
  (`assets/config/values.ron`, `(code, coins, group)` rows in shipped
  order), `trade` (`assets/config/trade.ron`: the need table by town kind
  and group, the 0.6 share, the 2% step within 0.5 to 2.0 of rest, a day to
  rest), `goodwill` (in `trade.ron`: +1 a trade, +10 a job, 100 at most,
  steps of 20, 2% a step, a fourth and fifth job at steps 2 and 4), `jobs`
  (`assets/config/jobs.ron`: the board and the taken jobs) and `follow` (the
  lost villager's step). `pbd-app` only feeds them and stores the records.
- **Item codes.** A good by its code, `block:<name>`, `tool:<code>`, and a
  fish as `fish:<species id>` (a fish in the pack is its roster index).
  Coins are whole: a trader's asking price rounds up, its paying price down,
  and the paying price is always at least a coin under the asking one, so
  buying and selling back always loses. A town sells the groups it makes
  (need 0.8 or less) and bread everywhere; it buys every valued good but a
  delivery's parcel, wanting twelve of each at size 1 (a walled town is
  size 2, a harbour 1.5), and stocks eight of what it sells.
- **The town jobs' things are scoped.** A job's objective names
  `<site>:<thing>` where only that town's counts (a sale to its trader, a
  parcel handed to its people); `Want::Prefix` (`<site>:*`) lets "sell six
  goods here" count anything sold there. The new kinds join
  `missions::Kind` and are counted by `Progress::record`, the starter
  missions' own function: the taken jobs are a chain of their own and keep
  their counts in a `Progress` inside the `town_jobs` record.
- **The board.** Each place on it keeps its job for three days, the places
  changing on staggered days; the first place is the town's own trade (a
  harbour fishes, a mountain town mines, a village harvests). A job's id is
  a hash of the world's seed, the town, the place and the period.
- **Talking is the interact key (F)**, the key every door and station uses
  in the game today (`interact.rs`), not M5's E, which predates it. Talking
  to anyone in a town hands over a parcel owed there. A town's trader and job
  giver are its people's roles, first match in `jobs.ron`'s
  `trader_roles` and `giver_roles`; a town with none of them trades and
  gives jobs through its first person, so every town has both.
- **The lines a deal writes.** A trade writes the market and goodwill, then
  the purse with the slots in one `with` line; handing in writes the purse,
  the goodwill and the jobs, in that order. A torn write costs only the last
  line (`saves::format`): a price moved with nothing bought, or a job to hand
  in again, never coins or goods lost.
- **Lamps, lost villagers and repairs** are made and counted by the rules
  (`JobKind`, `follow`) and tested there; a board posts them only once the
  game raises their events (`jobs.ron`'s `posted`), since the lamp's F, the
  villager's figure and the world's broken piece are not in the game yet.

## Friendly faces (research, 2026-11-21)

**The owner (2026-11-21):** "Let's start with npcs using existing NPC
mockup. Design new and friendly looking NPCs. High frontier repo has some
methods for drawing friendly faces. Ultimately I want you to do research on
how we can generate and animate interesting looking cartoony low poly people
withe expressive faces. Think of like kerbal space program".

### What the references do

- **Kerbal Space Program.** A kerbal is a skinned mesh with a head about a
  third of its height, a short round body and short limbs. Its face reads
  from a few large features: two big round eyes that move on their own, lids
  for blinks, and a wide mouth whose shapes (grin, gritted panic, "wheee")
  are blend shapes (shape keys) on the head; the community's Blender rigs add
  eyelids and eye controllers for the same reason. Expression follows state
  and trait: the same face panics under G load or grins, and KSP 2 makes eye
  shape a character trait and adds hair and more expressions
  ([forum, blend shapes](https://forum.kerbalspaceprogram.com/topic/173577-animated-meshes-blendshapes-shapekeys-keam-inside-look),
  [KSP 2 dev insights 6](https://forum.kerbalspaceprogram.com/topic/200600-developer-insights-6-%E2%80%93-kerbals-the-first-new-look),
  [space.com](https://www.space.com/kerbal-space-program-2-new-features-video)).
- **Wind Waker.** Eyes, brows and pupils are separate flat meshes floating
  just off the face, drawn after the head and hair so they always read, and
  swapped or slid to change expression; the mouth is drawn normally
  ([Link's expressions](https://medium.com/@gordonnl/links-expressions-eb7beae2c62c),
  [Polycount](https://polycount.com/discussion/comment/1655607)).
- **High Frontier fan game** (`character-kit` skill and its mascot, read
  2026-11-21 at `6cbe5b3`): flat colour, two tones per material, no
  outlines; a simple face of dot eyes with a white catch-light, short brow
  strokes, a small mouth and optional blush; **expressions are swaps of
  eyes, brows and mouth, never a deformation** ("happy" eyes are two upturned
  arcs); hands and heads are swaps; round joints so no seam shows.
- **Eye motion research.** Blinks every 2 to 6 s with an occasional double,
  closing faster than they open; saccades as fast moves (about 50 ms) to a
  new target, then a hold; lids drop a little when the eyes look down; blink,
  gaze and mouth kept as separate layers so none overwrites another
  ([Eyes Alive](https://www.cis.upenn.edu/~badler/papers/EyesAlive.pdf),
  [CMU blinks](https://ri.cmu.edu/?p=121999),
  [facial animation guide](https://abratabia.com/ai-talking-characters/facial-animation.php)).

### Decisions (proposed; the mockup shows them)

**13. A friendly townsperson: KSP's proportions in the game's hex style.**
- **Body:** about 3 heads tall (a kerbal is close to that), a head 0.45 m
  across on a short rounded body 1.35 to 1.6 m tall in all, stubby arms and
  legs. Built from six-sided pieces (hex prisms and six-sided lathes, flat
  shaded) so it keeps the owner's "hexagonal blocks that look like cubes"
  (survey T2) while reading round at a distance. Low poly: about 600
  triangles a person with the face.
- **Lit like the terrain** (CLAUDE.md): `LitByField` and `LitLikeTerrain`,
  no outlines, clothes in two tones from the town's palette.

**14. The face is layered pieces on the head, swapped and slid (Wind
Waker and High Frontier), not a deforming mesh.**
- Eyes: a white six-sided disc per eye with a dark pupil disc and a white
  catch-light, sitting just proud of the face. The pupil slides inside the
  white to look; the whole eye scales for size (a trait, as in KSP 2).
- Lids: a skin-coloured cap over each eye that slides down to blink and
  rests lower when sleepy or looking down.
- Brows: one short bar per eye, raised, lowered and tilted for mood.
- Mouth: a small set of drawn shapes in one atlas (smile, grin, open "o",
  flat, frown, wobble, talk A/O/M), picked per frame. Blush: two soft
  ovals, on when happy or embarrassed.
- Pieces are drawn in pixel art at the terrain's texel density, sampled
  nearest. A swap is a UV change on one instanced quad, so a face costs a
  handful of quads and no blend shapes; it works the same in the game and in
  a three.js mockup.

**15. Expressions are named sets of those layers, driven by state.**
- `neutral`, `happy`, `delighted`, `surprised`, `worried`, `sad`, `cross`,
  `sleepy`, `thinking`, `talking`; each is a choice of eye scale, pupil
  size, lid height, brow lift and tilt, mouth shape and blush.
- What drives them: meeting the player (surprised then happy), a fair trade
  (happy), a job done (delighted), a lost villager (worried, then delighted
  at home), night (sleepy), the smith at work (focused brows), rain (a
  squint). Goodwill shifts a town's resting face.
- Layers run separately so none overwrites another: blink (random 2 to 6 s,
  sometimes double, close 60 ms, open 120 ms), gaze (saccades to the player
  when near, else to points of interest, held 0.5 to 2 s; head follows the
  eyes late), mouth (expression or talk), brows (expression plus a lift on
  each saccade).

**16. People are generated from a seed and the town.** Height, build (round,
pear, tall, stout), head size, eye size and spacing, nose (none, button,
long), ears, hair from a kit of hex-built styles (bun, crop, braid, bald,
hat), beard, skin, and clothes in the people's palette; each picked from the
settlement record and the person's index, so they are the same on every
visit. A generator tab in the mockup shows the range.

**13a. Version 2 of the figure (owner, 2026-11-21, on M5 version 8):** "Faces
are kinda small. Id like blocky heads maybe taller head as well. Characters
looking great. Try rendering fps to give me a feel for height".
- The head is a block: a box with a small bevel (hex-flat sides kept only
  where they read as a block), taller than it is wide (about 0.5 m wide,
  0.6 m tall, 0.45 m deep), so the figure is about 1.55 m with the body kept
  as approved.
- The face fills the front of the block: eyes, brows and mouth about 1.5 to
  1.8 times their version 8 size, the eyes set wider and lower, the mouth
  wider, so the face reads from across a street.
- A first-person view in the tab at the game's own eye (1.6 m,
  `walking::EYE_HEIGHT`; a 60 degree vertical field of view as the game's
  camera), walked with WASD and mouse look, so the townsfolk's height is
  felt as the player meets them.

**17. Motion is procedural on a few bones.** A bouncy walk (body bob and
squash on each step, arms swinging), idle sways and weight shifts, a hop
when delighted, a shrug, a wave, a head tilt when thinking; keyed poses
blended by curves with named tunables (the `animation` skill), and
nothing read by a rule.

**18. The look is decided in M5 first.** A Faces tab in `npcs.html` puts
today's hex figure beside the proposed one, with the expression set, the
live layers (blink, gaze at the cursor, talk), the generator, and a lit
night square. The game waits for the owner's approval of that tab.

## In the game (2026-11-21, tasks 1 to 3 and 10.4)

**The owner (2026-11-21):** "People approved. Build.phase 3. Do it in
parallel if you can". The approved look is M5 version 9's Faces tab
(`docs/mockups/npcs.html#faces`, design 13a); it supersedes decision 2's
hexagonal-prism figure for the look. Decisions 1, 3 and 4 stand. Coins,
trade, missions, the lost villager and roads are built beside this by other
hands and are not part of it; what they need from the people is decision 27.

**19. The people come from the mockup's own `person()` calls.**
`tools/export_town_templates.js` records each call a scene makes, as it
records `building()`, into the template's `people`: a role id (the mockup's
role, lower case with underscores), where it stands (`at`, the mockup's
metres, its feet's height), the way it faces (radians from the mockup's
east toward its south), its tunic, its work (`hammer`, `wipe`, `sit`,
`idle`) or its path and pace, and whether it holds a spear. Nothing else in
the templates changes (the jungle's file is spliced so its kapok angles keep
their old rounding). What the templates hold, re-exported 2026-11-21:

| Template | People | Among them |
| --- | ---: | --- |
| `village` | 5 | a smith hammering, three villagers and a farmer walking |
| `town` (walled) | 19 | the smith, the innkeeper, four stallholders, a gate guard, a lookout, two wall guards on the wall walk |
| `coast` (harbour) | 12 | a boatwright hammering, four fishwives, fishers and a porter |
| `desert` | 6 | a caravan master, four traders, a water carrier |
| `tundra` | 4 | a watcher on the ice keep, two hunters, a wall guard |
| `jungle` | 3 | a forager, a tree dweller, a lookout |

**Task 1.1 said 16 for the walled town; it holds 19.** The mockup's market
puts four stallholders round its stalls (the count of 16 was taken before
they were added), and its other people are the ones the task names. The
test pins 19, with a smith and a gate guard among them.

**20. `pbd_core::townsfolk` derives a town's people from its record.**
`people_of(town, template, chart, patch)` gives each person, in the
template's order:
- an id (the site and its index), its role, its work or its path;
- its post as a direction on the planet, placed by `sea::point` as the
  boats and the dressing are, so it follows the sphere's cells as a
  building's corners do; its height is the town's terrace plus the
  template's height over its datum, as a lantern's is;
- its facing as a tangent there; a path as a polyline of directions, the
  mockup's straight line sampled every half metre through the same chart;
- its look from the generator (decision 16, the mockup's `genSpec` ported
  number for number, its xorshift too), seeded by the site and the index.
  The people's palette is the town's: the village, the walled town and the
  jungle wear the fields folk's (M5 has no jungle people; recommendation
  taken, ask only with screenshots), the harbour the harbour folk's, the
  desert the caravan folk's and the tundra the ice-camp folk's. Each
  person's shirt is the template's tunic, so the guards keep their grey and
  the smith his dark; trousers, skin, hair and the rest are rolled.
- a path stops half a metre short of the first wall it would cross,
  walking out from the person's post toward each end. The mockup's people
  walk through what was built after their lanes: its village lane walker
  goes from (25, 9) to (25, 27) straight through the village hall, which
  stands on (25, 10) and (25, 11). In the game that villager turns back at
  the hall's south wall. A path that crosses nothing is kept whole, and a
  test holds every path to more than half its length.
Nothing about a person is saved: it is derived, as the masonry is.

**21. The figure is M5 version 9's `friend()`, ported.** The same pieces
and numbers: the bevelled block head 0.5 x 0.6 x 0.45 m (`HC` 0.035) at
the given head scale, six-sided lathes for the torso (the four builds'
profiles), legs, boots, sleeves, cuffs and hands, ears, button and long
noses, the hair kit (crop, bun, braid, bald, hat) and beards as blocks; the
face 2 mm proud of the head's front: eye whites (r 0.08), pupils (0.048)
with catch-lights (0.016), lids and lash lines, brows, the mouth (0.24 x
0.12 m) and blush.
- **One texture, one material.** The field-lit shader reads a mesh's
  vertex colour as the town's bake, never a tint, so a person's colours are
  texels: one small atlas holds a swatch for every colour a person can
  wear (each palette's colours and the shades the mockup takes of them:
  0.84, 0.78, 0.74, 0.72, 0.62, the blush mixes) and the mockup's mouth
  atlas, drawn pixel for pixel as its canvas draws it. Every part's UVs
  point at its swatch, sampled nearest. So every person, of every town, is
  drawn with one opaque material and the mouths with one masked one: two
  pipelines.
- **A rig per drawn person**: a root with the body, two legs, two arms and
  the head as pivots, each part one mesh built from that person's look;
  the face pieces are shared meshes (unit discs and quads, scaled). A mouth
  shape is a swap of the shared quad that carries that cell's UVs, the
  game's form of the mockup's UV change.

**22. Expressions are a state machine in `pbd-core`, fed by the game.**
`townsfolk::face` holds the ten expressions with the mockup's numbers and
`Mood`, which picks one from what happens:
- the player comes within 5 m: surprised for 0.5 s, then happy for 6 s,
  then back to the resting face; met again once the player has been 8 m off;
- night (the sun's daylight under 0.12 where the person stands, as the
  herd's night is): sleepy, unless just met;
- the smith (any `hammer` worker) at work: the resting face with the brows
  drawn down ("focused"), not an eleventh expression;
- otherwise the resting face, happy or neutral from the seed;
- one set from outside (decision 27) wins over all of them while it holds.
The layers run on their own as the mockup runs them: blink (2 to 6 s,
one in five doubled, 60 ms closing, 30 held, 120 opening), gaze (saccades
held 0.5 to 2 s; to the player's eye while within 5 m, the head following
late, at most 0.6 rad), talk (A, O, M cycling, 80 to 170 ms each) and the
brows' lift on a saccade. Each person rolls its own sequence from its seed,
never the global random.

**23. Motion is the mockup's, on the CPU.**
- A walker goes back and forth along its path at its pace, as the towns
  mockup's `animateNpc` does, its start in the cycle from its seed, with
  the Faces tab's walk: a bob and squash on each step, the legs and arms
  swinging against each other, the steps' rate following the pace. It
  turns at the ends over 0.12 s.
- A worker works in place: the hammer stroke and the wipe of the towns
  mockup on the right arm; `sit` is drawn standing at its post, as the
  mockup draws it. An idler sways and looks about (the gaze layer).
- The actions wave, hop, shrug and tilt are keyed poses blended by the
  mockup's curves; delighted plays the hop.

**24. At most 32 people are drawn, within 60 m, faded over 0.8 s**, through
`Faded` as the critters are; the rest of a town's people still stand and are
solid. A rig is made when its person comes into range and dropped when it
has faded out. The two materials are prewarmed behind the loading screen by
one proxy figure, as the critters' are.

**25. Every person of a standing town is a solid circle**, 0.28 m round
from its feet to 1.9 m, in a list apart from the towns' pieces
(`TownsfolkSolids`), updated each frame from where each person is; the
walker's refused-move test and its slide ask it after the pieces, in the
planet frame in f64. A body already overlapping a person (one walked into
it) is let walk out, as with a wall built round it. People do not push the
walker.

**26. Lit by the field.** Each rig's root carries `LitByField` and
`LitLikeTerrain`, so a person is lit as the ground under them, by the sun
with its shadows and the cell light field's sky and lamps. **The forge is
not in the game yet**: the towns' cut has no smithy shed or forge fire in
any template (they are the mockup's dressing, not a `building()`), so task
3.2's "the side facing the forge" waits for the forge's light; at 22:30 the
smith is lit by the town's lamps and the field. Recorded as a limit, not
faked with a light of its own.

**26a. The forge (2026-11-24, task 3.2).** Written before the code.
- **Where it is.** The towns mockup builds a smithy in two towns: the
  village (`smithyShed(26, 22)`) and the walled town (inline, cells 28 to
  30 by 8 and 9). Each is an open shed two rows by three cells, six timber
  posts (r 0.15 m, 3.2 m) under a shingled gable roof (pitch 0.7, timber
  gables), a forge at the far end of its second row (a stone base 1.6 by
  1.3 m, a hearth bed, a stone hood 1.3 by 1.0 m up to the roof), its fire
  at `addFire(..., "forge")`, and an anvil (two iron blocks) by the smith.
  Neither went into the templates: the export recorded `building()` and the
  dressing, and the smithy is neither. `tools/export_town_templates.js` now
  writes each forge as the template's `smithies`: its fire's point and the
  cells of the area it burns in. Re-exported, every other byte of the six
  templates is as it was (the jungle's file is kept as spliced).
- **New towns only.** A town's pieces are derived from its template on
  every load, so a forge read from the template would stand up in every
  saved village and walled town, in cells a player may have built in. The
  forge is therefore a new layout: `LAYOUT_VERSION` becomes 3
  (`FORGE_LAYOUT`), a town laid from now on is stored at 3 and gets its
  smithy, and a town stored at 1 or 2 is cut exactly as before, with no
  smithy and no forge lamp. A test pins that a layout-2 town's pieces and
  lamps are those of its template with the smithies taken out, and that
  layout 3 adds exactly the shed and one lamp. The ground of a layout-3
  town is a layout-2 town's (it finds its cells by chord), so the ground
  digests stand.
- **What is cut** (`settlement::pieces::smithy`), one piece on the forge
  cell's terrace, in the settlement set's textures (timber, shingle, stone,
  iron), solid to the walker where the mockup's are solid (the posts, the
  forge, the anvil):
  - the posts are tapered six-sided posts with smooth normals, as the
    scaffold's are;
  - the roof is two shingled slopes over the shed's corners with a 0.3 m
    overhang, and timber gables, as a house's gable is;
  - the forge is one stepped stone mass, not a box with a box on it: a
    base, a hearth bed sunk into its top, and a hood that tapers from the
    base's back into a chimney through the roof;
  - the anvil is one lofted iron shape: a waisted foot, a face and a horn
    tapering off one end.
- **Its glow is a lamp in the voxel field.** The forge's fire is a
  `Brazier` lamp (the brightest there is, always lit) in the first air
  layer of the forge's cell, put in by `lamps_of` as a desert brazier is.
  So the field carries it to everything near, by night and day: the smith
  (`LitByField`), the shed, the ground and the houses round it, wherever
  the camera is. The forge is centred on that cell so the brazier's own
  drawn basket stands inside the forge's base, and only its fire shows,
  over the hearth bed, under the hood. The game's forge sits a little off
  the mockup's point (0.67 m, to the cell's middle): recommendation taken
  (ask only with screenshots).
- **The smith** keeps his post by the anvil from the template; nothing
  about him changes. Task 3.2's capture is the smith at the forge at 22:30.
- **The fire and the anvil redone (2026-11-24), replaced by 26b on merge.**
  Written before the code.
  The first cut broke the owner's art rules, plainly in
  `smith-forge-2045.jpg` and `forge-day.jpg`:
  - *The fire was five pale cones* (each `Sink::flame`'s three crossed
    blades) standing apart on the hearth: obvious primitives, untextured, one
    flat colour. It is now **one flame form fitted to the bed**: a crown
    lofted from an ellipse just inside the bed, tapering in as it rises, its
    rim cut into tongues of uneven height that share one base, with a taller
    core of fewer tongues inside it and a bed of coals under both. It is
    drawn in a new pixel sheet, `forge_fire` (eight 16 by 32 px frames,
    drawn by `tools/art/forge_art.py`, with its manifest row), whose ramp is
    the terrain brazier's own two fire colours (`planet_surface.wgsl`:
    white-gold 0.98, 0.66, 0.22 and orange 0.86, 0.28, 0.05) with a deep coal
    red under them, red at the root to yellow at the tips, every colour kept
    under the tonemapper's shoulder so it stays a fire at night, not a cream
    cone. Each frame's alpha cuts the tongues' ragged edges, so the sheet
    stepping at the held flame's 11 frames a second (`held.ron`'s
    `flame_fps`) makes it flicker gently, as a held torch does; terrain fires
    are still. It is unlit (it is its own light; the forge's brazier lamp
    lights everything else) and casts nothing, as every town flame.
  - *The anvil read as a dark blob*: two flat-shaded frustums in the
    settlement `iron`, the darkest tile in the set, and a four-sided horn.
    It is now **one smooth-shaded iron form**: rings of rounded section
    lofted from a spread foot, in to a waist, out to a flat face with a hard
    crease only at its edge, the horn lofted round off the face's end from
    the face's own section and tapering to a point, the heel square. It
    stands on a round elm stump (bark sides, timber end grain), so its face
    keeps the old 0.72 m. Its sides are a new 32 px `anvil` tile and its
    face a worn, lighter `anvilface` tile (forged iron's blue-grey, a value
    up on `iron`, with hammer scale and wear), drawn by the same script. It
    stays field-lit like the terrain (`LitByField`, `LitLikeTerrain` on the
    town root).
  - Look only: the pieces' cells, solids and lamps are unchanged and the
    layout stays 3 (no world has shipped it). Captures: the two shots
    retaken and `forge-close.jpg`, the fire and the anvil close up.

**26b. The forge's fire and anvil, redone (2026-11-24).** Written before
the code. *This is the redo that ships. A second redo landed on the branch
from another session at the same time (26a's last bullet, "The fire and the
anvil redone": a lofted flame crown in an animated `forge_fire` sheet and
the anvil on a stump); on merging the two, this one was kept, its fire
reading as tongues of flame and its anvil's face, horn and heel reading at
the player's distance, with every shot retaken and the full suites green
(recommendation taken, ask only with screenshots). The other is in the
history at `5c99a764`; its flickering flame sheet is a follow-up to port.*
Measured in the captures of 26a (`smith-forge-2230.jpg`,
`forge-day.jpg`): the fire is five crossed-triangle tongues
(`Sink::flame`) in one flat unlit colour, which read as five pale cones on
the hearth; the anvil is two frustums and a four-faced horn in the town's
`iron` texture, which reads as a dark blob with no face, horn or heel at
the distance a player stands. Both break "No obvious primitives" (CLAUDE.md:
"a part that reads as a primitive stuck on is redone, not shipped"). The
first redo of this was lost uncommitted with its container; this is done
again from the start.
- **The fire is one shaped, textured flame.**
  - A bed of coals fills the hearth bed: one low, lumpy surface (a grid
    over the bed, its heights from a fixed hash, so it is the same every
    load), in a new `embers` texture: dark clinker with glowing orange and
    yellow seams. Drawn unlit (it is the light), as a flame is.
  - Over it, one flame body: three ribbons crossed at 60 degrees through
    the fire's middle, each a strip of several rows that narrows to the top
    and leans a little, its outline cut by a new `forgefire` texture's
    alpha: several tongues of uneven height, a white-yellow core at the
    root through orange to red tips, painted pixel art in the town's
    palette. One mesh, no cone: the tongues are in the texture, so the
    silhouette is ragged and different from every side.
  - Drawn unlit, alpha-masked, both faces, never baked, casting nothing:
    the flame material's rules, with a texture. The `Brazier` lamp in the
    field still does the lighting.
- **The anvil is one lofted anvil.** A London-pattern profile along its
  length, sections joined into one closed surface with shared edges and a
  bevel at the face:
  - a foot flared on four sides, drawn in to a waist, the body widening to
    the face;
  - the face flat on top, a step down to the table, and a horn tapering
    to a rounded point and curving down a little; a square heel at the
    other end;
  - in two new textures: `anvil`, forged iron, dark grey-blue with lighter
    worn edges and hammer marks, and `anvilface`, the polished steel top,
    brighter, so the face catches the forge's light.
  - It stays a solid to the walker, the same box as before.
- **The art** is drawn in code by `tools/art/forge_art.py` (fixed seeds,
  the settlement palette, no outlines), written to
  `assets/textures/settlement/{embers,forgefire,anvil,anvilface}.png` and
  listed in the manifest.
- **Looked at close up** before it is shown: a capture from about 1.2 m off
  the anvil with the hearth in frame (`forge-close.jpg`), plus the 26a
  shots retaken. Only layout-3 towns have a forge, so no save changes.
- **As built (2026-11-25, task 3.2b).** As written, with these
  particulars and differences:
  - The coals are a 10 by 7 grid over the hearth bed, 1.2 cm over it at
    the bed's walls and heaped up to about 9 cm toward the fire's middle,
    its triangles split on alternate diagonals; `embers` repeats every
    metre.
  - The flame is three ribbons of four rows, 0.56 m wide at the root and
    0.3 m at the top, 0.47 to 0.55 m tall, each leaning 4 to 5 cm, every
    other one mapped the other way round so the tongues differ from each
    side; `forgefire` is 32 by 64, mapped once, `cut: true`.
  - The anvil is one surface of two lofts joined: 13 sections along it
    from the heel's end to the horn (twelve points round each, a bevelled
    block blending to an ellipse along the horn), and below the body six
    levels down through the waist to the foot, which share the body's
    underside edge for edge; a fan closes the heel's end, the horn's point
    and the foot. Normals are smooth across creases under 50 degrees and
    never across the polished face's edge. It is 0.74 m tall (the walker's
    box stays 0.72 m), shifted 3 cm along the forge's line so its heel
    meets the end of its box (`ANVIL_SHIFT_M`); the horn reaches out past
    the other end, as the old one's did.
  - Not in the design: fire in the town is one rule, `towns::is_fire`
    (`flame`, `embers`, `forgefire`): drawn unlit and both faces, never
    baked, shown with the flames once the town is half there, and kept out
    of the casters outside as well as in the rooms. An outside `flame` used
    to cast; none is left outside now that the forge's tongues are gone.
  - The anvil's palette is the settlement iron ramp, a touch cool: a
    bluer first try read saturated blue in the shade.
  - Captured on lavapipe (software): the look only, frame cost not
    measured.

**27. What other parts may use** (`pbd_app::townsfolk`):
- `TownPerson { id: PersonId { site, index }, role }` is on the root of
  every drawn person, so a talk or trade can find who is in front of the
  player.
- `Townsfolk::set_expression(id, Expression, seconds)` holds an expression
  on a person, drawn or not (a trader's smile on a trade: `Happy` for 2 s;
  a job done: `Delighted`, which plays its hop); `play(id, Act)` plays wave,
  hop, shrug or tilt.
- `Townsfolk::drive(id, feet, forward, walking)` takes a person off its post
  or path and stands them where the caller says, each frame, in the planet
  frame in f64 (a lost villager following the player); `release(id)` puts
  them back. The figure, the face, the solid circle and the gaze follow.
- `Townsfolk::people(site)` lists a standing town's people with their roles.
None of trade, missions or the lost villager is built here.

## The leftovers of phase 3 (2026-11-22)

Written before the code, on the owner's "Build phase 3": the lost villager
and repair jobs, the talking mouth, real captures of talking, roads the
player lays, cut and fill for the links that did not route, and the road's
smaller gaps. Each is a recommendation taken (ask only with screenshots).

**28. The lost villager, as built (task 7.5).**
- **Posted.** `lost` joins `jobs.ron`'s `posted`. Taking the job writes a
  `lost_villager` record (`pbd_core::economy::follow::Lost`, schema 1), id
  the job's id, authored by the player with the job in the same line set:
  the town, who it is (a person of that town's people, by index, rolled from
  the job's id over the roster's length, never its trader or job giver),
  where they stand, and whether they follow.
- **Where they stand** is rolled from the job's id: a bearing, and a reach
  from 120 m to `LOST_REACH_M` (300 m) from the town's anchor along the
  ground, on the first cell outward along that bearing that is dry ground
  (no sea, no river), outside every town's footprint, holds no player's
  edit, and stands within a layer of its neighbours. The point is stored in
  the body's frame in `f64` (the floating origin), so it is a stored record
  and never re-rolled.
- **Drawn as their own person.** While the record exists and their town
  stands, `Townsfolk::drive` stands them at the record's point, walking when
  they moved this frame; the figure, face, solid circle and gaze follow. At
  their post in town there is nobody in their place while they are out.
- **A map marker.** The map shows the job's villager as a marker at the
  record's point, with the town's name, as long as the job is taken and not
  handed in. Not built yet (2026-11-22): the map has no job markers.
- **Found.** The interact key on them opens the talk panel with one line,
  "Lead the way", and sets `following`; the record is written then.
- **Following.** Each frame `follow::step` moves them behind the player at
  the player's pace (hurrying when left past 8 m), never onto deep water:
  `dry` is the column under the point holding no water deeper than a layer
  over its ground, so they wait at a river's bank or the shore. Their point
  is written to the record when they come to rest after walking, never more
  than once a second, so a reload finds them where they stopped.
- **Home.** Within the town's wall (`follow::home`, the town's footprint
  radius) the job's `Escort` objective is counted from a `Did` the game
  writes (`Escort`, `<site>:villager`), the record is removed (a `removed`
  line), and the person goes back to their post (`Townsfolk::release`).
  Handing in pays as every job does.

**29. Repair, as built (task 7.6).**
- **Posted.** `repair` joins `posted`. Taking the job has the world break a
  piece: one wall of one of the town's buildings, at the ground storey, on
  an outer edge with no door or window, rolled from the job's id. The
  building record gains `broken: [[c, r, d, s]]` (a new field, written only
  where there is one, so every other building's record is as it was), and
  the cutter leaves that wall out, as it leaves a boathouse's open edge out.
- **Through the yield check.** The change is proposed by the world
  (`Author::World`, process `repair`, the job's id) through
  `WorldSave::propose`. A building whose record the player has changed is
  skipped, and the next is tried; a town with none to break has no repair on
  its board. A player's own pieces (`piece` records) are never in a town's
  building record, so the world never breaks a piece the player placed.
- **Mended by placing the material.** Near the gap (within 4 m of its
  middle) with planks in the pack, the interact key offers MEND. It takes
  two planks, writes the building record without the wall in `broken`
  (authored by the player, with the pack in the same line), writes a `Did`
  (`Repair`, `<site>:piece`) that counts the job, and stands the town again
  from its record. Planks mend every kit (the wall comes back in its own
  kit's look): one material to carry, recommendation taken.
- **Recut in place.** A town whose record changed is cut again on the pool
  and swapped for the standing one in one frame when the cut is done, with
  no fade, so it never blinks out.

**30. The talking mouth (task 6.3).** While the talk panel, the trade panel
or the board is open with a townsperson, `Townsfolk::set_talking` holds
their mouth talking; it stops the frame the panels close or the talk turns to
someone else.

**31. Roads the player lays (task 8.3).** *Replaced in the plan by design
39 to 41 (owner, 2026-11-22: roads marked by waypoints and paved with
gravel from a grinder and a mixer); the game keeps this until they are
built, and a road cell laid this way stays as it is in every save.*
- **Gravel** is a good (`gravel`, stack 99), made by hand from loose stone
  (`gravel` recipe: two stone make four gravel) and valued in `values.ron`.
  It is the seventh hand recipe, so the pack's craft list shows eight rows
  before it scrolls (`craft::ROWS_SHOWN`, was six), keeping decision 12 of
  the craft panel: every hand recipe shows without scrolling, with one row
  to spare.
- **Placing.** The place button (right) on the ground with gravel in the
  selected slot lays a gravel road cell on the aimed cell, taking one
  gravel; with the shovel in the tool slot and the selected slot empty it
  lays a dirt track (packed earth, nothing taken). The cell must be dry,
  its top open to the sky over two layers, not a town's footprint, not
  already a road. Its slope is `roads::built_up` toward the side faced. It is
  a `road-cell` record of the player's, through the durable path, with the
  pack in the same line; the walker walks it at its surface's speed.
- The place button with the shovel and an empty slot on a road cell of the
  player's takes it up (the gravel back for a gravel cell), so laying and
  taking up are one tool's two sides and the interact key stays the
  world's. As built.

**32. Cut and fill: roads version 2 (design 10a's "a later change may cut
and fill").** Measured on the shipped seed before this change: 53
settlements, 57 links, 27 routed with the shipped weighted search (35 with
the plain cheapest search, too slow for a loading screen); the rest have no
way that climbs at most a layer a cell.
- **Fill** raises a road cell's top up to `max_fill_layers` (2) over its
  ground on a bed of packed earth, a piece like a dip's raised bed: the
  ground is not edited. **Cut** lowers it up to `max_cut_layers` (2) under
  its ground: the world digs the layers between (world edits through
  `WorldSave::propose`, the yield check refusing any cell the player
  touched, which the route never crosses anyway). A cut or filled cell is
  dry, outside every town's footprint and not a ford or bridge.
- **The search** carries the road's height over each cell's ground as part
  of its state, and pays `fill_cost_m` (20 m) and `cut_cost_m` (25 m) a
  layer a cell on top of a cell's length, so a road cuts or fills only where
  it cannot climb round. A cut or fill is tried only onto a cell more than a
  layer from the road's height (so a ramp goes on cell by cell only while
  the ground still is); offering it on every cell put a copy of the whole
  search at each height and cost routes under the budget (22 routed of 57
  at first, against version 1's 27). The road still never moves more than a
  layer between cells.
- **Deterministic and bounded.** Costs stay whole millimetres; the order
  of states is total; the same budget (`max_expanded`) a link.
- **A new roads version, for worlds planned from now on.** The net record
  says `version: 2`. A world already planned keeps its roads as stored: it
  is never planned again. Version 1 is the same search with no cut or fill
  (`RoadsConfig::for_version(1)`), kept and pinned by its test.
- **The record.** A road with a cut or filled cell is written as a schema 2
  `road` record, each cell carrying its cut or fill; a road with none stays
  schema 1. Schema 1 still reads.
- **A cut cell's edits are the world's.** Every check that a road cell is
  laid asks whether the PLAYER touched its cell (the save's yield set), not
  whether it holds an edit, so a road's own cutting does not hide it.
- **What it reached, measured** (2026-11-22, this container, 4 cores,
  `--profile fast`, `roads::tests::the_shipped_seed_network` with no towns'
  ground installed, a measurement instrument whose version and budget are
  set by `PBD_ROADS_VERSION` and `PBD_ROADS_EXPANDED`):

  | Budget a link | Version 1 routed | Version 2 routed | Cells cut or filled |
  | ---: | ---: | ---: | ---: |
  | 100,000 (shipped) | 27 of 57, 2.0 s | 27 of 57, 2.2 s | 4 |
  | 300,000 | 31 | 30 | 4 |
  | 1,500,000 | 34, 18.8 s | 35, 18.6 s | |

  **Cut and fill does not reach the links left out.** Of the 27 links left
  out at 300,000 cells, 23 cross the sea on the straight way between their
  towns (from 12 to 855 sea cells: an inlet or another shore, which a road
  goes round by the coast, past the budget, or cannot), one crosses a river
  eleven cells wide (the longest bridge is ten), and three run 1 to 5.4 km
  over ground with a step of more than a layer every few cells, past the
  budget. The binding limit on the shipped seed
  is the sea and the search's budget, not the climb. Version 2 ships with
  the version 1 budget, so a loading screen is no longer; it cuts or fills
  the few cells where a road meets a two-layer step, and a later change may
  bridge longer water or raise the budget with a better estimate.

**33. No tufts on a road.** The surface word of a road cell carries a road
bit (bit 30, `planet_terrain::ROAD_BIT`), and the clutter pass grows no
pixel tufts there. A town's footprint keeps its tufts as it has them.

**34. Lamps along the roads near towns** (M15's Roads tab lights its roads
with lanterns on posts). Within `lamp_town_m` (90 m) of a settlement's
anchor, every `lamp_every` (5th) cell of a road from the town's edge has a
lantern post (`Material::LanternPost`, the towns' street lamp) on a
neighbouring cell off the road, the first such on the left of the way out
that is dry, within a layer of the road's ground and holds no player's
edit, in the first air layer over its ground. Derived from the stored roads
and installed with their clearance before the planet is built, as a town's
lamps are: never saved, the same every load. A cell the player edits shows
the player's edit, as with a town's lamps.

**35. The captures are played, not posed.** The talk, trade and board
captures stand the player beside a real townsperson (`--at` and `--yaw`,
the one thing set) and use the game's keys and buttons:
`PBD_KEY_SCRIPT` presses F, and a new `<frame>:press:<action>` step presses
a talk panel button (TRADE, WORK, BACK, a trade row) as a click would. The
old frames posed by `PBD_TALK` are replaced and the instrument removed.

**36. Beards are on the front of the face only (owner, 2026-11-22, M5
version 10).** The owner, on the Character sheets lineup and a Kerbal Space
Program 2 picture of a Kerbal with a goatee and a moustache: "get rid of
these old npcs, replace with new ones. also beards should only be infront
like this". Version 9's beard was a block wrapping the bottom of the head,
sides and back; from version 10 a beard is pieces on the face's flat front
and nothing on the sides or the back, in the hair's colour, each sitting
proud of the face as the other face layers do (embedded 5 mm, standing
25 to 30 mm out) and never over the mouth's opening:
- **Goatee**: a moustache over the mouth, 0.2 m wide and 35 mm high from
  0.15 m up the head (the highest mouth pixel any cell draws is at 0.148),
  and a chin tuft 0.09 m wide from 0.04 m below the chin to 0.055 m up (the
  lowest mouth pixel is at 0.058).
- **Full front**: the same moustache, a jaw plate 0.4 m wide (the head's
  flat front is 0.43) from 0.06 m below the chin to 0.055 m, and a cheek
  piece each side, 0.07 m wide at 0.13 to 0.2 m from the middle, from
  0.05 to 0.13 m up: it frames the mouth (whose widest cell reaches 0.07 m
  out, 0.105 m with the thinking face's shift) and stops under the blush.
- **The roll**: a beard as before (35% of those whose hair is not a bun or
  a braid), and its style half and half, goatee or full front, from one new
  roll appended at the very end of the sequence, after the resting face, so
  every other trait of every person is what it was.
- **Saves**: a person's look is derived from the town record and the seed,
  never stored (decisions 1 and 20), so this is a look change for old saves
  and new alike, and no save format changes.
- **The mockup**: version 10 replaces the hexagonal-prism figures of the
  square and the Character sheets with this figure (clothes rolled from each
  people's palette, the role's hat, apron, tabard and held prop), and puts a
  Beard control (none, goatee, full front) on the Faces tab's hero; the
  Faces tab's "today" figure goes, since the game now draws the new one.

**36a. A beard is one shaped piece, never boxes (owner, 2026-11-22, M5
version 11).** On a close-up of version 10's full front beard (a 0.4 m box
across the jaw and a box on each cheek): "I can see obvious primitives,
this is signof bad work ... avoid obvioud primitives" (CLAUDE.md, "No
obvious primitives"). The goatee's chin box and moustache bar are the same
fault. The owner's reference is a Kerbal Space Program 2 Kerbal with a
goatee and moustache, whose beard is one sculpted form that follows the
face, tapers to a point at the chin, has a soft irregular edge and grows
from the face rather than sitting on it. From version 11 each beard is one
mesh, a shell laid over the head's surface:
- **The surface.** The shell is drawn on a grid laid over the head's front
  and its two front bevels (the bevel is unrolled, so a cell 0.215 m or
  more from the middle lies on the bevel and wraps round the front corner),
  and below the chin it hangs free. The grid's columns are edged at 0,
  0.035, 0.07, 0.108, 0.14, 0.175, 0.215 (the bevel's start), 0.24 and
  0.2645 m (the bevel's end at the side) either side of the middle; its
  rows at -0.11, -0.075, -0.04, 0, 0.035 (the bottom bevel's top), 0.058
  (the lowest mouth pixel), 0.103, 0.148 (the highest), 0.178, 0.208, 0.238
  and 0.268 m. The mouth's opening is three whole columns and two whole
  rows, so no cell can reach it.
- **The shape** is a pixel stencil on that grid, one per style (`#` beard,
  `+` stubble), mirrored about the middle, so its edge steps irregularly like
  the game's pixel art instead of running as a straight box edge:
  - *Full front*: a moustache parted under the nose; cheeks that run from
    it round onto the front bevels and rise up the bevel as sideburn points
    to 0.268 m; a frame either side of the mouth; a chin that covers the
    bottom bevel and hangs 0.11 m below the head, narrowing row by row
    (0.35, 0.14, 0.07 m wide) to a rounded point.
  - *Goatee*: a moustache parted under the nose, its ends drooping beside
    the mouth and joining a chin piece that narrows (0.14, 0.07 m) to a
    point 0.075 m below the head: one ring round the mouth.
  - *Stubble* (`+`) cells lie flush (1.5 mm) in a colour between the hair
    and the skin (45% of the way to the skin): the edge fades into the face
    as if painted, where geometry would stand proud.
- **The thickness** tapers: 4 mm wherever the shell meets the face, growing
  inward with how much beard surrounds a point, to 18 mm on the cheeks,
  28 mm in the moustache, 26 mm beside the mouth and 34 mm at the chin, with
  a small fixed lump pattern (up to 3 mm) so the facets read as hair, not a
  slab. Each cell is two triangles on an alternating diagonal, flat shaded.
  *Soft shaded since 2026-11-22* (owner, on the game's full front beard
  capture `docs/screenshots/townsfolk/beard-full-front.jpg`: "soft shade
  the beard, not flat shade"; its faceted triangles read as crumpled): the
  shell's corners that meet at a point share one normal, the mean of the
  face normals there within 60 degrees of each other (decision 38's
  `smooth_from`), so light rolls over the cheeks, the front corners and the
  chin, and the crease stays crisp only where a wall turns into the head
  (about 90 degrees). The lump pattern is dropped: under soft shading it
  read as dents, not hair.
  Below the chin the shell's front tucks back and its underside slopes up
  into the head, so the point is thin.
- **What it keeps from 36**: the hair's colour, lit like the rest of the
  figure; nothing on the sides or the back (it wraps onto the front bevels
  only, never past them); every point in front of the head but for the
  5 mm it sinks; the mouth's opening clear in every expression and while
  talking; the roll and the saves unchanged (a look is never stored).
- **Why one piece**: boxes placed on a surface keep their own straight
  edges, flat tops and hard corners, so they read as parts put there.
  A shell laid on the surface shares the face's shape and meets it at
  nothing, and its taper and stepped edge are what makes a block head's
  beard look grown.
- The mockup's `FK.BEARDS` and the game's `figure::beard_shell` build it
  from the same grid, stencils and thickness rule; the stubble colours join
  the atlas's swatches (each people's hair with each of its skins).

**38. An ear is one shaped piece grown from the head, soft shaded (owner,
2026-11-22, M5 version 12).** Under CLAUDE.md's "No obvious primitives"
(the owner: "I can see obvious primitives, this is signof bad work ...
avoid obvioud primitives"), version 9's ear, a 0.05 x 0.14 x 0.1 m box
stood on the head's side face in the skin's shade, reads as a primitive
stuck on: straight edges, a flat outer face, a hard corner where it meets
the head. From version 12 each ear is one tapered, slightly cupped form
that grows out of the side face:
- **The outline** is an irregular rounded shape on the side face, wider at
  the top and narrowing to a lobe (eight points, `figure::EAR_OUTLINE`,
  about 0.1 m deep and 0.15 m tall at its rim), centred 0.26 m up the head
  and 5 mm behind its middle, where the box stood.
- **Three rings and a cup** (`EAR_RINGS`): a root ring 1.3 times the
  outline sunk 2 mm into the head, so the ear's foot flares into the side
  face like a bevel instead of meeting it at a corner; the outline itself
  14 mm out; a rim 0.86 times it, 42 mm out (the box stood 45 mm proud, so
  the ear still shows past the head from the front) and leaning back 10 mm
  at its top. Inside the rim a shallow bowl falls 16 mm to its middle
  (`EAR_CUP`), in the skin's shade (the colour the box wore),
  the rest in the skin: the cup reads as an ear's hollow, the outside as
  the head's own skin.
- **Soft shaded** like the beard (owner, 2026-11-22: "soft shade the beard,
  not flat shade", and the ears the same way): `Part::smooth_from` gives
  every corner the mean of the face normals meeting at its point within
  60 degrees of its own (`SMOOTH_CREASE`, cos 0.5), so the flare and the
  outside roll smoothly and only the rim's turn into the cup stays crisp.
  The rule is deterministic: corners are matched by their exact position,
  in the order built, and the mockup's `smoothGeos` does the same.
- 40 triangles an ear (the box was 12), so the hero is about 730 with its
  face. The roll is unchanged (`Look::ears`, 60%; hoods and headsets none),
  and a look is never stored, so saves are untouched.
- Painted into the side face was the other way open; a modelled ear keeps
  the block head's silhouette from the front, which is how a townsperson
  is mostly seen at game distance, so it is modelled.
- The mockup's `FK.earGeo` and the game's `figure::ear` build it from the
  same outline, rings, cup and crease.

**38a. The trade and board panels' icons sit in the pack's slot wells
(2026-11-22, on the owner's "you need proper icons for trade").** The trade
rows showed each good's pack icon at 16 px with no well, and the board's
rows none. Every buy row, sell row and job row now shows its good in the
same 24 px dressed-stone well as a pack slot (`slots::SLOT`, the
`slot-idle` piece, its 16 px icon floor at 4, 4, the item PNG's own size),
so a good reads the same in the pack, the hotbar and a deal; a job's well
holds the good it asks for (the dish, the ingot, the parcel, the planks a
repair is mended with), and a job with no good (the lost villager, lamps, a
catch of any fish, selling any goods) its well empty, the pack's empty slot. The coin sits
beside every price as before. The barter basket of decision 37 is drawn in
M5 only and waits for a request.

**38b. Long lists scroll, in a panel of fixed height (owner, 2026-11-22,
on the trade panel: "opt for a scroll bar incase theres lots of
products").** The trade panel listed at most nine rows a column and
dropped the rest (`ROWS`, `take(9)`); a pack of many goods, or a trader
with a long stock, lost rows silently, and a board of many jobs grew the
panel past the screen. From now:
- **The panel keeps one height.** Each trade column shows a window of
  `VISIBLE_ROWS` (7) rows, the board's list a window of 7; the window is
  that tall whether it holds one row or forty, so the panel does not jump
  as goods are bought, sold or filtered. The purse, the last line said and
  BACK stay fixed under the lists.
- **Each column scrolls on its own**: BUY and SELL each keep their own
  first-shown row. Only the rows in the window are built (the panel is
  rebuilt when what it shows moves, as before), so a list of a hundred
  costs what a list of seven does.
- **The mouse wheel over a column scrolls it by whole rows**, one row a
  notch, never half a row; over the other column, or outside both, it does
  nothing to this one.
- **A scroll bar beside the column, only when its rows overflow**: the
  stone kit's `slider-track` stood on end, 8 native px wide and as tall as
  the window, with a thumb inside it in the rune fill the settings sliders
  use (`RUNE_DIM`, `RUNE` while held or hovered; the kit has no stone thumb
  piece, and the oak `slider-knob` is the timber menus'). The thumb's
  height is the shown share of the list (never under 8 px) and its place
  the first row's share of the rows that can be first. Dragging the thumb
  scrolls, snapping to whole rows; a press on the track above or below it
  pages by a window. With no overflow there is no bar and the space stays.
- **Clamped**: the first row is never past `rows - VISIBLE_ROWS`, so the
  window is never short at the bottom; a list that shrinks (the last of a
  good sold) clamps on the next paint.
- **Focus**: the talk panels have no keyboard or controller focus today
  (a press is the pointer's), so there is nothing to follow. The maths
  carries `reveal(row)`, which scrolls the least that shows a row, ready
  for when they do.
- **The job board scrolls the same way**: YOUR JOBS and ON THE BOARD are
  one list (their headings rows of it), in one window with its bar.
- **Approved by the owner** (2026-11-22, on this plan and 38c: "approved";
  then on M5 version 13's trade panel screenshots, scroll and filters:
  "looks good"; and for the game: "approved trading ui", the trade panel,
  its scroll bars and its filters).
- The rule is one place, `desktop::scroll_list` (`ListView`: window,
  clamp, wheel, thumb size and place, drag, reveal), tested apart from
  the UI; the mockup's `ListView` is the same maths.

**38c. Each trade column filters by shelf (owner, 2026-11-22: "some
filtering on each side would be nice: food, organics, raw materials,
tools, etc").** Above each column's list, a row of tabs: ALL, then one tab
per shelf that has goods in that column (a shelf with nothing to buy, or
nothing in the pack the trader wants, shows no tab).
- **The shelves are the goods' own group, read in fewer words**: every
  good already carries a `Group` in `values.ron` (what a town's need is
  read by), so no new data is added and no row changes.
  `values::Shelf::of(group)` maps them onto the owner's words:
  - **Food**: crop, fruit, fish, dish, and the herd's eggs and milk
    (`values::HERD_FOOD`; owner, 2026-11-22: "put egg in food, not
    organics (milk too)");
  - **Organics**: seed, herd (wool, hides, a fertile egg for hatching),
    wood;
  - **Raw**: ore (ore, coal, ingots) and stone;
  - **Tools**: tool;
  - **Goods**: made things and lights (rope, buckets, boats, lamps).
  The ids are `food`, `organics`, `raw`, `tools`, `goods`; their names are
  the locale file's `trade.shelf.<id>` (`trade.shelf.all` for ALL), never
  text in code. Fibre is a crop, so it stands on Food; a shelf column in
  `values.ron` would move it, and is left until the owner asks. Tools are `untraded` today, so their tab shows on
  no trader yet; it is there for when they are.
- **A tab is a pack slot well** (24 px, `slot-idle`, `slot-hover` under
  the pointer, `slot-selected` when chosen), holding a shelf's picture: a
  good of it (grilled fish, a log, an iron ingot, the pickaxe, rope), and
  ALL its word. The shelf's name is its tooltip (the kit's own).
- **Choosing a tab puts that column's scroll back to the top**; the other
  column keeps its own filter and scroll. Both go back to ALL and the top
  when the talk closes.
- **Approved by the owner** with 38b (2026-11-22: "approved"; on the
  version 13 screenshots: "looks good"; for the game: "approved trading
  ui").
- **The board is not filtered**: a job is a kind of work (cook, bring,
  deliver, mend, find), not a good on a shelf, and a board posts a handful
  a day; it scrolls (38b) and no more.
- **No trader sells basic tools (survey E1, owner, 2026-11-22: "we'll have
  a special weapons and enchanted tools that give +1 mining speed +1 yeild,
  stuff like that").** The stone pickaxe, axe, hoe, rod and the rest are
  made, never bought: making them is the loop's first step. The Tools
  shelf on a trader's side is for what comes later: enchanted tools and
  special weapons, each with its bonus (such as +1 mining speed or +1
  yield). Neither is designed yet; they wait for a write-up of
  enchantments and weapons (weapons come with survival, phase 4,
  `survival-mode`). Until then tools stay `untraded` and the buy side
  shows no Tools tab, as today.

## Roads marked by waypoints, paved with gravel (2026-11-22)

**The owner, 2026-11-22:** "building roads shjould be like a way point
system, where you have to mark road for pavving and then you walk up to
blocks with gravel and pave it. to make gravel you need a grinder and mixer.
which needs to be crafted in phase 3. with gravel you just equip a shovel and
interact with the blocks marked for crafting a road." This replaces design
31's place-a-cell in the plan; the game keeps design 31 until this is built
(task 13). Written before code; shown first in M15 version 4's Paving scene
(`towns.html#paving`). The town tree's Roads branch
(`town-missions-tree` design 2) walks a player through it. Every choice the
owner did not make is a recommendation taken (ask only with screenshots).

**39. Marking a road with survey stakes.**
- **Survey stakes** are a good (`survey-stakes`, stack 32), made by hand:
  one planks and one fibre make four (`survey-stakes` recipe, unlocked by
  the town tree's "Mix gravel"). A stake is a waypoint.
- **Driving a stake.** With stakes in the selected slot, the place button on
  the ground drives one into the aimed cell's top and takes it from the pack.
  - The first stake starts a **plan**: a road the player means to pave.
  - Each next stake joins the plan's last stake: the cells between are found
    by the world's own road search (`roads::route`, version 2's rules, design
    10a and 32): at most a layer between cells, a shallow ford allowed, deep
    water bridged only by the world, never the sea, never a town's footprint
    (its lanes are already road), never a cell holding a player's edit
    unless that edit is a road cell. A segment is at most 24 cells
    (`max_stake_cells` in `roads.ron`) and its search is capped
    (`stake_expanded`, 4,000 cells), so it is quick.
  - Every cell of the segment, the stake's cell included, is **marked**.
  - Refused, with nothing taken, when there is no way: "Too far", "Too
    steep", "Water in the way" (the locale's `roads.mark.*`).
  - **While aiming**, the cells the next stake would mark are shown in the
    mark's own look at half strength, so the player sees the way before
    driving the stake.
- **A plan ends** when the stakes leave the hand. The next stake starts a new
  plan, unless it is driven on the last stake of a plan (which continues it)
  or on a marked cell (a new plan that starts there: a junction).
- **Taking a stake back.** Digging a stake (any tool, or the hand) takes it
  into the pack. The segments either side of it lose their marks where they
  are not paved, and its neighbours in the plan are joined by a fresh search
  if there is a way, otherwise the plan splits in two. A paved cell stays
  road whatever happens to the marks.
- **One cell wide.** A plan is one lane. A wider road is two or three plans
  side by side; when wide roads are built (task 8.6), a player's lane beside
  another takes its top from their shared plane (design 10b).

**39a. How a mark looks.** No outline, no primitive stuck on the ground,
lit as the ground is (CLAUDE.md):
- **A marked cell is dusted with lime along the line of the road**: a band
  about 0.9 m wide from the middle of the edge the road comes in by to the
  middle of the edge it leaves by (to the cell's middle at the plan's two
  ends), pale and chalky, with a ragged, broken edge and the grass under it
  flattened. It is painted into the cell's own top, never laid over it: in
  the game a mark bit on the cell's surface word (`MARK_BIT`, bit 29, beside
  design 33's road bit) and the band's two edges (three bits each) make the
  terrain shader blend a lime dust texture into the biome's own top along
  the band, so it is lit, shadowed and coloured by the ground's own light
  and palette, and sits in the scene. The clutter pass grows no pixel tufts
  on the band, as on a road (design 33).
- **A stake at each waypoint**: one tapered peg of split timber driven into
  the cell, its foot sunk under the top, its head split and frayed by the
  mallet, a strip of dyed cloth tied under the head, 0.9 m out of the
  ground; in the timber of the land's own trees. One mesh shaped as one
  form, `field_light::LitByField` and `LitLikeTerrain` on its root, so at
  night it is as dark as the ground there, lit by the lamps and the moon.
- **At night** the band and stakes are seen only where light falls on them,
  like the ground; the compass bar shows the open plan's next unpaved stake
  as a tick and the map shows every plan's stakes.
- M15 version 4's Paving scene shows marked cells beside a paved run, by day
  and at night.

**39b. Marks as data.**
- **`road-plan`**, a record of the player's, schema 1, id the plan's number
  (counted up, never reused): its stakes (cell keys, in order) and its cells
  (cell keys, in order along the road, each stake's cell among them).
  Written through the durable path with the pack (the stake taken or given
  back) in the same line, on every stake driven or taken back.
- **The cells are stored, not re-searched**, so a later search or road
  version never moves a plan a player made. Marked is derived: a plan's cell
  that is not paved.
- **Old saves** have no plans; nothing they hold changes.

**40. Paving.**
- **Shovel in hand, gravel in the pack, interact on a marked cell.** With the
  shovel in the tool slot, the interact key (F) on a marked cell within reach
  (4 m) takes one gravel from the pack and makes the cell a gravel road cell
  of the player's. Held, it paves the next marked cell in reach on the
  shovel's stroke beat (`held-swing`), so walking the line with F held paves
  it. One gravel a cell. The shovel's dig stroke plays and the gravel tips
  out.
- Refused, nothing taken, with one plain line: "Needs a shovel", "Needs
  gravel".
- **The slope is the world road's.** A paved cell's top follows the plan's
  order exactly as `roads::road()` cuts a world road (design 10, 10a): flat,
  or sloped up toward the plan's neighbour a layer higher, from the edge of
  its other neighbour; a dip one cell wide is raised a layer on packed earth
  (`RoadCell::raised`). The order is known when the cells are marked, so a
  cell's slope does not wait on its neighbours being paved, and paving in any
  order gives the same road. The cells were found with at most a layer
  between them, so a paved plan never steps.
- **The record.** A paved cell is a `road-cell` record of the player's
  (schema 2: the surface, the plan, the two edges its slope runs between and
  whether it is raised), with the pack in the same line. Schema 1 (design
  31's cells, with `built_up`'s slope) still reads and is drawn as it was.
- **Taking up.** Digging a road cell of the player's with the shovel takes
  it up and gives its gravel back; a cell of a plan goes back to marked.
- **Over a world road.** A plan may cross a world road cell (a village's dirt
  track); paving it writes the player's road cell at that key, which the
  world's road yields to, as it yields to every player edit (design 11). So
  a village track can be upgraded to gravel cell by cell.
- **The place button no longer lays a road.** Design 31's place rule (gravel
  in the slot lays a gravel cell; the shovel and an empty slot lay a dirt
  track) is retired from the game when this is built. Every road cell laid
  that way stays in its save, drawn and walked as before.

**40a. Dirt tracks still exist, laid by the world only.** (Reversed for a
planned road by design 42c: a plan's surface may be dirt.) The world's roads
between villages stay dirt (design 10, 11), and every world already planned
keeps its tracks. The player lays only gravel: a dirt track needed no
material, so it skipped the grinder and mixer the owner asked for, and two
ways to lay a road were one too many. A player's dirt cell from design 31
stays as it is. Recommendation taken (ask only with screenshots). On foot,
gravel is still walked 1.15 times the ground and dirt 1.0 (design 10a);
for wheels, `land-transport` design 9 gives each its own factor.

**41. Gravel from a grinder and a mixer.**
- **The grinder** (a station, `grinder`) crushes stone. A hopper over a pair
  of stone rollers in a timber frame, turned by a crank, crushed stone
  falling onto an apron. Made at the workbench, unlocked by the town tree's
  "A job from the board". It stands on one dry level cell like the forge,
  holds a queue like every station (`farming-and-smelting`), and is lit like
  the ground.
- **The mixer** (a station, `mixer`) mixes crushed stone with sand into
  road gravel. A tumbling drum on a timber frame with a crank and a trough
  under it. Made at the workbench, unlocked by "Crush stone".
- **The recipes** (`recipes.ron`, append-only codes):

| code | station | secs | inputs | outputs |
| --- | --- | ---: | --- | --- |
| `grinder` | workbench | 6 | 10 stone, 6 planks, 2 iron ingots | 1 grinder |
| `mixer` | workbench | 6 | 8 planks, 2 rope, 1 iron ingot, 1 bucket | 1 mixer |
| `crushed-stone` | grinder | 4 | 2 stone | 3 crushed stone |
| `mixed-gravel` | mixer | 6 | 3 crushed stone, 1 `block:sand` | 4 gravel |
| `survey-stakes` | hand | 1 | 1 planks, 1 fibre | 4 survey stakes |

  So a cell of road costs half a stone and a quarter of a sand block, and
  ten seconds of station time per four cells. Recommendation taken (ask only
  with screenshots); the numbers are data.
- **New goods**: `crushed-stone` (stack 99), `survey-stakes` (stack 32),
  `grinder` and `mixer` (placed stations, stack 1), each a row appended to
  `values.ron` and an icon from `tools/gen_item_icons.py`.
- **Gravel's hand recipe goes.** The `gravel` recipe of design 31 (two stone
  make four gravel, by hand) is removed from `recipes.ron` when this is
  built. Its code is retired, never reused: a hand recipe has no station
  queue, so no save holds it. Gravel itself stays a good, and gravel already
  in a pack stays there and paves. `craft::ROWS_SHOWN` stays 8 (bc54742e):
  the survey stakes take gravel's row, so every hand recipe still shows
  without scrolling, with one to spare.
- **The rest of phase 3 is unchanged**: `values.ron`'s gravel row (1 coin)
  stays; traders may stock gravel as before.

**43. A sparse road network: roads version 3 (owner, 2026-11-22: "towns
should start out unlinked, or very loosly linked (at most 2 towns are
connected in a network)").** Roads are now something the player builds
(design 39 to 41), so the world lays far fewer.
- **Each settlement is joined to at most one other.** Every pair within
  `link_m` (2 km) is taken nearest first, by arc length then by site ids;
  a pair whose either town is already joined is skipped. The large towns'
  long link (`large_link_m`) is not made. So a network of the world's roads
  is one town or two, never three.
- **A pair stays apart on the ground as in the plan.** A road whose route
  crosses a third settlement's footprint, or shares or touches a cell of a
  road kept before it (in link order), is left out: a crossing or a fork
  would join four towns on the ground whatever the link list says. The
  check is on the stored cells, so it is the same on every machine.
- **A new roads version, for worlds planned from now on.** The net record
  says `version: 3`; `RoadsConfig::max_network` (2 in `roads.ron`) is the
  rule, and `for_version(1)` and `for_version(2)` set it to nought, which
  is the old rule (two nearest and the large towns' link). A world planned
  by version 1 or 2 is never planned again and keeps every road it has,
  cell for cell. Pinned before the change by a count and digest of
  version 2's links on a fixed layout of settlements
  (`roads::tests::version_2_links_are_pinned`).
- Surfaces are unchanged: gravel between two walled towns, dirt otherwise.
- Recommendation taken (ask only with screenshots): pairs, not lone towns
  only, so a new world still shows a road or two for the player to follow
  and widen.

**43a. The world's own wide roads: roads version 4 (2026-11-25, task
8.6, on the owner's "cool implement all the phase 3 stuff!" and design 10b;
written before the code).** The player's planner lays two- and three-wide
roads (42l, 42m); the world's own roads are still one wide.
- **What the world lays wide.** A gravel road between two walled towns is
  two wide; inside a large town's gate (the cells within the larger town's
  footprint) it is three wide. Dirt roads stay one wide. The links are
  version 3's (design 43): the same pairs, the same order, the same
  centre line; only the lanes are added.
- **Lanes as the player's are.** Each lane is the centre line moved one
  lattice step to the side, on the one shared plane of 42m (the lane bed,
  the plane moved down the hill where that saves a cut, straight climbs),
  bends and junctions only on the level; a lane cell the planner refuses
  ("Too steep across", a settlement's footprint outside its gate, a cell of
  another road) is left out and the road is narrower there, never bent to
  fit. Wide bridges railed outside, and lamps both sides of a three-wide
  stretch, as 42m. The same code as the player's planner, not a copy.
- **A new roads version, for worlds planned from now on.** The net record
  says `version: 4`; `RoadsConfig` gains `gravel_width` (2) and
  `gate_width` (3), and `for_version(3)` and below set both to 1, which is
  the old rule. A world planned by version 1, 2 or 3 is never planned again
  and keeps every road it has, cell for cell. Pinned before the change by a
  count and digest of version 3's cells on a fixed layout of settlements
  (`roads::tests::version_3_cells_are_pinned`).
- **Verify:** a core test that a version-4 gravel link between two walled
  towns is two wide along its length (where no lane is refused) and three
  wide inside the large town's gate; the walker test of 42m holds on a
  world road; a capture of a world wide road up a slope.
- **As built (2026-11-25).** Where the plan left a choice, each is a
  recommendation taken (ask only with screenshots):
  - **The gate is the road's end at the town.** A world road's cells stop
    at a town's footprint (design 10a: the town's lanes are the road
    there), so "inside the gate" is the stretch where the road meets the
    footprint: the last `gate_cells` (6, a new tunable beside
    `gravel_width` and `gate_width` in `roads.ron`) centre cells at each
    end. Both ends of a gravel road are walled towns, so both ends are
    three wide; no road cell is laid in a footprint.
  - **One implementation.** `planner::Lanes` takes the width at each
    centre cell and the stretches of the centre line each laid beside by
    one lattice step; `Lanes::widen` is the plane's choice (flat, or moved
    down the hill half a cell at a time). The player's planner passes one
    stretch, the whole line, so its roads lay cell for cell as before (its
    tests unchanged). A world road's stretches are its straight runs: on
    one face and no further than a cell's spacing off the great circle
    between their ends, so a lane runs beside the route's own wiggle. The
    bend rule is `planner::wide_bends`, the lamp verge `planner::verge`.
  - **What is left out.** A lane the planner refuses (a town, the
    player's work, "Too steep across", "Too wide to bridge"), one beside a
    centre cell that bends or zigzags on or beside a rise, and one that is
    or touches a cell of another road (so no two pairs meet on the ground,
    design 43). The centre cells are re-read off the plane chosen (a bed
    where the plane was moved), never moved.
  - **The record.** A road with lanes or a bed is a `road` record of
    schema 3 (`LANES_SCHEMA`): every cell with its cut or fill and its bed,
    and the lanes with the centre cell each lies beside and its side.
    Schema 1 and 2 records read as before. The net record says
    `version: 4`.
  - **In the game.** `RoadNet` lays the lanes after their road; a bridge
    cell of a wide road is railed only on its sides with no cell of the
    road beside it. The lamps near a town (design 34) stand beside a wide
    road's outermost cell on its left, and on both sides of its three-wide
    stretch.
  - **Tests.** `roads::tests::version_3_cells_are_pinned` (taken first:
    916 cells, in dirt and every town walled in gravel);
    `a_gravel_road_between_walled_towns_is_two_wide_and_three_at_the_gate`
    (flat ground: a road cell beside every centre cell on the left, and on
    both sides of the six at each end; the schema 3 record reads back;
    version 3 lays the same centre line one wide in schema 1; dirt stays
    one wide); `a_world_wide_road_up_ten_layers_is_crossed_lane_to_lane_
    without_a_step` (two walled towns along a row, a hill of ten layers up
    and down between: seams under 3 cm, nothing steeper than 0.37 m a
    metre, lanes on beds and never cut); in the app,
    `a_new_world_lays_its_gravel_road_wide_and_a_version_3_world_keeps_its_own`
    (on the start's rolling ground about half the lane cells are left out,
    "Too steep across" or beside a bend on a rise; a save whose net record
    says version 3 is read as it is, never planned again).
  - **No world wide road in a new world on the shipped rules.** Walled
    towns are placed at least `spacing_m` 2,500 m apart (`sites.ron`) and
    version 3 links only pairs within `link_m` (2,000 m), with the walled
    towns' long link dropped (design 43). So no two walled towns are ever
    linked, no world road is gravel, and version 4 lays no lane on the
    shipped seed: a new world's plan logs 8 roads, all dirt (2026-11-25).
    The capture of a world wide road up a slope cannot be taken until a
    walled pair is linked; that is a change to the links (design 43's
    rule), left for the owner. The lanes' look on a slope is the player's
    wide road's, the same pieces; that capture (`road-slope.jpg`) is still
    owed: the takes of 2026-11-25 did not frame the road (see task 14.5).

**44. Grass tufts go at once from a paved or marked cell (owner,
2026-11-22: "ok").** A cell the player paves or marks has no pixel tufts
from that moment, not from the world's next open (the known issue
`phase-3-roads.2`): the road bit of the cell's surface word (design 33) is
set in the standing fine set as the record is written, and the cells are
added to the installed clearance so a set built later has it too. Every
piece of ground clutter goes from a road cell, the world's or the
player's (a shrub through fresh gravel read as a mistake in the first
capture), but a tree standing on the cell stays until it is felled, so
nothing the world grew vanishes behind the player's back.

**45. As built (2026-11-23, design 39 to 41, 43 and 44, on the owner's "continue
build").** What the game does, and where it differs from the plan:
- **The stations are blocks, as the forge and the workbench are.** The
  grinder and the mixer are `Material::Grinder` (30) and `Material::Mixer`
  (31), stations of `pbd_core::station` with a queue each, made at the
  workbench (`block:grinder`, `block:mixer`; valued 40 and 35 in
  `values.ron`). The block is drawn in the local trees' wood, as the
  workbench's is, and its form stands on it: lofted, tapered and bevelled
  parts (`pbd_app::roads::models`): the grinder's splayed posts, rails,
  thick-walled hopper, two bellied stone rollers, iron axle and crank and
  sloping apron; the mixer's bellied drum of staves with three iron hoops,
  two A-frames, axle, crank and trough. Textured in the towns' own pixel
  textures (owner, 2026-11-23: "Make sure things have textures"): timber
  for posts, rails and pegs, plank for the hopper, apron and trough, barrel
  staves for the drum, iron for the fittings, stone for the rollers, cloth
  for a stake's tie; nearest-sampled, laid in metres at the manifest's
  repeat, the grain running along each piece and round it. Prewarmed
  behind the loading screen, `LitByField` and `LitLikeTerrain` on the root.
- **Gravel by hand is retired** into `station::RETIRED_RECIPES` (two stone),
  so a hand queue saved with it refunds its stone; gravel in a pack stays
  and paves. Survey stakes take its row in the hand list.
- **One rule for a way between waypoints**: `roads::paving::way` gives
  the cells the search takes, or why there is none, and every cell on the
  straight line that refuses a road with its reason (water, a town or the
  player's work, a step of more than a layer). A stake asks it; the road
  planner's live ghost (the owner's road planner, written up beside this)
  can ask the same function. Test `a_way_names_each_clashing_cell_and_why`.
- **The stake search allows no ford** (recommendation taken, ask only with
  screenshots): a player's plan stays on dry ground, so a paved cell is
  never under water; "Water in the way" refuses it.
- **The band** is the record's `spare[2]` bits 16 to 22
  (`planet_column::MARK_*`), not the surface word (bits 26 to 30 of that
  are taken by the placed sheet and the road bit). The cap's fan triangles
  each carry the signed distance across the band's nearest leg in the
  vertex's `shade`, which is linear over the triangle, and the fragment
  reads it at each texel's centre, so the band's ragged edge is the tile's
  own pixels. Lit and shaded as the ground is.
- **Taking a stake back** is the dig press on its cell (any tool or the
  hand), spent before the digging; taking up a paved cell is the place
  button with the shovel and an empty slot, as design 31 had it.
- **Held paving** paves the nearest marked cell within 4 m of the aim every
  `pave_beat_s` (0.45 s) while F is held with the shovel in hand.
- **Not yet built:** the half-strength preview of the next stake's cells,
  the map's stakes, and the shovel's stroke and the gravel tipping out as
  a cell is paved. (The compass tick is built, 2026-11-25: 42d's "As
  built".)
- **For the town tree** (`town-missions-tree` design 4): every stake, mark
  and paving raises `pbd_app::roads::paving::RoadWork` (`Stake`,
  `Mark(cells)`, `Pave`) on the frame its line is accepted; the grinder's
  and the mixer's jobs are station recipes, counted as every recipe is.
- Frame cost not measured (cloud session, lavapipe; CLAUDE.md).

## Road planner: waypoints walked in first person (2026-11-23)

**The owner, 2026-11-23:** "Do you have a good mock.up for road building
too? Should work similar to the house architecture but we can just use the
regular fps controller for that since it's just moving from one spot to
another to place down waypoints.gor road". So road planning takes the house
planner's shape (`building-planner`: a plan, a ghost checked as a whole with
red reasons on only the cells that clash, a bill of materials, building piece
by piece from the pack, edits mid-build that give materials back, tunables)
but has no drafting table and no camera of its own: the player walks the
road out with the ordinary walker and drops waypoints where they stand.
Written before code; shown first in M15 version 5's Road planner scene
(`towns.html#roadplanner`). Every choice the owner did not make is a
recommendation taken (ask only with screenshots).

**42. What stays from design 39 and what is new.**
- **Kept.** Survey stakes are still the item (`survey-stakes`, its recipe and
  stack); a stake is still a waypoint; the `road-plan` record still holds the
  plan; marked cells still wear design 39a's lime band and each waypoint its
  stake; paving is still design 40's shovel and gravel, the slope cut by the
  world road's rule from the plan's order; gravel still comes from design 41's
  grinder and mixer.
- **New.** The live ghost of the finished road (not only the next segment's
  cells), refusals red on only the clashing cells with their reason, a width
  and a surface chosen in a small menu, a running bill and turn count,
  waypoints grabbed, inserted and deleted, the house planner's mid-build edit
  rule, lamps along the plan, and the plan saved whole as one record.
- **Replaced.** A segment is no longer a search (design 39's `roads::route`
  between stakes): it is the line of cells between its two waypoints (42a).
  `roads::route` stays the world's; both are checked cell by cell by one rule
  (`roads::check_cell`, `roads::check_step`), so a player's road and the
  world's agree on what is allowed. Taking back a middle stake whose join
  would be refused is refused (42f) instead of splitting the plan.

**42a. Walking it out.**
- **Stakes in the selected slot** make the hand a planner: the aimed cell is
  tinted by the ghost and, while a plan is open, **the ghost road runs live
  from the plan's last waypoint to the aimed cell**, recut every time the
  aimed cell changes.
- **The place button drives a stake** into the aimed cell if it is within
  `stake_reach_m` (6 m) and the segment to it is allowed. It takes one stake
  from the pack, adds the waypoint and writes the plan. The first stake opens
  a plan; the plan stays open (the ghost keeps following the aim) until the
  stakes leave the hand or the player ends it (the use key on the last stake,
  Enter in the mockup). A stake on the last waypoint of a closed plan reopens
  it.
- **A segment is the cells on the straight line between its two waypoints**,
  the hex line (cube lerp, ties broken to the left of the direction of
  travel), so the road goes exactly where the player walked it. To go round a
  hill, a tree or a town, the player drops another waypoint. Recommendation
  taken: the owner's "moving from one spot to another to place down
  waypoints" puts the player in charge of the line, and a search that
  wandered off it would not.
- **Cut by the road rule**, the same cut a paved cell gets (design 10, 10a,
  10b, 40), shown in the ghost before anything is staked:
  - **slope a layer a cell**: a cell a layer below its next cell in the road
    is a sloped cell (design 10), never a step; a one-cell dip is raised a
    layer on packed earth (`RoadCell::raised`);
  - **the steepest grade** is a tunable (`max_grade`, a layer every 1 cell by
    default, 19 degrees; every 2 cells is 10, every 3 is 7): two rises closer
    than it are refused;
  - **wide roads on one plane**: every cell of a road two or three wide takes
    its top from the one surface along the centre line (design 10b), cut or
    filled to it within `max_cut_layers` and `max_fill_layers`;
  - **fords**: water at most `ford_depth_m` (1 m) over its bed is crossed on
    a gravel bed under the water, whatever the road's surface;
  - **bridges**: deeper water is bridged, a plank deck at the near bank's
    height for at most `max_bridge_cells` (6 for a player's road), shown in
    the ghost as a deck with its posts and rails.

**42b. Refusals: red on the clashing cells only.**
- A segment that breaks a rule is drawn with its allowed cells in the ghost
  tint and **only the cells that clash in red**, the first clash named in the
  HUD. Its stake cannot be driven: the place button shows the reason as a
  toast and takes nothing.
- The reasons (locale `roads.plan.*`), each taking only the cell it is about,
  never its whole column or segment (the house planner's rule, owner
  2026-11-23: "only apply to the block three is actually on"):

  | Clash | Cells red | Text |
  | --- | --- | --- |
  | more than the grade between two cells | the higher of the two | "Too steep" |
  | water deeper than a ford, past `max_bridge_cells` | the water cells past the limit | "Too wide to bridge" |
  | still water (a lake, the sea) | those cells | "Open water" |
  | a town's footprint (its lanes are road already) | those cells | "In a town" |
  | a tree on the cell | that cell | "A tree in the way" |
  | a player's block on the cell (not a road cell) | that cell | "Something is in the way" |
  | a segment longer than `max_stake_cells` (24) | the cells past 24 | "Too far" |
  | a wide road bending on a slope | the bend cell | "Wide roads bend on the level" |
  | a lane cut or filled past its limit | that lane cell | "Too steep across" |

- A tree is felled with the axe and a block dug, as anywhere; the ghost goes
  from red to the tint on the next aim, so the fix is seen working.

**42c. Width and surface.**
- **The planner menu** (a radial menu on the use key held with stakes in
  hand, R in the mockup) holds three choices for the plan: **width 1, 2 or
  3**, **dirt or gravel**, and **lamps on or off**. They apply to the plan's
  unpaved cells; the ghost and the bill change at once.
- **Width.** A wide road's lanes are the cells beside its centre line, to the
  left for 2 and both sides for 3 (design 10b). A lane's cells are marked and
  paved like the centre's.
- **Dirt or gravel.** Gravel costs a gravel a cell; **dirt costs nothing**
  but is walked at 1.0 against gravel's 1.15 and wheeled at 0.6
  (`land-transport` design 9). This brings back the player's dirt track that
  design 40a retired: a road planned whole makes the surface a menu choice,
  not a second way to lay a cell, and a dirt road is the cheap first road
  before the grinder (recommendation taken, ask only with screenshots; it
  reverses design 40a's "the player lays only gravel"). Fords are gravel
  either way.
- A paved cell keeps the surface it was paved with; changing the menu changes
  what is still to pave.

**42d. The bill and the turns.** While the planner is in hand, and whenever
the player is within `ghost_reach_m` of an open plan (as the house planner's
site HUD):
- **the bill** for what is still to build against the pack, short goods in
  red: survey stakes (one a waypoint), gravel (one a gravel cell and a ford
  cell, times `cost_scale`), planks (`bridge_planks`, 6 a bridge cell), logs
  (1 a bridge cell for its posts, 1 a lamp post) and street lanterns (1 a
  lamp);
- **the counts**: cells, metres, **turns** (each waypoint where the road's
  heading changes; a hex line's own zigzag along a diagonal is not a turn)
  and "12 of 38 left";
- the aim's segment shows what the next stake would add, so its cost is seen
  before it is driven.
- **As built (2026-11-25, task 14.5).** The bill and the HUD line are
  `planner::bill` and `planner::hud_line` (42l). The compass tick, which
  this decision did not draw, is the building site's (`building-planner`
  design 6), recommendation taken (ask only with screenshots): with the
  shovel in hand on foot, the nearest marked cell not yet paved within
  `ghost_reach_m` (60 m) is a red pin on the compass bar
  (`roads::paving::PaveTick`, set by `pave_tick`), after the site's next
  piece and before the player's waypoints, named "Next cell" with its
  distance when it is the one faced (`roads.pave.next`). The marks are
  laid out again only when the save changes. Pinned by the app test
  `the_bill_falls_a_gravel_a_cell_paved_and_a_short_good_is_red`: the bill
  falls by one gravel and one cell left for each cell paved, the line is
  red while the pack is short of gravel and not once it holds enough, and
  the tick moves on to the next cell as each is paved.

**42e. Lamps.** With lamps on, a lamp post stands beside every
`lamp_spacing`th cell (6) of the centre line, on the left verge, outside the
road. Its ghost stands with the plan; it is built (a log and a street lantern
taken) when the cell beside it is paved, and lights the road from then on, by
the cell light field like every lamp (`night_lights.ron`'s street lantern).
Turning lamps off drops the lamps not built; a built lamp stays until dug.

**42f. Editing.** With stakes in hand, aimed at the plan:
- **Grab a waypoint**: the place button on a stake picks it up; it follows
  the aim, its two segments recut live (refusals and all), and the place
  button sets it down where the new segments are allowed. Esc (the drop key in
  the game) puts it back.
- **Insert a waypoint**: the place button on a plan cell between two stakes
  drives a new stake there (a stake from the pack) and picks it up, so the
  road can be pulled aside at that point.
- **Delete a waypoint**: the dig button on a stake takes it back into the
  pack; its two neighbours are joined by one segment. If that join would be
  refused, the stake stays and the reason is shown: a plan never holds a
  refused segment.
- **Edits mid-build follow the house planner's rule** (`building-planner`
  7): the new route is diffed against the old by cell key:
  - in both: unchanged, paved stays paved (re-cut to the new order if its
    neighbours changed, at no cost);
  - only in the new: a new marked cell, a ghost;
  - only in the old, not paved: unmarked;
  - **only in the old, paved: marked for removal** (`removal` "mark", the
    default): drawn under a pale chalk wash, still walked, and taken up with
    the shovel like any road cell of the player's, giving back
    `refund_share` of its gravel (1.0 by default); with `removal` "return"
    the cells go back to their ground at once on the edit and the gravel goes
    to the pack (a crate at the last stake when the pack is full).
  - While a grabbed waypoint would drop paved cells, the HUD names what
    setting it down does: "4 paved cells to take up, 4 gravel back".
- Narrowing the width is an edit of the same kind: paved lane cells the
  narrower road drops are marked for removal.

**42g. Building it.** Shovel in hand (design 40): the use key on a marked
cell within `reach_m` (4 m) paves it, held paves the next one in reach on the
shovel's beat. The ghost of a paved cell goes and the road is there, so
**what is left is what is still ghost**; the compass bar shows the nearest
unpaved cell of the open plan as a tick (as the house planner's site does). A
bridge cell is built the same way with planks and a log in the pack (its
deck, posts and rails at once), and a lamp when its cell is paved. A refused
cell can never be in a plan, so nothing paved is ever refused.

**42h. The plan as a record.** `road-plan` goes to schema 2: the waypoints
(cell keys, in order), the width, the surface, lamps on or off, the paved
cells' keys with the surface each was paved with, and the cells marked for
removal. The route's cells are re-derived from the waypoints by the hex line,
a pure function of the keys fixed forever, so a plan reads back cell for cell
whatever later road versions do; the cut of each paved cell is stored in its
`road-cell` record (design 40). Written through the durable path with the
pack in the same line on every stake driven, moved or taken back, every menu
change and every cell paved. A schema 1 plan (design 39b: stakes and
searched cells) reads as a width 1 gravel plan whose route is its stored
cells.

**42i. Tunables.** `assets/config/road_planner.ron`, validated, units named:

| Tunable | Default | Unit | What it changes |
| --- | --- | --- | --- |
| `ghost_opacity` | 0.35 | 0..1 | the plan's ghost (the aim's segment at 0.7 of it) |
| `ghost_tint` | blueprint cyan, as the house planner's | linear RGB | the ghost |
| `refuse_tint` | brick red | linear RGB | a clashing cell |
| `max_grade` | 1 | cells a layer | the steepest road (1: 19 degrees) |
| `max_stake_cells` | 24 | cells | the longest segment |
| `max_bridge_cells` | 6 | cells | the longest bridge a player builds |
| `stake_reach_m` | 6 | m | how far a stake is driven |
| `reach_m` | 4 | m | how far the shovel paves |
| `ghost_reach_m` | 60 | m | how near the ghost and the bill show |
| `cost_scale` | 1.0 | x | every cost |
| `bridge_planks` | 6 | planks | a bridge cell |
| `lamp_spacing` | 6 | cells | between lamp posts |
| `removal` | mark | mark / return | paved cells an edit drops |
| `refund_share` | 1.0 | 0..1 | gravel back from a cell taken up |

**42j. The mockup (M15 version 5).** `towns.html#roadplanner`: a meadow
rising east in one-layer terraces with a two-layer bank, a brook shallow
enough to ford at its north end and deep to the south, a lake, a walled
hamlet, trees, and the worksite with the grinder and the mixer. A plan is
staked out from the worksite (part paved, part marked, part ghost) and open,
so the ghost follows the aim. The page's own walker; 1 stakes, 2 shovel, 3
axe; a click drives, grabs, inserts, paves and fells; X or right click
deletes; R the menu; Enter ends the plan; the same by clicks in the
overview. Keep paving walks the line and paves it; Edit mid-build moves a
waypoint over paved cells to show the removal rule. Sliders: ghost strength,
the steepest grade, the cost scale, the lamp spacing and the share of gravel
returned. By day and at night, when the built lamps light the road.

**42k. A road by its two ends (owner, 2026-11-24: "road planner: needs to
allow the player to click on start and end segment"; then "you still have
the plan auto start in road planner ... when entering it, you have to click
on the ground to start a segment, and you click again to end it. that way
you arent stuck accidentally clicking around and generating random
segments", and "add a eraser to delete segments too").** This replaces
design 42's open plan that every click extended:
- **Nothing is planned on entering.** No plan is open and no ghost follows
  the aim until the player starts a road.
- **Two clicks make a road.** With stakes in hand, a click on the ground
  sets a road's start (one stake); the ghost then follows the aim from it,
  with its red reasons (a tree, open water, too steep); the next click sets
  its end, and the road between is the straight hex line, staked wherever
  a leg would pass `max_segment` cells. A click on a clashing end does
  nothing and says why. Then the planner is idle again: a click elsewhere
  starts another road, never extends this one. The end may be anywhere the
  aim reaches.
- **A plan is a list of roads**, each its own segment with its own width;
  the `road-plan` record holds them as `roads`, each with its waypoints.
- **Roads are added, never replaced (owner, 2026-11-24: "two-click is
  additive", "you add segments not replace", "if i click on a stake that
  means I want to extend another segment").** A click on a stake of any road
  starts a new road from it, joined there and sharing that stake; an end
  clicked on another road's stake joins it the same way. Shift and a click
  on a stake picks it up to move it.
- **The tools carry their icons** (owner: "I expected icons for a more user
  friendly gui"): the game's survey stakes, shovel and axe, and the
  planner's rubber for the eraser.
- Clicking the start again, Esc, or changing tool drops a road that has no
  end yet. Right-click on a stake takes it back; a road of two stakes goes
  with it.
- **The eraser (4)** deletes the whole road under the aim: its stakes come
  back to the pack, and its paved cells are taken up by the removal rule
  (marked, or given back), unless another road still runs over them.

Shown in M15 version 10 (version 8 had a New road button on the old open plan; version 9 picked a stake up on a plain click).

**42l. In the game (2026-11-24, on the owner's "cool implement all the
phase 3 stuff!"; written before the code).** Where 42 to 42k leave a choice
open, each is a recommendation taken (ask only with screenshots):
- **The hex line is the same both ways round.** `roads::hex_line` samples
  the great circle between the two cells every quarter metre, always from
  the lower key to the higher (reversed after for the other way), takes the
  nearest cell at each sample, and where two cells in a row are not
  neighbours (the line through a corner) puts in their common neighbour of
  lower key. This replaces 42a's "ties broken to the left of travel", which
  would give a different line each way round.
- **One rule for a cell and a step.** `roads::check_cell` (a planned cell's
  ground and whether a tree stands on it) and `roads::check_step` (two
  neighbours' heights against the steepest grade) are what `roads::route`
  asks too: the player's edits and the sea are never entered, and no step
  is more than a layer. The world's roads plan the same cells as before
  (`world_routes_are_pinned`, taken before the change).
- **Bridges are not built yet.** A river deeper than a ford is red, "Too
  wide to bridge", whatever its width, until the player's bridge cell (its
  deck, posts and rails) is built. Shallow water is forded on a gravel bed,
  whatever the road's surface (42a), and a paved ford is drawn as the
  world's fords are.
- **Lanes.** A wide road's lanes are its centre line moved one lattice
  step to its left (width 2), or to both sides (width 3): the step whose
  cell lies most square to the road, so a lane is the centre's own line
  beside it, cell for cell, with no gap and no cell twice, even where the
  line zigzags across the lattice. A lane takes the height of its
  nearest centre cell. The player's road neither cuts nor fills yet (that
  needs the world's digging in the paving's durable line), so a lane is
  allowed only where its ground is on that height, or a layer under it,
  raised on packed earth as a dip is; otherwise "Too steep across". **On a
  slope a lane is refused for now**: the row beside a sloped cell is half a
  cell along it, so sharing design 10b's one plane needs a bed half a layer
  high, which no road cell has yet. A lane drawn on its own slope would
  meet its neighbour with a seam of up to half a layer, a step, which the
  roads never have; so a wide road is built on the level, and climbs as a
  one-wide road, until the half-layer beds are built (task 8.6 stays open
  for them and for the world's wide roads). A wide road bends only where
  the bend cell and its neighbours on the line are level ("Wide roads bend
  on the level").
- **The plan record.** `road-plan` schema 2 is `{ roads: [{ stakes,
  width, surface, lamps }] }`; its cells are re-derived from the stakes. A
  schema 1 plan reads as one road of width 1 in gravel whose cells are the
  ones it stored (`cells`, kept on the road), so it never moves. The
  planner writes into the plan whose stake a road starts from, else the
  newest plan, else a new one. Paved cells stay `road-cell` records naming
  their plan; **marked for removal is derived**: a paved cell of a plan
  that no road of it runs over now. So it is not stored twice.
- **The menu is X held** (with stakes in hand, on foot; F is the shovel's
  paving key and G the tool picker). It lists width 1, 2 and 3, dirt,
  gravel, lamps, and the eraser; the wheel moves the highlight and letting
  X go takes it, as the G picker does. Width, surface and lamps are for the
  next road started; a road already staked is changed by erasing it (its
  stakes come back) and staking it again.
- **Esc** opens the menu as ever, and drops a road with no end yet; so do
  clicking its start again and putting the stakes away.
- **The eraser** stays on until it is picked again in the menu or the
  stakes leave the hand; a click erases the road under the aim.
- **Lamps** are recorded on the road, stand in its ghost and are in its
  bill; a lamp post is not built when its cell is paved yet (left open).
- **The HUD** is one line under the crosshair: the red reason while the
  aim is refused, otherwise the bill of what is still to build (stakes,
  gravel; short goods in red) and the counts ("12 of 38 left", turns).
- **Tunables** are `assets/config/road_planner.ron` (42i), validated at
  load; `max_segment` (24, the cells between two stakes) is 42i's
  `max_stake_cells`.

**42m. Wide roads on a slope, bridges, lamps and inserting a stake
(2026-11-24, on the owner's "cool implement all the phase 3 stuff!";
written before the code).** What 42l left open, each a recommendation
taken where 10b and 42 to 42k leave a choice (ask only with screenshots):
- **One shared plane.** A road's plane is set by its centre line as the
  one-wide rule sets it (design 10a): at each edge between two centre cells
  the higher cell's ground, at a centre cell's middle the mean of its two
  edges, a whole or a half layer at each of those points and straight
  between. Every cell of the road, centre or lane, takes its top off that
  plane **where the cell really is**: the middles of its sides toward the
  cells it comes in and leaves by are placed along the centre line and the
  plane read there. On a regular lattice a lane is exactly half a cell
  along its centre cell; on the sphere the lattice is not exactly regular
  (measured: a lane a third, not a half, of a cell along), and a lane read
  off a half-cell grid met its neighbours with seams of 15 cm. Read where
  it is, the seam is under a centimetre (`a_wide_road_up_ten_layers_is_
  crossed_lane_to_lane_without_a_step`).
- **A lane's top** is a lift over its whole layer and a rise over part of
  its length, level either side (`RoadCell::bed`: `lift_mm`, `from`, `to`,
  `rise_mm`; all nought is the plain one-wide cell). That covers the
  one-wide slope, the half-height bed 42l found missing (a lift of up to a
  layer), and the creases where the plane is held level past the foot and
  the top of the climb (10b). The walker stands on it through the same
  `Steps` floor a slope is, its foot and length cut to the part that rises,
  and the mesh is the same fan.
- **Fill, never cut, and the plane moved half a cell where that saves a
  cut.** The player's road still digs nothing (42l). A lane cell whose
  ground is under the plane's lowest point on it is filled to the plane on
  a bed of packed earth (less than two layers: `raised` and the bed's
  `lift_mm`); one whose ground stands over it would need a cut, and is red,
  "Too steep across". On an even hill a lane off the centre's step is
  part of a layer off its ground, so with the plane where the one-wide rule
  puts it a lane can need a cut (10b's own measurement). So the planner
  tries the plane **moved down the hill** half a cell, then a whole cell
  (each point of the half-cell grid the higher of its two neighbours, the
  ends of the road held where they are): the lanes then sit on their ground
  or a little over it, and the centre cells take a bed instead. It takes
  the least moved plane that needs fewest cuts and still climbs no faster
  than a layer a cell. A one-wide road keeps today's plane cell for cell.
- **Straight up a slope.** 10b's lanes share a plane only along a lattice
  row. A wide road whose centre line turns, or zigzags across the lattice,
  on a cell that rises or beside one, is red there, "Wide roads bend on the
  level"; on the level it may run any way.
- **Bridges, one rule.** A river cell deeper than a ford, and any river cell
  after one while a bridge is carried, is bridged with a plank deck at the
  road's height on the bank before it, for at most `max_bridge_cells` (6, a
  new tunable in `road_planner.ron`) and only where the deck stands over the
  water: `roads::planner::bridged`, which the world's search asks as well,
  so the two agree (the world's `max_bridge_cells` stays 10; its routes are
  pinned by `world_routes_are_pinned`). The water past the limit, or under a
  deck that would be under the water, is red, "Too wide to bridge". The far
  bank meets the deck within a layer or is red, "Too steep". A bridge cell
  is paved with `bridge_planks` (6, new) planks and a log, its deck, posts
  and rails at once (the world's bridge piece), and stored on its
  `road-cell` as `deck_m`.
- **Wide bridges.** Every lane cell beside a bridged centre cell is a
  bridge cell at the same deck where its ground (land or a river's bed) is
  under the deck; level with it, a road cell; over it, red. Rails run only
  along a bridge cell's sides that have no road cell beside them, so a wide
  bridge is railed on its outer edges and open between its lanes (10b). A
  lane over deep water beside a centre that is not bridged is red, "Too
  wide to bridge".
- **Lamps built on paving.** Lamp posts stand off the road on its verge:
  for every `lamp_spacing`th centre cell, on the cell beside the road's
  outermost left cell most to the left that is not the road's, dry, no
  town's, and within a layer of that road cell's ground; a three-wide road
  also has them on its right, half a spacing on (staggered, 10b). A post is
  the town's own street lantern (`LanternPost`, lit at dusk by the cell
  light field like every town lamp), placed as the player's block in the
  first air layer over its cell's ground when the road cell beside it is
  paved, a log and a street lantern taken from the pack in the same line
  (`WorldSave::accept`). Short of either, the cell is paved and the lamp
  waits: the use key with the shovel on that paved cell builds it once they
  are in the pack. A built lamp is a block like any other, and is dug like
  one. Its ghost stands with the plan (a post in the ghost tint), and the
  bill counts the posts not built and their logs and lanterns, and a bridge
  cell's planks and log.
- **Inserting a stake (42f).** Shift and a click on a road cell that is not
  a stake drives a stake there (one from the pack) into the leg it lies on
  and picks it up, so the road is pulled aside from that point as a moved
  stake is: the next click sets it down where both new legs are allowed.
  Refused, with the reason, where the leg through the new stake would
  clash.
- **The walker across the lanes.** Pinned by a `pbd-core` test that walks a
  two- and a three-wide road up ten layers diagonally across every lane and
  back, on the drawn floors, and finds no rise steeper than a slope's.
- **The world's wide roads stay one wide.** 10b asks the world to lay its
  walled towns' gravel roads two wide. That is a new roads version (a new
  `road` schema for the lanes' beds), for new worlds only, with version 3
  pinned by its digest. Left open in task 8.6; nothing in the world's
  planning changes here.

**42n. The owner's answers on the planner (survey P2, 2026-11-24).**
- **A stronger ghost**: `ghost_opacity` 0.55, up from 0.35.
- **A slightly aerial view** (owner: "Can we something like a slightly
  aerial view? Let's say something like 10-20 m off the ground?"). While
  survey stakes are in hand on foot, the eye rises to `aerial_m` (15 m, a
  validated tunable from 5 to 30) over the player's eye and stands a little
  behind them along the way they face, looking down at the ground ahead,
  so the ghost and the stakes read whole. It eases up over `aerial_ease_s`
  (0.4 s) when the stakes come into the hand and back down when they leave
  it, never a jump cut. Only the eye's height and place ease: mouse look
  turns the view by the frame's raw movement with no smoothing, as
  everywhere (CLAUDE.md), and the pitch is the player's own pitch tipped
  down by the angle that keeps the spot they aimed at from the ground
  where it was. The walker, its collision and the reach of the shovel are
  unchanged; only the eye moves.
- **Picking from the lifted eye**: the aimed cell is found by the same ray
  the view looks along, cast from the lifted eye, so a click lands on the
  cell under the crosshair. While lifted, the ray reaches as far as the
  ghost shows (`ghost_reach_m` from the player), since the point of the
  lifted view is to stake what it shows; the shovel still paves only
  within `reach_m` of the player's feet.

**42o. The ghost's bridges and lamp posts, and the trader on the map
(2026-11-25; written before the code).** Three things 42a, 42m and 46 ask
for that the game did not yet draw. Each a recommendation taken (ask only
with screenshots): none changes a rule, a tunable or what is built, only
what the planner's ghost and the map show of it.
- **A bridge in the ghost.** 42a: deeper water is "shown in the ghost as a
  deck with its posts and rails". Today a bridged cell is in the ghost as
  a flat top at its deck's height, with nothing under it. A bridge cell of
  the aimed road that is not refused is now cut by the same
  `roads::piece::cut` that cuts it once paved (its deck, the posts from
  the bed to the deck, the rails along its sides with no cell of the aimed
  road beside them, so a wide bridge is railed outside), and every part of
  it is drawn in the ghost's tint. The ghost then shows what paving will
  build, from the one cutting code, never a second picture of a bridge.
- **Lamp posts in the ghost.** 42m: "Its ghost stands with the plan (a
  post in the ghost tint)". The aim already finds the posts
  (`PlannerAim::lamps`); they are now drawn: on each post cell not already
  holding its lamp (`paving::lamp_standing`), on the cell's ground, one
  shape in the ghost tint at the street lantern's size as the world draws
  it (a post 1.5 m high, a lantern 0.3 m across from 1.5 to 1.86 m): a
  six-sided timber post, bevelled at its foot and swelling into the
  lantern's housing, lofted as one piece with no part stuck on (CLAUDE.md,
  no obvious primitives), its smooth normals from the loft. It is a
  ghost: the tint and its opacity are the plan's, lit by the field as the
  rest of the ghost is, and it goes with the ghost.
- **The trader on the map.** 46 places the escort job's trader on the
  road, and the map and the compass mark a lost villager (design 9) but
  not them. Now each taken escort job not yet done marks its trader where
  they stand (`road_jobs::trader_point` of the stored record, so where
  they stopped on a reload): on the map a pin with "Trader" beside it
  (`jobs.escort_marker`), as the lost villager's pin is, and on the
  compass bar a pin after the lost villagers', named "Trader" with its
  distance when faced. The record is taken back when the trader reaches
  the other town, and the marker goes with it.
- **Tests.** App tests: the ghost of a bridged road carries the bridge
  piece's deck, posts and rails (more of the mesh than the cells' tops
  alone, and reaching under the deck), and a lamp post stands in it on its
  cell's ground up to the lantern's height; a lamp already standing is not
  drawn. The escort test reads the trader's marker where the stored record
  puts them, and none once the job is done.
- **Captures.** The ghost of the bridge from the bridge capture's spot, and
  of a road with lamps, by day.

**42o, as built (2026-11-25).** As written above. Tests:
`roads::tests::the_ghost_carries_a_bridge_s_deck_posts_and_rails_and_its_lamp_posts`
and the escort test's marker checks
(`economy::tests::a_trader_leads_the_player_along_the_road_and_the_escort_is_paid`).
The capture instrument holds the planner's aim on the checked road's far
end (`roads::planner::AimPin`, set only by `--road-plan ghost-*`), opens it
from its near end (a bridge is laid from the bank it starts on: the other
way round the same cells were "Too wide to bridge"), and logs its stand
point's `--yaw` by `place::yaw_of`. Captures: `road-planner-ghost-lamps-1100.jpg`
and its close crop. The ghost of the bridge is not captured: at the bridge
spot the lifted eye (42n) looks down on forest canopy over the road.

**Found in the captures (2026-11-25; not fixed, for the owner).**
- **A deck awash.** `road-bridge.jpg`: where a river runs bank-full, the
  deck (the bank's height, `LIFT_M` over it) stands at the water's surface,
  and the river draws its ripples over the planks. The rule that a deck
  stands over the water (`planner::bridged`) holds by a hair. Fixing it is
  either a render change to the water under a deck, which waits for the
  session that owns rendering, or the deck raised off its bank (a step or
  a ramp onto every bridge, the world's included): a look change, for the
  survey with this screenshot. Recommendation: the deck a quarter metre
  over the water where the river is that full, its ends ramped on the
  bank's cell, as a new road version for the world's bridges.
- **Rails as panels.** The bridge's rails are solid timber panels a metre
  high along each open side, as the approved roads mockup draws them
  (`mockup-roads-bridge-11h00.jpg`). Close up they box the deck in like a
  trough, which sits uneasily with "no obvious primitives" (2026-11-22).
  A rail of posts and a top bar is the alternative; it is a look change
  for the owner, with this screenshot.

**42p. The owner's answers on bridges (survey P6 and P7, 2026-11-26;
written before the code).**

*P6, a deck awash.* The owner picked "Raise it about 0.25 m over full
rivers" and wrote: "River surface seems to be right on the surface of top
of terrain hex. You could offset water surface a bit to account for that
instead of raising bridge". So no road version and no raised deck: the
river is drawn lower.
- **Measured.** The water mesher (`water_flow::cell_pieces`, Luanti's
  corner rule) draws a top's corner at the full layer wherever a source
  (`Water`) is one of the three cells round it (`corner_level`), so a
  bank-full river's surface is drawn at its layer's top, `layer_altitude +
  1.0`, which is exactly the bank cell's ground top: an offset of 0.00 m.
  The deck stands at the bank's height plus `LIFT_M` (0.02 m) and is
  `DECK_M` (0.2 m) thick, so its underside is 0.18 m under the water and
  the ripples are drawn through the planks.
- **The change.** A drawn top's corner level is scaled by `1 - STILL_DROP`,
  `STILL_DROP` 0.2 of a layer (0.2 m): a full corner is drawn 0.2 m under
  its bank, at the deck's underside, so the planks stand 0.22 m over the
  water, and a riffle's lower corners come down in proportion, so no reach
  draws higher than the still water upstream of it. A side's top edge
  follows its corners, so the sides meet the top.
- **What it leaves alone.** Every corner that touches a fall is drawn as
  it is today, so a fall keeps its lip, its sheet and its foot exactly (the
  waterfall rule): a corner is left whole where any of its three cells has
  falling water in that layer, falling water on it (a fall's foot), or is
  open with falling water under it (a lip). Water standing in a stack (the
  layers under a top) is never scaled. The sea is not touched: its sheet
  draws its top, and water at the sea's own layer is left whole so a river
  meets the sea level. Two cells sharing a corner work it out from the same
  three cells, so the top stays one surface with no crack.
- **What reads it.** It is a draw-time rule: the column, the rule's levels,
  the tick, the save, swimming and buoyancy are unchanged. The readers of
  the drawn top follow it through `water_flow::drawn_top_m`, which applies
  the same scaling: whether the eye is under the water, the rain's
  ripples, the eye's path. A raft floats on the drawn top
  (`PlanetContact::drawn_water`), which no longer stops at the column's
  level but at most `STILL_DROP` under it, so a raft sits on the water it
  is drawn on. The walker swims to the rule's surface, as before: a
  swimmer's float line is up to 0.2 m over the drawn water in a full river.
- **Caches.** The river band's cached pieces (`river_band`, a file beside
  the worlds) were meshed at the old height; its file's magic moves to
  `PBDBAND2`, so an old file is read as none and the band is meshed again.
  Derived data only: no save changes.
- **Tests.** App tests: a full river's drawn top is `STILL_DROP` under its
  bank's top, and `drawn_top_m` says the same; a fall's lip and foot
  corners are where they were.
- **Captures.** `road-bridge-after.jpg` from `road-bridge.jpg`'s spot, and
  a river bank close up before and after.

*P7, the rails.* The owner picked "Keep the solid panels" and wrote: "You
should use a texture with alpha cutout instead of adding geom (single
sided, hand rails and posts and the land parts)". So the panels' look
stays (a timber rail along each open side), drawn as flat cards in a
cut-out texture instead of modelled boxes.
- **The cards.** A bridge cell's railed side gets two:
  - **the rail**, from the deck's top to `RAIL_M` (1 m) over it, along
    the side, in `bridgerail`: a post at each end, a hand rail on top, a
    mid rail and the bottom rail, the gaps clear;
  - **the trestle** (the land part: the bridge's legs, standing on the
    bed or the bank), from the deck's underside down to the cell's bed, in
    `bridgetrestle`: a leg at each end and cross braces, one repeat a
    metre, the gaps clear.
  Each is mapped once across the side (its posts or legs land on the
  side's corners, where the next side's begin), nearest sampled.
- **Single sided, read from both sides.** Each card is two single-sided
  quads back to back on the same plane, one facing out over the water and
  one facing in over the deck, each culled from behind, so from the deck
  and from the bank the card is seen from its front, lit by its own
  normal, and the two never fight for a pixel (only one faces the eye).
  One double-sided quad would be lit by one normal on both faces, dark on
  the side away from the sun; two thin boxes are the geometry the owner
  asked to do without.
- **The texture.** `tools/art/bridge_cards.py` draws both, 64 x 32, in the
  timber's and the plank's own palette (each pixel snapped to theirs), the
  grain and the pegs painted in, no outline, the clear texels at alpha 0;
  rows in the settlement manifest with `cut: true`. The world's roads draw
  a texture the manifest cuts in the masked town material (`AlphaMode::Mask`),
  as the net is.
- **What stays.** The deck, its timber sides and its underside are as
  they were. The rail's solid (what stops the walker) is as it was, so
  walking and the paving rule are unchanged; the cut is the same
  `roads::piece::cut` for the paved road and the planner's ghost.
- **The ghost.** The ghost draws the cards in its own tint and opacity in
  each card's texture, blended, so the holes are clear in the ghost too:
  the ghost's meshes are split by texture (`planner::ghost_cards`), the
  cells' in the gravel as before.
- **Tests.** Core: a bridge cell railed on four sides cuts four rail cards
  and four trestle cards, each two quads (four triangles), in the two card
  textures; no timber box faces stand over the deck; every card's quads
  face opposite ways. App: the ghost carries the cards in their textures.
- **Captures.** The bridge close up by day and at night.

**46. Jobs on the roads: a rockfall and a trader to walk with (task 8.5,
2026-11-24; written before the code).** Two new kinds on a town's board,
through the board, the taken jobs, the counting and the hand-in every town
job already goes through (design 8, 28, 29); posted only by a town a road
of the world's leaves (`jobs::Town::roads_to`, the towns joined to it by a
stored `road`), so neither is posted where there is no road.
- **Rockfall (`rockfall`, `Clear`).** Taking the job has the world drop
  rock on its road: from the road's cell `rockfall_from` (8) cells out of
  the town, a run of the job's count (2 to 4) cells, each a stone block in
  the first air layer over the cell's ground, proposed by the world
  (`Author::World`, process `rockfall`, the job's id) through
  `WorldSave::propose`, so the yield check skips any cell the player has
  touched. The cells are stored as a `road_rockfall` record (by the job's
  id: the town, the cells and how many are clear). The job counts a cell
  (`Clear`, `<site>:road`) when its rock is dug: the game compares the
  cells' edits with the record each frame and writes a `Did` for each cell
  newly clear, then the record's count, so a reload never counts a cell
  twice. A road cell the player dug its rockfall off is drawn and stood on
  again: the road yields to the player's work except where that work is
  only a rockfall's rock taken back to air. `Clear` joins the live kinds.
- **Escort a trader (`escort`, `Escort`).** Taking the job puts a trader
  (a person of the town rolled as the lost villager is, never its trader or
  job giver) on the town's first road cell toward the job's town, stored
  as a `road_escort` record (the town, the other town, who, the road's cells
  in order and how far along them they are). The lost villager's follow
  turned round (`follow::lead`): once the player is within `LEAD_START_M`
  (6 m) the trader walks the road's cells at a walker's pace while the
  player keeps within `WAIT_M` (12 m), and waits where they stand when the
  player falls further back, facing them. Reaching the other town's
  footprint counts the job (`Escort`, `<to>:trader`), the record is taken
  back and the trader goes home to their post; the job is handed in at the
  town that gave it, as a delivery is. Their place along the road is
  written when they stop and at most once a second as they walk.
- **Pay and counts** in `jobs.ron`: rockfall counts 2 to 4 cells at 8 coins
  a cell, escort 1 at 40, both times the board's `pay_share`; both join
  `posted` (the boards change, which no save stores).
- **Tests** play each through the player's own inputs where the app tests
  allow: the job taken from the board, the rock dug with the same edit the
  dig makes, the walker moved along the road with the trader, the job
  handed in and paid.

**47. A parcel's pick-up and drop-off (playtest, 2026-11-25; written
before the code).** The owner: "Carry a parcel" has no place to pick the
parcel up or drop it off. Today taking a delivery puts the parcel straight
into the pack (`Shop::take`), talking to anyone in the job's town hands it
over (`Shop::deliver`), and nothing on the compass or the map points the
way.
- **Pick-up.** Taking a delivery puts no parcel in the pack. The parcel
  waits with the posting town's trader (`Keepers::trader`): talking to them
  hands over every parcel waiting there, one pack slot each (a full pack
  keeps it waiting, and the talk says "No room in the pack"). Where the
  trader is the job giver too, the parcel is handed over as the job is
  taken. The talk's line is "Here is the parcel".
- **What is stored.** `TakenJobs` gains `waiting: BTreeSet<u64>` (serde
  default, empty), the deliveries whose parcel is still with the trader.
  Taking a delivery adds its id; the pick-up takes it out in the same line
  that puts the parcel in the pack (`store_with_hands`). An abandoned
  delivery still waiting takes nothing from the pack.
- **Saves.** A record written before this has no `waiting`, so every
  delivery in it reads as picked up: a parcel taken today stays in the pack
  and is handed over as before. The schema stays 1 (a defaulted field).
- **Drop-off.** Talking to anyone in the job's town still hands it over,
  but only once it is picked up. A delivery picked up and not yet done
  marks its town on the compass bar and the map, as a lost villager's job
  marks the villager (`jobs.parcel_marker`, "Parcel drop-off"), at the
  town's centre.
- **The tracker** (playtest-fixes design 2) names the step: "Get the
  parcel from the trader" while it waits, "Take the parcel to {town}"
  after, "Hand it in at {town}" once delivered.
- **Tests.** Taking a delivery leaves the pack as it was and the job
  waiting; talking to the trader puts the parcel in the pack; the
  destination's people take it only after; a record without `waiting`
  reads every delivery as picked up, keeps its parcel and gains the marker.

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
- [Face pieces z-fighting or hidden by hair] → The face pieces sit 2 mm
  proud of the head and are drawn after it; hair never covers the eyes
  (Wind Waker's villagers show the same rule).
- [Cute against the game's look] → The proposed figure is shown beside
  today's hex figure in the same light; the owner picks.
- [Sloped cells and the rest of the engine] → A slope is a top shape on an
  ordinary cell: the cell's material, light and saving are unchanged, and
  only the mesher, collision and the walker's ground height read the slope.

## Migration Plan

- There is no save change, because people are derived.
- Rollback is the previous build: the towns are empty of people again.
