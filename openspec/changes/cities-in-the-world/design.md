# Design: cities in the world

## Context

See `proposal.md` for why. What the design has to work with, on `main` and in
the changes this one follows (2026-09-27):

- **The settlements are designed and prototyped.**
  - `tenebris-towns` covers the cut (walls on edges, floors in cells, a
    three-layer storey), the three stairs, thin-solid collision and the
    walker's three rules. It also covers the kits, seven settlements, lights,
    no coplanar faces, furniture clearance, doors as world state, and no
    building on a pentagon.
  - Its delta to `world/settlements` holds those requirements.
  - Its tasks 1 to 5 are the engine work, and none is started.
  - `docs/mockups/towns.html` is the prototype.
- **The engine has none of it.**
  - `PlanetContact::stand` answers floors and ceilings per hex column.
  - The walker climbs up to 1.05 m in one tick.
  - There are no pieces, no kits and no settlement data.
- **Lights, after `lamps-and-lanterns`:**
  - the lantern family, the brazier and the candle, as materials;
  - dusk-lit emitters that re-bake the field at dusk and dawn;
  - moving things lit by the field;
  - a measured bake time for 300 lanterns.
- **Sites, after `city-sites`:** each world holds its resolved site list (id,
  kind, anchor, name) in its save. The footprints avoid pentagons and their
  neighbours.
- **Pop-in is the owner's first priority** (CLAUDE.md). `detail-fade`
  dither-fades terrain and trees. It still pops in a third to a half of
  landings in flight, and that is being fixed on its own.
- **The column generator** is a pure function of the direction and the
  config, cached per column. The edit log replays on top of it.

## Goals / Non-Goals

**Goals:**
- Every site has its town, and the town is the mockup's town: the same stairs,
  collision, kits and light.
- Towns sit in the landscape: on terraces cut into the real ground, with
  walkable edges.
- Towns read at every distance: pieces up close, a reduced form beyond, and
  lights from orbit at night.
- A played world's towns never change under it, and a town never overwrites
  the player's work.

**Non-Goals:**
- **Townsfolk.** The mockup's figures walk fixed paths. People who live in a
  town are their own change (what they do, and whether they trade or talk).
- **Roads, trade and factions** between towns.
- **A procedural town generator.** Each kind has an authored template with
  seeded variation. A generator that grows streets and plots could replace
  the templates later, behind the same interface.
- **Destroying town pieces.** A town's cells (the masonry wall, the terraces)
  can be mined like any cell. Its thin pieces (walls, floors, stairs, roofs)
  cannot be removed until `player-building` brings the tools that remove
  pieces. Doors open and shut.
- **Boats that sail.** The harbour's boats are fixed obstacles and the cog is
  boarded like a building, as in `tenebris-towns`.

## Decisions

**1. `tenebris-towns` is built first, as written, and this change only
places it.**
- Its tasks 1 to 5 are this change's group 1, done in its own change folder so
  its history stays whole. This change's design does not reopen the cut, the
  stairs or the collision.
- Its questions are answered by the owner before group 1 starts. Two are
  answered by the request:
  - towns are both generated and hand-placed, through `city-sites`'
    overrides;
  - the player builds with the same pieces (`player-building`).

**2. A layout is charted onto the sphere through the neighbour tables.**
- A template is authored on an axial grid, as the mockup's is.
- At a site, the anchor cell is axial (0, 0). The chart maps axial (q, r) to
  a cell by walking the neighbour table from the anchor: q steps along one
  edge direction, then r along the next, with the direction rotated by the
  site's variation.
- There is no pentagon within the footprint plus two cells, so every cell in
  the chart has six neighbours in consistent order. The walk is well defined,
  and two layout neighbours are sphere neighbours. This is tested on every
  site of the shipped seed.
- Every piece is then cut from its cell's real corners (`tenebris-towns`
  section 6), so the ±9% geodesic spread is absorbed by the stair's tread, not
  by a gap.
- *Alternative:* project the flat layout onto the tangent plane and take the
  nearest cell per point. Rejected: near the footprint's edge two layout
  cells can land on one sphere cell, or skip one.

**3. The ground is generated, not edited.**
- The column generator asks the world's settlement ground, a pure function of
  the stored settlement records, whether a column lies in a footprint or
  margin:
  - in a footprint, the surface is the layout's terrace layer;
  - in the margin (a band of cells set per kind), the surface eases from
    the terrace to the natural height, one layer a cell at most;
  - trees and clutter read a "cleared" bit in the same answer.
- Water rows in a harbour keep the real sea. The layout places its quay where
  the terrace meets the sea.
- *Alternative:* stamp the town as a batch of edits in the durable log.
  Rejected: that is about ten thousand cell edits per town, written the first
  time a player comes near, for something the stored record already
  determines.

**4. Variation is a seeded choice among layouts that pass the checks.**
- A site's seed picks the rotation (one of six), the mirror, each building's
  kit from its biome's list, and the plots left empty (up to a fifth).
- The layout checks from `tenebris-towns` (headroom, roof overlap, landings,
  furniture, coplanar faces) run on the result.
- A failure tries the next variation from the same seed, in a fixed order. The
  identity variation is last, and is proved to pass at build time.

**5. Towns are pieces in packed arrays, and a town in range is an active chunk.**
- Pieces are not entities (CLAUDE.md). A settlement in range becomes one
  entity, holding:
  - its piece arrays;
  - its thin-solid index for `stand`;
  - its meshes, built per settlement on the pool and published as a complete
    generation;
  - its emitter list.
- Out of range it is only its far form.

**6. Three forms, faded.**
- Pieces near, a reduced form (walls as boxes, roofs as their planes) beyond
  that, and at night a point list of lit windows and lanterns at any
  distance.
- Each change of form is a dither-fade on the same distance band terrain
  detail uses. It is tested by the same frame-by-frame check the
  `detail-fade` work uses, so it cannot pop where the terrain does not.
- The night point list is drawn by the planet pass. Orbit sees it without
  the settlement being loaded.

**7. Light.** The owner (survey C3): "I also hope to see the lights on
cities contribute to LOD hexes on the night side of the world too".
- Street and wall lanterns are `lamps-and-lanterns`' dusk-lit materials,
  placed by the template at the spacing its kind sets (the mockup's rule:
  densest in the walled town).
- A lit window is a new dusk-lit `WindowCandle`, placed in the room behind
  about half the windows, chosen by the site's seed. It lights the room, and
  the room's light shows through the pane.
- Hearths are always lit.
- **Far off, the lights light the ground, not only themselves.** Each
  settlement's lit windows and lanterns add a warm term to the coarse LOD
  hexes within its footprint and margin. The term is baked per settlement
  into a small per-hex glow value, from the same emitter list, so the night
  side from orbit shows lit patches of ground under each town as well as its
  points of light.
- The field bakes all of them as any other emitter. The bake-time measurement
  from `lamps-and-lanterns` task 5.5 decides whether the tier needs splitting
  before this lands.

**8. A town is a stored record, and its buildings are definitions, not
pieces** (the owner's save model, `world-persistence` decisions 1 and 2).
- When a world is made, every site's settlement is generated: the chart, the
  variation, and each building. The result is written to the record store
  before anything is shown.
- A building is stored as its own definition: plot cells, walls and their
  openings, stairs, storeys, kit, roof and state. It is not a reference to a
  template. A revised template changes new worlds only, and a town already
  made keeps what it was built as.
- Pieces are derived from the definitions by the cut rules each time a town
  is built into the world, so a fix to a stair or a sill reaches every town.
- A building's state is standing, abandoned or ruined. Only standing is
  generated. An abandoned building keeps its window candles and hearth dark.
  This is the one piece of the owner's "towns can grow, buildings can be
  abandoned" built here, and it is exercised by `world-persistence`'s
  test-only process.
- A site is left unsettled if the journal holds a player-authored entry
  within its footprint or margin from before its settlement record was made.
- *Alternative:* keep every template version forever and regenerate towns
  from the version recorded in the save. Rejected: a town that can grow or
  lose a house cannot be a pure function of a template, and the owner has said
  towns will change over time.

## Risks / Trade-offs

- [This is the largest change in the plan] → It is built in the order of the
  tasks, and each group ends in a capture the owner can see. The walker and
  contact (`tenebris-towns` 1 and 2) are tested on a synthetic stair before
  any town exists.
- [Hundreds of emitters in a walled town] → `lamps-and-lanterns` measures a
  300-lantern bake first. If it is over its 12 ms budget, the tier bakes
  settlements separately, and that is argued in this design before group 5
  starts.
- [Frame cost of towns: draw calls, thin-solid queries, light] → It cannot be
  measured in a cloud session. The owner runs `tools/perf_suite.py`, and a
  `town` scenario is added to the suite in this change.
- [Records grow as towns do] → A building definition is a few hundred bytes,
  which puts about 50 towns of 20 buildings near 500 KB. Growth adds
  buildings to plots, so a town's record is bounded by its plots.
- [The far form pops even though the pieces fade] → The fade covers the far
  form's own entrance too, and the night points fade in at dusk rather than
  switching on.

## Migration Plan

- An old save gets its settlement records the first time it is opened with
  towns, written through the durable path before anything is shown. Its sites
  are checked for earlier player edits at that moment.
- Rollback is the previous build. The towns vanish and the terraces return to
  natural ground. Door edits are ignored. No player edit is lost.

## Open Questions

For the owner, before group 1:
- `tenebris-towns`' questions 1, 2 and 5 to 9: walking speed indoors, what
  the Tenebris people look like, the stair feel, walkable roofs, which kits,
  settlements per biome, and boats.
- Should an unsettled site be built later, once the player's edits there are
  gone?
