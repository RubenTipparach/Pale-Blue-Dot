# Design: industry tiers

Everything here is proposed. Numbers are the mockup's recommendations to tune,
not measurements of the game. Recommendations are taken where nothing can be
compared side by side (CLAUDE.md, "Ask only with screenshots"); each is
marked so.

## 1. Strip mining: five authors of the same edit

A pit is still cut only where the player marked cells, a 1 m layer at a time
(M20's I2). What changes with the tier is who digs, how fast, how hard a rock
it breaks and how far it reaches. Every dug cell is an edit on the durable
path naming its author (the crew's or the machine's ID), exactly like a
player's dig.

| Tier | Who digs | Unlocked by | Rate | Hardest rock | Reach | Runs on |
| --- | --- | --- | ---: | ---: | --- | --- |
| 1 | **A hired crew**: 2 to 6 townsfolk with picks and barrows, a tent and a pay chest | Nothing: wages | 1 cell per 3 min per worker | 2.2 (no iron ore) | Any marked cell they can walk to | 4 coins a worker a game day |
| 2 | **Excavator**: tracked, diesel, one operator | Tier I (oil) | 1 cell per 40 s | 3.0 | 3 cells round where it stands; it drives to the marks | Diesel, 12 kg a game hour (a 100 kW engine) |
| 3 | **Quarry frame**: an orange gantry round a hex of marked cells, a drill head on its rails | Tier I to II grid | 1 cell per 8 s | 3.0 | Every cell inside the frame, radius up to 4 (61 cells), down to layer -24 | 120 kW |
| 4 | **Bucket-wheel excavator**: crawlers, a boom with the wheel, a counterweight, a spoil conveyor | Tier III (fission) grid | 40 cells a minute along a bench face | 3.0 | A bench face up to 30 m from its crawlers; it walks the bench | 4,000 kW down a trailing cable |
| 5 | **Mining drones**: a hive of 12 | Tier IV (fusion) | 1 cell per 45 s each, 16 a minute | Any | Any marked cell within 300 m of the hive: cliffs, overhangs, a cave roof | 30 kW per drone charging, 360 kW |

Rules, all recommendations taken:

- **Crews are townsfolk.** A crew is hired at a town (the `townsfolk`
  change), walks to the pit, works by day and walks home at night. A lit pit
  (lamps within 4 cells of every marked cell) and double wages keep a night
  shift. Unpaid, a crew stops and goes home; nothing they dug is undone.
  They carry dug cells to a chest or a belt, so a crew's pit wants a belt
  head nearby.
- **The excavator is M20's digger**, named for what it is. It drives on the
  pit's benches (a 1-layer step at most) and refuels from a tank or a
  pipe.
- **The quarry frame** is built of frame cells round the area, one layer
  above the ground. It digs every cell inside it, top layer first, unmarked
  or not: building the frame is the mark. Its head drops what it digs into a
  hopper at a frame corner with a belt or tube out.
- **The bucket-wheel excavator** is a stored multi-cell machine (37 cells
  of crawlers and house, a 30 m boom). It takes a whole bench a layer at a
  time as it walks along it, and its spoil conveyor drops onto a belt or a
  spreader. It needs a bench at least 4 cells wide to stand on.
- **Drones** fly from a hive (7 cells, 2 layers) to a marked cell, cut it
  and carry it back; a drone with no charge returns. Their reach makes them
  the tier for mountains and for small bodies (Taper, the belt rocks) where
  nothing heavy can be landed.
- **Yields do not change with the tier** (M20's strata table): a cell is a
  cell. A faster tier is more cells a minute, not more ore a cell.

## 2. Oil: the rig on land, the platform at sea

M20's oil (`steer-and-sail` decision 5) is a seed-derived field of reservoir
pressure, a pumpjack over it, a distillation column and an oil plant. This
adds the two machines the owner named.

- **Drilling rig** (onshore): a derrick on 4 cells over the field, 22 m
  tall, with its pipe rack, mud pits and a generator shed. It drills a well
  in 2 game days drawing 300 kW (or burning diesel at 36 kg an hour), then is
  moved on; the well it leaves takes a pumpjack. A well is a stored record
  as before; the rig is the thing that makes one.
- **Offshore platform: a rig and a power plant in one** (the owner's words).
  It stands on 4 piled legs in the shelf sea (10 to 60 m deep, the shelf
  where M20's field reaches the sea), its deck 19 cells.

  | Part | What it does | Number |
  | --- | --- | ---: |
  | Derrick | Drills up to 6 wells from the deck, risers to the seabed | 1 well per 3 game days |
  | Wells | Crude, as a pumpjack's (3 t a day each at full pressure, falling as drawn) | up to 18 t a day |
  | Gas turbines | Burn the gas that comes up with the crude, fuel oil topping up | 4,000 kW gross |
  | Own load | Drilling, pumps, quarters for 12, lights, crane | 900 kW |
  | Sea cable | To a shore substation, joining that grid | 3,100 kW exported |
  | Leg tanks | Crude stored in the legs | 2,000 t, a tanker every 4 days at full rate |
  | Flare | Burns what the turbines cannot use | the brightest light at sea |

  So one platform is three oil plants' worth of power and six wells' crude.
  Its gas never reaches the shore: power and crude do. It is placed only on
  the shelf over the field, never in deep sea, and needs a sea cable (built
  cell by cell along the seabed) for its power to count on land.
- **Night**: deck floodlights, quarters' windows, navigation lights on the
  legs, the flare; the sea round it dark.

## 3. Freighters: hired crews, then automated

A freighter is a ship from the drydock (M23) with cargo cells: container
cells (one container a cell, 20 t of goods) or tank cells (40 t of crude or
fuel a cell). It sails a **route**, the same stored record as ship-tiers'
space freighters: its ID, its stops, what it loads and unloads, a schedule.

- **Hired crew first.** A crew of 4 to 8 townsfolk sailors, hired at a port
  town, 6 coins each a game day. Near a player the ship sails its route in the
  world under the GPS's advice (steer-and-sail 3a); away from every player it
  moves along the route on rails, so it is where the route says when someone
  arrives. Unpaid, it finishes its leg and ties up.
- **Automated later**: at tier III a radio-navigation pilothouse replaces
  the crew; the route record does not change, only who sails it. Freighters
  in space are the fusion tier's (ship-tiers 5).
- **Cargo** loads and unloads through the durable save path, authored by the
  freighter, and never takes from a player's own store (CLAUDE.md, "The
  player is not the only author of change").
- **Kinds drawn**: a container ship for goods and a tanker for crude; the
  tanker is what empties the platform's leg tanks.

## 4. The four carriers

Everything that moves between machines moves on one of four carriers. Each is
built of cells (or on poles), one network per connected run, and each run
carries one kind of thing.

| Carrier | Carries | Built as | Capacity | Power | Rules |
| --- | --- | --- | ---: | ---: | --- |
| **Wire** | Electricity | Wood poles 4 cells apart; steel pylons 10 apart for high tension; buried cable through cells; sea cable on the seabed | 2,000 kW a line on poles; 50,000 kW on pylons | none | Poles in reach join one network that shares its supply (M20 section 4). A substation joins a pylon line to pole lines. |
| **Pipe** | Water, crude, fuel oil, diesel, propellant, liquid oxygen, steam | 1-cell segments on the ground, on stands, or through cells underground; valves and junctions | 0.5 t a second; large pipe 2 t | a pump every 20 cells, 10 kW | One fluid per network at a time; a valve splits a network. Tanks are pipe nodes. |
| **Conveyor belt** | Bulk: ore, coal, stone, dirt, crates | 1-cell segments, up one layer per 2 cells | 2 items a second; express 6 | 0.2 kW a cell | Splitters and mergers; a belt feeds a machine's input face. |
| **Pneumatic tube** | Small goods: plates, parts, circuits, ammunition for the station's needs, mail | 1-cell segments through walls, floors and up shafts | 1 capsule a second, 5 items a capsule, 15 m/s | 0.1 kW a cell, 5 kW a station | Each capsule is addressed to a named station; the fastest way across a base. |

Recommendations taken: one fluid per pipe network (no mixing rules to learn),
tubes addressed by station name (no sorting puzzles), belts as M20 already
draws them.

## 5. The loads on the ladder

The owner: "keep in mind for the power system, 3d printers, mining setups,
conveyor belts, launch pads, fuel systems". What each draws, so the ladder's
tiers are sized for it:

| Load | Draws | When |
| --- | ---: | --- |
| Hex printer (M22), printing | 60 kW | Tier I |
| Quarry frame | 120 kW | Tier I to II |
| Bucket-wheel excavator | 4,000 kW | Tier III |
| Drone hive (12 charging) | 360 kW | Tier IV |
| Conveyor belt, 100 cells | 20 kW | any |
| Pneumatic tubes, 100 cells and 4 stations | 30 kW | any |
| Pipe pumps, 100 cells | 50 kW | any |
| Drilling rig, drilling | 300 kW | Tier I |
| Launch pad: air separation unit (liquid oxygen) | 300 kW, 1 t of oxygen a game hour | Tier I to II |
| Launch pad: propellant blending and pad pumps | 200 kW while fuelling | Tier I to II |
| Fuel depot pumps (dock, airfield, filling station) | 30 kW | Tier I |

**The fuel system**: crude from wells and platforms, by pipe or tanker, to
the column; fuel oil and diesel by pipe to a tank farm; from the tanks by pipe
to the oil plant, to filling stations for vehicles and aircraft, to the dock
for ships, and to the launch pad. Rocket propellant (the VAB's 4.2 t a tank
cell) is kerosene cut from diesel plus liquid oxygen from the air separation
unit, 1 t of the cut to 2.4 t of oxygen; a First orbit stack's tanks take about
a game day of one unit's oxygen. Recommendation taken: one blended
propellant item, as the VAB already counts it.

## 6. The mockup

All in M20 (`docs/mockups/industry.html`), one 3D view at a time:

- **Strip mining**: a tier switch for crew, excavator, quarry frame,
  bucket-wheel and drones on the same pit, each at work when Run is pressed
  and still otherwise, lit at night (the crew's lanterns, the machines'
  lamps). The dig estimate uses the tier's rate.
- **Oil**: two more views: the drilling rig on the field, and the offshore
  platform at sea with its sea cable to a shore substation and a tanker
  alongside, lit at night.
- **Transport**, a new section: the four carriers running side by side
  between machines (a siphon's water and a fuel line in pipes, a tube with
  capsules, belts with ore, poles and a pylon line), and a dock with a
  container ship, hired or automated.
- **Power**: the platform joins tier I as a generator; the grid gains the
  loads of section 5.
- New icons and textures by `tools/art/industry_art.py` in the solitaire
  style; reference pictures the owner gave are references only, not copied
  (no real names or liveries).

## 7. Decisions

- Five mining tiers, the excavator being M20's digger: the owner's list.
- Rates, hardness, reach and power in section 1: recommendation taken.
- The platform as rig and power plant in one: the owner's words; its numbers
  (section 2): recommendation taken.
- Freighters on the ship-tiers route record, crewed then automated at tier
  III: recommendation taken.
- The four carriers and their rules (section 4): the owner's list;
  capacities recommendation taken.
- Loads (section 5): recommendation taken; the printer's 60 kW is this
  change's number until M22 gives one.
