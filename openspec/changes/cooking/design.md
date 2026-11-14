# Design: hearth and kitchen (cooking)

## Context

The proposal quotes the owner. This design fixes what the mockup shows and
the rules behind it, so the page, the data and a later build agree.

What the game gives us, measured on `main`:
- **The cell.** 2.833 m flat to flat and 1 m a layer on every body (CLAUDE.md,
  "Hex size is fixed"). A station stands on one cell; a storey is three
  layers (M6, `player-building`).
- **The inventory.** Ten hotbar slots and a thirty-slot pack
  (`inventory-grid`); `Item` is `Block`, `Tool` or `Fish(u16)`, `Copy`, saved
  as a short code; stacks merge by kind (`openspec/specs/player/inventory`).
- **Time.** A day is 48 minutes of play and a year 100 days
  (`orbit-and-seasons`); world time passes only while the world is played
  (CLAUDE.md).
- **What food could touch.** Walking and sprinting (8 and 14 m/s, halved in
  water, `swimming`); digging and break times (`dig-and-place`); fishing's
  line tension and fish stamina (`pbd_core::fishing`); lamps lighting what is
  near them (`lighting-model`). No hunger, health, breath or cold exists.
- **The upstream mockups.** M3 (stations, fuel as an ingredient, no random
  quality, recipes as data), M8 (crops, climates, seasons, compost), M7
  (animals and drops), M25 (water life and drops), M5 (townsfolk: farmer,
  fisher, trader, innkeeper), M12 (station goods and prices). The husbandry
  draft (`animal-husbandry`) owns taming, breeding and the butcher's block.

## Goals / Non-Goals

**Goals:**
- Every dish the owner named, with a chain a player can follow from the
  field, the pen or the sea.
- A kitchen that reads as this game: hex cells, the game's textures, its HUD,
  dark nights lit by the kitchen's own fires.
- Food that matters without nagging: a reward for cooking, never a penalty
  for not eating.
- Nothing that resets a save or changes the generator.

**Non-Goals:**
- Game code. This change is the write-up and the mockup.
- Animals: which are kept, bred and slaughtered (`animal-husbandry`).
- Health, breath or cold mechanics. Effects only touch what exists.
- Frame cost. Not measured here (a cloud container, SwiftShader).

## Decisions

### 1. Meals give buffs; there is no hunger bar (Q1, asked with screenshots)

A dish gives one effect for some minutes of play, scaled by its stars (x1,
x1.5, x2). Up to three dishes work at once; a fourth replaces the one with
least time left; the same effect does not stack, the longer one counts. Not
eating costs nothing.

| Effect | Does | Touches |
| --- | --- | --- |
| Stride | sprint and swim 15% faster | walking (built) |
| Steady hands | line holds 20% more tension, the fish tires faster | fishing (built) |
| Keen | dig and mine 15% faster | digging (built) |
| Glow | a 4 m light on the player | lamps (built) |
| Warmth | cold does not slow you | waits for a cold mechanic; does nothing until then |

Why not hunger: the game is a calm explorer with no health and no death;
world time stops when the game closes; a long flight to Sequoia or a dive
should not be nagged by a stomach. A drain punishes doing something else.
The alternative is drawn on the page (a hunger bar that drains and slows,
then hurts) and screenshotted beside the buffs (`cook-hud-buffs.png`,
`cook-hud-hunger.png`).

### 2. Fifteen stations, five tiers (Q2, asked with screenshots)

| Tier | Station | Built from | Burns | Makes |
| --- | --- | --- | --- | --- |
| 0 | By hand | nothing | - | fillets, sushi, salad, juice |
| 0 | Campfire and spit | 4 stone, 2 log | log | roasts, grills, skewers, flatbread |
| 0 | Drying rack | 3 log, 2 planks | - | jerky, dried pasta, tea, kelp sheets |
| 1 | Quern | 6 stone, 1 log | - | flour, semolina |
| 1 | Hearth and pot | 10 stone, 1 iron ingot | log | soups, stews, stock, sauce, boils, sugar, salt, tea |
| 1 | Brine and ferment barrel | 6 planks, 1 iron ingot | - | cures, salt fish, pickles, soy sauce, ale, leaven |
| 1 | Churn and cheese press | 6 planks, 1 iron ingot | - | butter, cheese |
| 2 | Brick bake oven | 20 brick, 6 clay | log | bread, pastry, pies, cake, lasagne, hardtack |
| 2 | Smoker | 10 planks, 2 iron ingot | log | smoked meat and fish, sausages |
| 2 | Pasta and noodle table | 8 planks, 2 log | - | dough, pasta, noodles, ravioli, batter |
| 2 | Pizza oven | 16 brick, 10 clay, 4 stone | log | pizza, flatbread |
| 2 | Oil press | 8 planks, 2 iron ingot | - | olive and soy oil |
| 2 | Icebox | 8 planks, 2 glass | ice | keeps food four times as long |
| 3 | Iron stove and wok | 8 iron, 2 copper ingot | coal | stir fry, fried rice, steak, omelette, spaghetti |
| 3 | Fryer | 6 iron, 2 copper ingot | coal | calamari, fish and chips, tempura, chips |
| 4 | Ship's galley | a ship module (M4, M23) | ship power | any hearth, oven, stove or fryer recipe; cans; freezer |

Each is a placed block on the durable edit path, as M3's stations are; a
heated one is a light at night. Fuel is an ingredient, one per cook (M3's
recommendation), so nothing burns while the player is away. The alternative,
five combined stations, is the question; separate stations are recommended,
as M3 keeps its six.

### 3. Heat is the only cooking skill (Q3, asked with screenshots)

A heated recipe names low, medium or high. Choosing it is one click in the
panel; the right one is worth a star. There is no timing minigame and no
random roll; a dish never fails outright (`cook-panel.png`).

### 4. Quality is worked out, never rolled

Points: the freshness band of the oldest ingredient (fresh 2, good 1, stale
0), the right heat (1; recipes with no heat give it free), and practice (1
once a recipe has been cooked five times). Four points is three stars, two or
three is two, less is one. Deterministic, as M3's tools are, so a server has
one rule to check. Stars scale the effect's minutes and the price.

### 5. Freshness counts played time; spoiled food becomes compost

A food stack stores the world time it was made. Age is world time now minus
that, so a closed game ages nothing. Bands: fresh to a third of its keep,
good to its keep, stale to twice its keep (it still cooks, at a lower star),
then spoiled, when the stack becomes compost (M8's fertiliser) one for one,
so nothing a player made simply vanishes. Two stacks of the same food and
stars merge and keep the older time.

| Keeps | Examples |
| --- | --- |
| 1 day | raw fish, fillets, seafood, raw meat, dough, batter, fried dishes |
| 2 to 3 days | milk, cooked meat, most dishes, bread, stock |
| 4 to 20 days | produce, butter, eggs, cheese, onions, ale |
| 30 to 60 days | smoked, cured, bacon, pickles, brine, salt fish |
| for good | grain, flour, sugar, salt, oil, soy sauce, dried pasta, tea, jerky, kelp sheets, hardtack, cans |

An icebox multiplies keep by four; on a frozen world a box outdoors in winter
does the same. The galley's freezer stops ageing.

### 6. Food is a new item kind; fish stay fish

`Item::Food(u16)` indexes an append-only food roster (as `Fish(u16)` indexes
the fauna roster), and the stack gains `made_at` (world time) and `stars`.
`Item::Fish` is unchanged and is an ingredient as it is, so a save with fish
in it opens exactly as before; no save migrates. Known recipes are a list of
recipe ids in the save. A placed station is a stored record; a cooking job is
a record on its station that advances only while the world is played.

### 7. Recipes are data

Recipes live beside M3's in `assets/config/recipes.ron` with a station, a
time in seconds, a heat, ingredients, an output and a count; food data
(keep, effect, minutes) in `food.ron`. Retuning a recipe never touches food
already made; a food whose recipe is removed still keeps and still works.

### 8. Learning recipes

Known at the start: the campfire's basics, bread, flour, porridge, salad,
juice and fillets. The rest come from townsfolk (the innkeeper's stews, pies
and sauces; the fisher's seafood and salt; the farmer's semolina, butter and
cheese; a **baker** and a **noodle cook** proposed to M5 as town roles), from
recipe books sold at stalls (M12) and found, and by discovery: putting a
recipe's ingredients into a station that makes it teaches it.

### 9. Selling

Food sells to innkeepers and at stalls, priced by stars and freshness; towns
pay more for what their land does not grow. Stations grow nothing, so
**provisions** (cans, hardtack, jerky, cheese) are proposed to M12 as a ninth
good on the exchange.

### 10. Twelve crops for M8, seeds only

| Crop | Climate outdoors | Seasons | Days | Water | Yield | Into |
| --- | --- | --- | --- | --- | --- | --- |
| Tomato | hot, temperate | summer, spring | 6 | normal | 4 | sauce, pizza, soup, salad |
| Durum wheat | temperate, dry | summer, spring | 7 | light (4 dry days) | 3 | semolina |
| Soy | temperate, hot | summer, autumn, spring | 6 | normal | 3 | soy sauce, oil |
| Bok choy | temperate, cold | autumn, spring | 3 | thirsty | 2 | stir fry, salad |
| Onion | temperate, cold | summer, autumn, spring | 6 | normal | 3 | savoury dishes |
| Garlic | temperate, cold | autumn, winter (hardy), spring | 8 | normal | 3 | sauces, seafood |
| Sugar cane | hot, wet | all year | 8 | thirsty | 3 | sugar |
| Sugar beet | temperate, cold | summer, autumn, spring | 7 | normal | 2 | sugar |
| Herbs | temperate, hot | summer, autumn, spring | 4 | normal | 3 | most dishes |
| Potato | temperate, cold | summer, autumn, spring | 5 | normal | 4 | chips, stew, mash |
| Olive (tree) | hot, dry | all year | 12, then every 6 | light | 4 | oil, brined olives |
| Tea (bush) | hot, wet | summer, autumn, spring | 10, then every 3 | thirsty | 3 | tea |

They are appended to M8's crop roster (append-only, a planted crop is saved
by its index) and follow M8's rules unchanged. Olive and tea are
**perennials**, a new rule: they stay after harvest and fruit again; frost
kills a young olive, not a grown one. Seeds come from farmers and traders;
nothing appears in the generated ground, so no generator version changes.
The crops are Tenebris's roster; on Sequoia they grow only in a greenhouse,
and Sequoia's own food waits for its own roster (CLAUDE.md: no accidental
species fallback across planets).

### 11. Meat, milk and eggs: products only

| Animal (M7) | Gives | Cut |
| --- | --- | --- |
| Meadow hen | eggs, meat | poultry |
| Plodder | milk, meat | beef |
| Woolback | meat | mutton |
| Ridge goat, sand strider | milk | none (M7 gives no meat) |
| Tusk boar | meat | pork (a domestic pig, the rootling, is the husbandry draft's) |
| Meadow and snow hare, shagback, mire lurker | meat | game |
| Gull, reed dabbler (M25) | eggs | none |

Each cut has four states: raw (keeps a day), cooked (two), smoked and cured
(a month). Venison is a gap: Tenebris has no deer in M7; the shagback's
haunch stands in until the animals or husbandry pass adds one. Honey from
the husbandry draft's bees would sweeten as sugar does, once that lands.

### 12. Fish and seafood, and three proposed catchables

Any caught fish cuts into two fillets by hand. Sushi takes only fish safe
raw: silverfin, deepback and redrun; M25's bloatfin is never served raw.
From M25: glow squid (jigged from a boat at night, when it rises to the
shelf) is calamari; octopus, lobster, crab, crayfish and pearl clam go to the
grill, the pot and the stove; kelp, cut and dried on the rack, wraps sushi.
Missing for the dishes asked for, proposed to `ocean-environments` (appended
to its roster):
- **sand prawn**: coastal sand 1 to 15 m, a hand net at low tide or a trawl;
  M25's cleaner shrimp is a finger-long reef cleaner, three to one;
- **sand clam**: temperate beaches, dug with the shovel at low tide as bait
  is dug today (`fishing-and-equipment`); M25's pearl clam is a warm-reef
  giant;
- **rock mussel**: coastal rock between the tides, by hand; M25's mussels
  live only at vents and in rivers.

### 13. The kitchens on the page

Each kitchen is built from boxes and hex prisms on the game's cell, textured
with the game's terrain atlas and settlement textures at their `repeat_m`,
and lit by one lamp array so every fire, oven mouth, lantern, candle and
window lights what is near it wherever the camera is; the moon stays dim.
Ovens are stepped domes of hex prisms, never a smooth sphere. Food on a table
is a pixel card turned toward the viewer. Interiors are cut away: the roof is
off and the walls between the viewer and the room drop to sill height. Fire
flickers and smoke rises because the world moves them; the camera stays where
the viewer left it and nothing turns on its own. The galley keeps a ship's
day: white cabin strips by day, dim red by night.

## Risks / Trade-offs

- **Fifteen stations is a lot to build.** Each is small (a model, a panel
  entry, a few recipes); the galley folds them back together late. Q2 asks.
- **Buffs only for built systems** leave warmth inert until cold exists. It
  is listed so the cold-world dishes are not lost, and does nothing until
  then.
- **A stack's made-at time** makes two stacks of the same food different
  stacks in the slot rules only by stars; merging takes the older time, so a
  fresh stack added to an old one ages it. That is the conservative choice
  and matches how a pile of food really goes off.
- **Seeds bought, not found**, means a new player meets new crops through
  towns. Wild herbs or olives in the world would be a generator change and a
  new version; not proposed.

## Migration Plan

None: there is no food in any save. A world saved before food exists opens
with no food, its fish as fish. Every roster this adds to is append-only.

## Open Questions

Asked with screenshots (`docs/screenshots/mockup-cooking/`):
- Q1. Buffs or a hunger bar (`cook-hud-buffs.png`, `cook-hud-hunger.png`).
- Q2. Many stations or a few (`kitchen-day.png`, `bakery-day.png`,
  `yard-night.png`).
- Q3. Heat as a choice at the station (`cook-panel.png`).

Recommendations taken (ask only with screenshots): decisions 4 to 13 above.
