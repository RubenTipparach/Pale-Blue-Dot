## Why

The owner, 2026-11-26, starting phase 4 with survival mode:
- "Survival: you can start foresr fires during dry seasons."
- "Torches can light up stuff or can be out out by rain."

Today:
- fire exists only as the campfire, the brazier, candles and torches, all
  placed blocks that burn for ever and light the cell light field
  (`glowing-mushrooms` design, `lamps-and-lanterns`);
- `lightning-fire` (designed, not built) plans a short fire where a strike
  lands, which the rain puts out in seconds, with nothing saved: "The fire
  is weather, like the rain";
- the weather is a CPU model in the core, saved with the world and run by
  the server (`pbd_core::atmosphere`): it knows the air's temperature,
  humidity, wind and rain at every place, but not how dry the ground is;
- the flora's season (`flora::season_at`) already dries the grass to straw
  in its season (`SEASON_STRAW`).

The owner, 2026-11-26, on the first draft and its mockup: "Let's not spread
fire too much for now , consider lots of factors like dryness of plants",
with Our World in Data's *Wildfires* page (https://ourworldindata.org/wildfires)
as "some ideas": grassland burns far more than forest, the fire season is
each hemisphere's summer, even the most fire-prone land burns only a few
percent a year, and farmed and grazed land breaks the fuel up (design 12).

A forest fire is a change the world makes to itself, over minutes, across
hundreds of cells, that has to be in the save when it is over and agree on
every machine in multiplayer. That is a world process with its own records,
like the beavers' dams (`pbd_core::beaver`), not a survival rule about the
player's body, so it is its own change. `survival-mode` turns it on.

## What Changes

- **How dry it is, plant by plant.** The atmosphere gains one more field
  per cell, the ground's wetness, raised by rain and dried by sun, warmth
  and dry air. From it, the air's humidity and warmth, and each cell's own
  plant comes a plant dryness, 0 to 1, derived and never saved: the plant's
  kind (green or cured grass, a shrub, a deciduous tree, a conifer, never a
  cactus), its season in its own hemisphere (cured in late summer), shade
  under trees, water nearby (a river's banks stay green), hollows and rises,
  the soil's own patches, grazing, and snow (design 2).
- **Green plants do not burn.** A cell below 0.5 never catches; above it
  the hazard grows as the square of the margin. In spring nothing catches.
- **Fires start in the dry season.** In a Survival world, a lit torch used
  on a dry plant lights it, and so does a lightning strike from a dry storm
  (at most one in 10 world days within 2 km). On a plant too green the
  flame dies in seconds, as `lightning-fire`'s does.
- **They stay small** (design 12): they spread cell to cell through dry
  plants only, by a fixed one-second rule the server runs, the same on every
  machine; a late-summer grass front runs about 0.2 m/s, under 1 m/s in
  any wind; trees burn ten times slower and only a very dry conifer in a
  strong wind burns in its crown; no embers are thrown; a fire burns itself
  out where its front meets green or wetter ground, and is held to 300 cells
  (about 0.2 ha) and 40 burning at once, with at most 2 in a world.
- **Rain, water, bare and green ground stop them.** Rain slows and then puts out a
  fire; water, sand, rock, snow, dug, tilled or paved ground, paths, roads
  and grazed pasture are breaks;
  a bucket of water puts out a burning cell and its neighbours. Whatever a
  player built or planted is never burned: it is a break too (CLAUDE.md: a
  change the world makes never overwrites the player's).
- **What they leave**: ash-grey ground, standing charred trees that give
  charcoal when chopped, and regrowth over world days: grass in 4, shrubs in
  10, saplings in 20 and full trees in 60. All of it is saved.
- **Torches** (Survival): a torch in rain under the open sky goes out, held
  after 10 s and placed after 20 s, and is relit from any fire; a lit torch
  lights a torch, a brazier, a candle, a lantern, and dry growth. The placed
  torch stays the player's block; that it is out is a record of the world's.
- **Explorer is unchanged**: no forest fires, torches never go out, and
  lightning's short fires stay weather.

## Capabilities

### New Capabilities
- `world/fire`: plant dryness, forest fires (starting, spreading, stopping,
  what they leave and its regrowth), and torches going out and being relit.

### Modified Capabilities
<!-- `world/weather` gains the ground's wetness; written here as part of
`world/fire` until it is built, then moved beside the weather's own
requirements. `lightning-fire` keeps its short fires and gains a dry strike
that starts a blaze in Survival (design 6). -->

## Impact

- **`pbd-core`**: `pbd_core::fire`, the fire weather, the plant dryness,
  the fuel table and the spread rule as pure functions of the blaze record, the tick, the weather
  sample and the yield set, with tests; `atmosphere` gains `ground_wet`.
- **Saves**: `blaze` records and their lines, authored `@wfire:blaze/<n>`
  with the igniting player or the lightning named; `lamp_out` records for
  torches the rain put out. Additive; no generation change, so no world
  reads its ground differently.
- **Multiplayer**: the server runs every blaze and every torch going out;
  clients draw what the lines say.
- **Rendering**: flames, smoke and embers as derived GPU particles, the
  fire's light in the cell light field, a burnt-cell table the ground and
  tree draws read (the felled-tree list holds 64 and cannot carry a forest).
- **Mockup first**: a forest fire spreading at night in the dry season, lit
  by its own flames, with the rain putting it out, the plants' dryness
  shown, and a season control (`docs/mockups/survival.html#fire`).
