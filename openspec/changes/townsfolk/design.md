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

**31. Roads the player lays (task 8.3).**
- **Gravel** is a good (`gravel`, stack 99), made by hand from loose stone
  (`gravel` recipe: two stone make four gravel) and valued in `values.ron`.
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
