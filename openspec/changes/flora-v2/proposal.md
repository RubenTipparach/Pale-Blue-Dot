# Proposal: flora v2 - pixel tuft grass, richer clutter, leaf cards, seasons

## Why

Phase 2 of the roadmap is Environment v2: "advanced foliage and flora (M13),
and an underwater overhaul (M25)" (CLAUDE.md, "The first game loop"). The owner,
2026-11-15: "continue with next phase while i review". This change is the flora
half; the underwater half and audio are other changes and touch none of these
files.

The mockup is M13, `docs/mockups/foliage.html` ("Foliage Detail Playground").
Its notes list seven decisions; the owner's one direct request on it is the
grass (2026-11-14): *"this grass is actually better than what we have in the
game ... add it to the foliage pack, add a toggle for me to try this alternate
grass in the different biomes"*. That is built first.

## What Changes

Ordered by value, each a slice that stands on its own:

1. **Pixel tuft grass, behind a player toggle (the owner's request).** The
   Stone and Timber Play demo's tuft - nine one-pixel blades on a 16 px card,
   dark root to light tip, on two crossed cards 0.8 m across and 0.6 m tall,
   sampled nearest and cut out - drawn in the grass slots of the clutter draw
   in place of the blades cut from the ground tile. Each biome has its own
   three-tone ramp (the mockup's ramps, mapped onto the game's eight biomes).
   A **Pixel grass** check on the settings' GRAPHICS page switches it, off by
   default, kept in the player's settings file (version 5); `--grass
   tufts|blades` sets it for a capture.
2. **Richer per-biome clutter.** Flower heads in four pixel colours from each
   biome's palette (they were a sand- or snow-coloured fan), mushrooms (a stem
   and a cap; red in the fields, brown elsewhere), and ferns (fronds cut from
   the jungle sheet's fern tile by a pixel mask), at per-biome weights as the
   mockup's biome table has them. They dither in and out at the edge of the
   clutter reach.
3. **Leaf cards on the hex-prism trees** (mockup Q1 C). The prism stays the
   shape you chop - every cell, material and save is untouched - and each crown
   layer is wrapped in leaf cards hanging from its rim, cut from the biome's own
   leaf tile by a pixel mask, so the silhouette is ragged leaves rather than a
   hexagon. They are drawn out to a reach of their own and dither out before
   it; past it the prism alone is the far tree, as it is today. They cast sun
   shadows through the same cut-out.
4. **Seasons on plants** (mockup Q5 A, one calendar per planet). The clock's
   year (`orbit-and-seasons`, already built) turns deciduous leaves gold and
   red in autumn and bare in winter, and dries the grass to straw, flipped by
   hemisphere and fading out toward the equator. A tint and a thinning of
   derived decoration: no cell changes.

Not in this change, and why:

- **Baked mid and far tree impostors** (mockup Q2). The recommendation (crossed
  cards mid, billboard far, baked from the near tree) is taken, but its build
  is a render-to-texture bake, an atlas binding and a new LOD switch, and a day
  is better spent getting the near cards right first. Past the card reach the
  game draws the prism crown it draws today, which is what a card-less bake of
  the same tree would show. Left as an open task.
- **Fungal forest** (mockup Q7). It is a new biome, so a new generator version
  for new worlds (CLAUDE.md "Saved games survive every change"); it waits for
  its own change.
- **Wild plants** (`farming-and-smelting` decision 14) keep their own mesh and
  behaviour; nothing here changes where they grow or how they are picked.

## Capabilities

### New Capabilities

- `planet/flora`: the derived plant decoration on the ground and on trees -
  tuft grass, per-biome clutter, leaf cards and seasonal colour.

### Modified Capabilities

- `platform/settings`: one more player setting, Pixel grass.

## Impact

- `assets/shaders/planet_surface.wgsl`: the grass slots, two new clutter
  pieces, leaf cards in the foliage branch, cut-outs in `dropped`, the flora
  colours and the season tint in `shade_terrain`.
- `assets/shaders/planet_visibility.wgsl`: the new vertex budgets, and the
  clutter eligibility knows mushrooms and ferns.
- `crates/pbd-app/src/planet.rs`: the uniform gains the flora lanes and the
  per-biome tables; the vertex arithmetic grows.
- `crates/pbd-core/src/flora.rs` (new): the ramps, the per-biome table and the
  season rule, with tests.
- `crates/pbd-app/src/settings.rs`, `desktop/menu.rs`, `desktop.rs`: the
  setting, its row and its flag.
- `assets/config/scatter.ron`, `config.rs`: tuft, mushroom, fern and leaf card
  knobs, validated.
- **Saves:** nothing a world stores or generates changes. Every piece here is
  derived decoration, a pure function of the cell and the clock, never saved,
  never collided with. The settings file moves to version 5; an older file
  reads Pixel grass off and keeps the rest.
- **Frame cost:** more vertices per tree (18 cards) and per clutter cell (a
  mushroom and four fronds). Not measured in this change (the perf suite is
  the owner's on the desktop); said so in the report.
