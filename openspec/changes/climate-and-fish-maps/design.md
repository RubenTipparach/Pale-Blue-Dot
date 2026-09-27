# Design: climate and fish maps

## Context

See `proposal.md` for why. What the design has to work with, observed on
`main` (2026-09-27):

- **The spawner's rule.** A school may spawn where three things hold:
  - `fauna::water_class(terrain, direction, depth, limits)` gives a class the
    species lists (River, Shallows up to 6 m, Shelf up to 40 m, Deep);
  - `Species::fits(depth)` holds;
  - `Species::lives_in(class, temperature, limits)` holds for the live
    temperature, `air.now.sample(direction).temperature`, above the -1.8 °C
    freezing point.

  The spawner in `pbd-app/src/fish.rs` puts these together near the player.
  `fish::water_here` answers the class and temperature anywhere, for captures
  and logs.
- **The species** (`assets/config/fauna.ron`, append-only):

  | species | water | temperature |
  | --- | --- | --- |
  | minnow | river, shallows | 10 to 30 °C |
  | silverfin | shallows, shelf | -1.8 to 17 °C |
  | banded perch | river | -1.8 to 24 °C |
  | ray | shallows, shelf | 16 to 32 °C |
  | eel | river, shallows | 6 to 28 °C |
  | reef fish | shallows | 23 to 32 °C |
  | deepback | shelf, deep | -1.5 to 9 °C |
  | serpent | deep | -1.8 to 32 °C |
- **Range maps already exist outside the game.** `examples/fish_ranges.rs`
  runs the atmosphere forward and writes the altitude, the river carve and
  each year's mean, coldest and warmest daily-mean water temperature at
  1,440 × 720. `tools/fish_ranges.py` applies the rules and draws each
  species' range: all year in solid red, part of the year in pale orange.
  The output is in `docs/wiki/fish-ranges/`.
- **A year of weather is slow.**
  - The atmosphere runs on a level-5 grid of 10,242 cells, 181 m apart.
  - A year is 100 days of 2,880 s.
  - The shipped 200-day run took 2,660 s of wall time, so a year is about
    22 minutes.
- **The shipped planet cools.** In that same run, the mean sea surface fell
  from 7.7 °C on day 10 to -23.5 °C by day 200. A run with `solar_wm2` 1360
  (the shipped value is 1000) held it at about -14 °C by day 140. The climate
  normals will show this as a mostly frozen planet. That is a finding for the
  owner (see Open Questions), not something this change fixes.
- **Every world has the same seed.** The menu makes new worlds with
  `planet::TERRAIN.seed` and refuses a save from another seed. So a baked
  asset serves every world, per generator version.

## Goals / Non-Goals

**Goals:**
- A player can see where a fish can be caught now, and where it lives across
  the year.
- A player can see what a place is like across the year before they go there.
- The map, the spawner and the wiki's range maps can never disagree.

**Non-Goals:**
- **Climate simulated in the game.** Twenty-two minutes of simulation is not a
  loading screen.
- **Fixing the planet's cooling.** That is `climate-balance`, which lands
  first. The owner: "the sun should maintain average temperature of the
  planet to 15 c". The climate asset is baked from the balanced atmosphere.
- **Fish population, depletion or migration.** The layer shows where a species
  can spawn, not how many there are.
- **A weather forecast.** The live overlays show now, and the climate shows
  normal. What happens tomorrow is not either.
- **Biomes from the climate.** See `bigger-biomes`' non-goals.

## Decisions

**1. The spawner's rule becomes one public function, and the map calls it.**
- `fauna::can_spawn(species, class, depth, temperature, limits)` is the
  conjunction of `water_class`, `fits` and `lives_in`.
- The spawner calls it, and so do the map's fish layer and the field-guide
  thumbnail.
- A test samples 10,000 seeded water points and compares the layer's answer
  with the spawner's.
- *Alternative:* the map re-implements the rule in a shader. Rejected, because
  CLAUDE.md asks for one authoritative implementation of a gameplay rule.

**2. The fish layer's "now" is a raster built on the pool.**
- At 1,024 × 512, it is rebuilt when the map opens and every in-game hour
  while the map stays open, from:
  - a static class-and-depth raster, built once with the base map;
  - `Air.now`'s temperature sampled per pixel.
- Rivers are a few cells wide and would vanish at 29 m a pixel. So the class
  raster marks a pixel as river if any river channel crosses it, and the
  close-zoom tiles draw them exactly.
- The build time is measured before it is relied on (task 2.2).

**3. The climate normals are baked on the atmosphere's own grid.**
- The instrument runs one full year (a second is optional) from a new world
  with the shipped settings. For each of the 10,242 grid cells it records:
  - the mean temperature;
  - the coldest and warmest daily means, of the air and of the water
    (`ground_k`, as `fish_ranges` uses);
  - the yearly rain;
  - the four seasonal means.
- The file is about 0.5 MB. It is `assets/climate/tenebris-v<generator>.bin`,
  behind a header: magic, seed, generator version, a digest of the
  atmosphere settings, the cell count, and the planes.
- The map samples it with the grid's own `sample(field, direction)`
  interpolation.
- *Alternative:* an equirectangular raster. Rejected: it adds a projection and
  a resampling on top of a field whose real resolution is the 181 m grid.

**4. Provenance is checked, and staleness is shown rather than hidden.**
- At load the header is compared with the running world:
  - a different seed or generator version: the climate layer is unavailable,
    and the legend says so;
  - a different settings digest: the layer is shown with "made before the
    weather was last retuned".
- A fast test compares the committed asset's header with the shipped
  `atmosphere.ron`, and fails when it is stale. It names the command to
  re-bake.
- The full byte-for-byte check is the instrument's own `--check` mode, at 22
  minutes. It is run when the asset is remade, not in every test run.

**5. Climate classes are a small, Köppen-like scheme, with thresholds in
data.**
- Five classes: polar, cold, temperate, dry and tropical. Each is decided from
  the warmest and coldest seasonal means and the yearly rain.
- The thresholds are in `assets/config/climate.ron`, and set on the mockup.
- The sea-ice view marks water as frozen all year, frozen in the chosen
  season, or open.

**6. The wiki's range maps come from the same run.**
- `tools/fish_ranges.py` reads the rules from `fauna.ron` instead of its own
  copy, as its docstring already promises for when fishing is built.
- It draws from the same run that makes the climate asset, so the wiki and
  the game show the same ranges.

**7. The field guide gains a thumbnail and "show on map".**
- The thumbnail is rasterised from the normals and `can_spawn` at 128 × 64
  when the guide opens, using the two-range convention.
- The action opens the map on the fish layer with that species chosen.

**8. Habitat is derived, and a zone is stored only when it must be
remembered.** This is `world-persistence` decision 1, which answers the
owner's "certain zones of species" question.
- Where a species can live is a law. It is read from the live climate and the
  species rules, both tuned often, so it is derived and never saved.
- A zone becomes a stored record only when it has a name or a state: a named
  fishing ground, a stock that can be fished down, a legendary creature's
  lair.
- This change creates no zone records. The record store can hold them when a
  change designs one.

## Risks / Trade-offs

- [A 22-minute bake whenever the weather is retuned] → Staleness is shown, not
  fatal in the game. The fast test tells whoever retuned that the asset needs
  a re-bake, with the command to run.
- [The atmosphere is not bit-for-bit deterministic across machines, so
  `--check` fails on another CPU] → The weather spec says it is a
  deterministic field. If `--check` finds otherwise, that is a bug in the
  atmosphere reported on its own, and the check compares within a tolerance
  until it is fixed.
- [A frozen planet makes most fish layers nearly empty] → Answered by the
  owner: `climate-balance` holds the planet's average at 15 °C, and lands
  before the map mockup and before the asset is baked.
- [The fish raster is slow to build per pixel] → It is measured first. If
  needed, the temperature is sampled at 512 × 256 and the class at full
  resolution, and the spawner-agreement test runs at the temperature raster's
  pixel centres.

## Migration Plan

- There is no save change. The asset is new, and the layers are new.
- `fish_ranges.py` moving to `fauna.ron` changes no output while the rules are
  unchanged. The wiki is regenerated to prove it.
- Rollback is the previous build.

## Open Questions

In the owner survey (K4 and K5), and at the mockup:
- The climate classes' thresholds, and whether five is the right number.
- Should the fish layer show only species the player has caught (a
  discovered-species rule), or every species from the start?

The planet's cooling is answered: `climate-balance` holds the average at
15 °C.
