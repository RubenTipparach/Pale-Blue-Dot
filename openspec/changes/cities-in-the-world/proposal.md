# Proposal: cities in the world (step 2d of the cities plan)

## Why

**The owner (2026-09-27): "step 2 implement cities ... After I have approved
1. lights, 2. a) mapping system, b) city maps, c) climate map, fish map then
finally D) implement cities".**

This is the last step of step 2, and the one the others lead to:
- `tenebris-towns` designed the settlements and proved them in a mockup: their
  stairs, collision, kits and lights.
- `lamps-and-lanterns` gives them lights the engine can bake.
- `city-sites` says where they stand.
- `world-map` and its layers let the owner see and approve all of that.

What is left is to build them into the planet: pieces the walker stands on and
the renderer draws, fitted to the real cells around each site, lit at night,
seen from a distance, and kept with the world.

`tenebris-towns` has its own tasks for the pieces (contact, the walker, pieces,
kits, drawing, doors). This change does not repeat them. It takes them as its
first group, and adds everything a town needs to stand in the world rather
than on the mockup's flat grid.

## What Changes

- **`tenebris-towns`' tasks 1 to 5 are built:**
  - thin solids and surfaces in `stand`;
  - the walker's three rules;
  - pieces cut to the cell;
  - kits as data;
  - the per-biome settlement templates and the harbour;
  - drawing, lights and doors.

  Its questions to the owner are answered first. This request answers two of
  them: towns are both generated and placed by hand, and the player builds
  with the same pieces.
- **A settlement is built at each site.** Its template is laid onto the real
  level-11 cells around the site's anchor. The layout's axial grid is mapped
  cell by cell through the neighbour tables. Sites exclude pentagons, so the
  chart is a true hex grid there, and every piece is cut from its cell's real
  corners.
- **The ground is shaped to the town.** Inside a footprint, the terrain's
  surface is the template's terraces. A margin eases back to the natural
  ground, one step a cell at most, so it can be walked. The trees and clutter
  inside are cleared, apart from the template's own plants. This is part of
  generation, not a stack of edits.
- **No two towns of a kind are the same.** Each site's seed rotates the
  template by a multiple of 60°, may mirror it, draws each house's kit from
  its biome's set, and leaves some plots empty.
- **Lit at night, near and far.**
  - Up close, street lanterns and wall lanterns are `lamps-and-lanterns`'
    dusk-lit materials, and about half the windows have a dusk-lit candle in
    the room behind them. The field bakes them like any other light.
  - Far off, and from orbit, a town is drawn as a small set of its lit
    windows and lanterns, so a night-side city is a cluster of lights.
- **No pop-in.** A town's detail dither-fades in and out as terrain detail
  does. This is the owner's first standing priority.
- **The player's work comes first.** A site whose footprint already holds
  edits made before its town existed is not built. The map shows it as
  unsettled.
- **A town is part of the world's identity.** The settlement template version
  is recorded in the save beside the generator and site versions. A town is
  never regenerated under a world that has played it.
- **Out of scope:** townsfolk (a separate change), trade, and roads between
  towns.

## Capabilities

### New Capabilities
- None. `world/settlements` is introduced by `tenebris-towns`' delta, and this
  change adds to it.

### Modified Capabilities
- `world/settlements`: added requirements on building a settlement at each
  site, fitting it to the ground, varying it, drawing it far off and at night,
  fading it in, and keeping it with the world.
- `planet/terrain`: "Generation is deterministic and order-independent". The
  terrain becomes a function of the world's site list and template version as
  well as the seed, so a town's terraces are generated the same way in any
  order.

## Impact

- **`pbd-core`:**
  - `settlement`, which is `tenebris-towns`' pieces plus the chart onto the
    sphere, the variation and the ground shaping;
  - `planet_contact`, which gains thin solids and surfaces;
  - the column generator, which reads the settlement ground inside footprints;
  - `terrain::Material`, which gains a dusk-lit window candle.
- **`pbd-app`:**
  - settlement streaming: a settlement becomes an active chunk entity when in
    range;
  - piece meshes with the pixel textures;
  - the far and night representation;
  - the fade;
  - doors on E, through the durable path.
- **Assets:**
  - `assets/settlements/<kind>.ron` (the templates);
  - `assets/config/kits.ron`;
  - the piece textures, as committed PNGs with manifests (CLAUDE.md's art
    rules).
- **Saves:** `WorldFile` gains the template version. Doors' states are
  edits.
- **Performance:** this is the heaviest step of the plan. There are more draw
  calls, a light bake with hundreds of emitters, and contact queries against
  thin solids. None of it can be measured in a cloud session. The owner runs
  `tools/perf_suite.py` on the release build, and a town scenario is added to
  the suite so later changes measure against it.
