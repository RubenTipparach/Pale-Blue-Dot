# Proposal

## Why

The owner, 2026-09-29, on version 5's biomes: "Too much dessert. Biome is the
right size, but that's a lot of deserts." The 750 m moisture field stays
(survey B4, answered by that sentence). Only the desert's share moves.
Survey B2 chose fields, desert, and jungle with swamp at about a third each
of the temperate land. On the map that puts a fifth of all the land under
desert: 21% on version 6's ground, as much as the fields.

## What Changes

- Version 6's desert shrinks to the share the owner picks (survey B6), from
  a third of the temperate land. The land it gives up is split evenly between
  fields and jungle. The candidates are measured and drawn in the design.
- The moisture field's size, and everything else about the biomes, stay as
  they are.
- Version 5 keeps its deserts (CLAUDE.md, "Saved games survive every
  change"). The change rides generator version 6, which is not released:
  `taller-mountains` builds it, and no world has been made on it.
- Version 6's settled climates are made again, since the climate reads the
  biome (its wetness and albedo).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `world/terrain`: "Grass is not the majority" gives each temperate biome a
  band of its own. Fields and jungle with swamp each hold more than a third,
  desert holds the owner's share, and none holds half.

## Impact

- `pbd_core::planet_gen::TENEBRIS_V6`: `desert_below` and `wet_above`.
- `grass_is_not_the_majority_on_five_seeds` and its scenario.
- `assets/climate/settled-g6-l{3,5}`, remade.
- Captures of version 6's biomes.
