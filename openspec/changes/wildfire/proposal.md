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

A forest fire is a change the world makes to itself, over minutes, across
hundreds of cells, that has to be in the save when it is over and agree on
every machine in multiplayer. That is a world process with its own records,
like the beavers' dams (`pbd_core::beaver`), not a survival rule about the
player's body, so it is its own change. `survival-mode` turns it on.

## What Changes

- **How dry it is.** The atmosphere gains one more field per cell, the
  ground's wetness, raised by rain and dried by sun, warmth and dry air.
  From it, the air's humidity and temperature, the flora's straw and the
  biome comes a fire danger, 0 to 1, at every place.
- **Fires start in the dry season.** In a Survival world, where the danger
  is high enough, a lit torch used on grass, a shrub or a tree starts a
  forest fire, and so does a lightning strike from a dry storm. Where it is
  not dry enough, the flame dies in seconds, as `lightning-fire`'s does.
- **They spread cell to cell through what grows**: grass, shrubs, trees,
  fastest downwind and uphill, through each cell's five or six neighbours,
  by a fixed one-second rule the server runs, the same on every machine.
- **Rain, water and bare ground stop them.** Rain slows and then puts out a
  fire; water, sand, rock, snow, dug or paved ground and roads are breaks;
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
- `world/fire`: fire danger, forest fires (starting, spreading, stopping,
  what they leave and its regrowth), and torches going out and being relit.

### Modified Capabilities
<!-- `world/weather` gains the ground's wetness; written here as part of
`world/fire` until it is built, then moved beside the weather's own
requirements. `lightning-fire` keeps its short fires and gains a dry strike
that starts a blaze in Survival (design 6). -->

## Impact

- **`pbd-core`**: `pbd_core::fire`, the danger, the fuel table and the
  spread rule as pure functions of the blaze record, the tick, the weather
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
  by its own flames, with the rain putting it out
  (`docs/mockups/survival.html#fire`).
