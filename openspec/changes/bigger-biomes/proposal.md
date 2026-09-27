# Proposal: bigger biomes, and fewer fields

## Why

**The owner (2026-09-27): "We should also modify biomes a bit to make them
about 4x bigger, so the majority isnt just grass anymore."**

Two measured facts sit behind that:
- **The biomes are small.** The moisture field that divides the temperate
  land has a 188 m feature size (`TerrainConfig::TENEBRIS.moisture_m`). A
  kilometre's walk crosses several biomes, and a test pins that it does. The
  owner chose 188 m in `terrain-feature-scale`, from three renders. This
  request revisits that choice with the owner's new words.
- **The land is mostly grass.** The moisture thresholds cut desert below 0.36
  and jungle or swamp above 0.64. A 4-octave noise field rarely reaches either,
  so Fields is about 59% of the land, while Desert, Jungle and Swamp are a few
  percent each (`distribution_report`, `docs/tenebris-comparison.md`).

A bigger field alone would not change the shares: the thresholds decide the
shares, and the feature size does not. So both move together.

This comes before `city-sites`, because a site's kind is chosen by its biome:
a desert town needs a desert large enough to hold one.

The saved world also has to know which generator made it. The main spec
already says it does ("The generator is versioned"), and it does not: a save
stores only the seed. Changing the biomes is the first change that would
silently alter an existing world, so it closes that gap first.

## What Changes

- **The moisture field's feature size grows.** "About 4x bigger" could mean
  the width (188 → about 750 m, sixteen times the area) or the area (188 →
  about 375 m). The `world-map` mockup shows both, from the real generator,
  and the owner chooses.
- **The desert and jungle thresholds are retuned from the measured
  distribution**, so no temperate biome holds most of the temperate land. The
  target shares are set with the owner on the mockup.
- **The generator version is saved with the world.** A save records the
  generator version that made it. A save made before this change reads as
  version 4. Each version names its own `TerrainConfig`, so an old world keeps
  the biomes it was made with, and a new world gets version 5.
- **The biome test moves from one kilometre to a longer walk**, to match the
  new size, and a new test pins the shares.
- No new biome, and no change to the land, the sea, the mountains, the rivers
  or the height of anything. Only the classification of dry land changes, and
  with it the top block, the trees and the scatter.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `world/terrain`: "The biome is a classification the surface reads". The walk
  that crosses more than one biome becomes longer, and a new clause says no
  temperate biome holds the majority of the temperate land.
- `planet/terrain`: "The generator is versioned". It gains the scenarios the
  saves need: an old save opens as version 4, a new world records the current
  version, and each version names the config that generates it.

## Impact

- **`pbd-core`:**
  - `planet_gen.rs`: `TerrainConfig::TENEBRIS` becomes version 5's config. The
    version 4 values are kept as a named constant, and
    `TerrainConfig::for_version(v)` picks between them.
  - `terrain.rs`: `GENERATOR_VERSION` becomes 5.
  - The biome tests change, and a new shares test is added.
- **`pbd-app`:**
  - no save format change of its own: the generator version is recorded in
    the identity `world-persistence` adds, and that change lands first;
  - `planet_terrain.rs`: this is the one app call site that picks the config,
    so it asks the world's identity for its version.
- **Captures:** where the spawn stands may change biome. The `surface`, `seam`
  and `coast` captures are re-framed if their views change.
- **Performance:** more jungle means more trees on screen. Jungle scatters 115
  trees in 256 against the fields' 13. This can move frame time, and it cannot
  be measured in a cloud session. The owner runs `tools/perf_suite.py` on the
  release build before this is merged.
- **Saves:** additive. An old build opening a new save ignores the unknown
  field and would generate version 4's biomes; that is the usual one-way limit
  of a newer save.
