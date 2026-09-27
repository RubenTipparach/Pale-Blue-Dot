# Proposal: city sites (step 2b of the cities plan)

## Why

**The owner (2026-09-27): "step 2 is big, we'll first need to designate
cities on a map ... So map with cities first mocked up, then I will approve."**

`tenebris-towns` designed nine settlements: the walled town, the village, the
desert, tundra, jungle, swamp and harbour settlements, and two in the mountains,
a cliff village and a town inside a mountain. Each is built from its own
biome's materials. The owner, on whether the map places the mountain two
(survey T12, 2026-09-27): **"recommended"**, which is "add both as site kinds
now". None of them has anywhere to stand yet: the engine
has no settlements and no notion of where one could go.

Before a town is built into the ground (`cities-in-the-world`), the planet
needs a list of places. That list must be:
- **Deterministic**, so every machine and every future multiplayer peer puts
  the same town in the same place.
- **Readable on the map**, so the owner can see it and approve it.
- **Open to the owner's hand**, so a place the owner wants can be pinned and a
  bad one struck out.

## What Changes

- **Sites are generated from the seed and the generator version.** A pure
  function in `pbd-core` scores places by rules and keeps the best, spaced
  apart. The rules:
  - dry land;
  - flat enough for the settlement's footprint;
  - no pentagon under or beside it;
  - a harbour on a shelving coast.
- **A site has a kind, chosen by its biome.** Its kind is one of the nine
  settlements of `tenebris-towns`:
  - fields: walled town or village;
  - desert: desert town;
  - tundra: tundra camp;
  - jungle: jungle village;
  - swamp: swamp village;
  - the beach where it meets fields: harbour;
  - mountains: cliff village or cave town (T12).

  Open ocean gets none.
- **A site has a name**, drawn from name tables for its biome's people. It is
  stable for the life of the world and unique on the planet.
- **An authored list the owner controls.** `assets/config/sites.ron` holds the
  counts, the spacings, the flatness limits and the name tables. It also holds
  overrides: a site pinned at a place with a kind and a name, or a generated
  site struck out. The owner can place cities by hand, and the generator fills
  the rest.
- **A map layer.** Each site is a marker for its kind, with its name. Towns
  are labelled at world zoom and villages closer in. At close zoom the site's
  footprint is outlined.
- **The mockup first.** The `world-map` mockup runs these rules in the page,
  over the real rasters, before any Rust is written. The owner can drag, add
  and remove sites there, and copy the result out as the RON override list.
- **A world keeps its sites.** The resolved list is written to the save the
  first time the world is opened with sites, with the sites version that made
  it. A retune never moves, renames or removes a town in a world that has been
  played.

## Capabilities

### New Capabilities
- `world/sites`: where settlements stand, what kind each is, what it is
  called, how that follows from the seed and the owner's overrides, and how a
  saved world keeps its list.

### Modified Capabilities
- `player/map` (added by `world-map`, not yet in the main specs): an added
  requirement that the map shows every site.

## Impact

- **`pbd-core`:** a new `sites` module. It reads the generator through
  `planet_gen` and the level-7 cells through `topology::dual_sphere`, and
  touches nothing else.
- **`assets/config/sites.ron`:** new and validated at load, with units.
- **`pbd-app`:**
  - a sites resource, built once per world on the async pool;
  - a `player/map` marker layer;
  - the resolved list stored as `site` records in `world-persistence`'s
    record store, written once.
- **Docs:** the `world-map` mockup gains the site editor.
- **No terrain change.** A site is a place on the map; nothing is built on it
  until `cities-in-the-world`.
- **Performance:** a one-off pass over 163,842 level-7 cells, each sampled
  over its footprint, off the frame. It is timed by the instrument before it
  is built (task 1.1).
