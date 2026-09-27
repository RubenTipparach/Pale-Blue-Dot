# Design: city sites

## Context

See `proposal.md` for why. What the design has to work with, observed on
`main` and in the planned changes (2026-09-27):

- **The settlements are designed, not built.** `tenebris-towns` (its design
  section 10 and the `docs/mockups/towns.html` prototype) has seven kinds. Each
  takes its materials from one biome and no other:
  - walled town and village (fields);
  - desert town;
  - tundra camp;
  - jungle village;
  - swamp village;
  - fishing harbour (beach, with the fields behind).

  The walled town is laid out on 50 × 34 cells, about 142 × 83 m. The harbour
  is laid out from the sea up: twelve rows of water shelving from 4 m to
  0.35 m, three rows of beach, a quay, and terraces a layer apart.
- **No building on a pentagon or on a pentagon's neighbour.** `tenebris-towns`
  requires it (its design section 6). The pentagons are the icosahedron's
  twelve vertices at every level of `topology::dual_sphere`.
- **The planet.** The radius is 4,800 m. Level 7 has 163,842 cells about 45 m
  apart. Level 11, the terrain's cells, has cells 2.833 m apart. About half the
  sphere is land today. After `bigger-biomes` the biomes are regions of the
  size the owner picks.
- **The generator answers anywhere.** `surface_altitude`, `biome_at` and the
  river carve can be read for any direction without loading terrain. The
  water classes (Shallows up to 6 m, Shelf up to 40 m) are in
  `pbd_core::fauna`.
- **Saves.** `WorldFile` stores the seed. `bigger-biomes` adds the generator
  version. Every accepted world mutation enters the durable transaction path
  (CLAUDE.md).

## Goals / Non-Goals

**Goals:**
- A list of places for the seven settlements that every machine agrees on.
- Places the settlement will actually fit: flat enough, dry, off the
  pentagons, and a harbour on real water.
- The owner can put a city exactly where they want one.
- A played world never has a town moved under it.

**Non-Goals:**
- **Building anything.** That is `cities-in-the-world`.
- **Roads between sites, trade, or factions.** A later change can join the
  sites. The list keeps each site's id stable so one can.
- **Settlements on other bodies.** The airless bodies are lifeless
  (CLAUDE.md), and no other body has terrain that would hold one.
- **Mountain settlements** (a monastery, a mine), and **Ocean settlements** (a
  stilt town on the shelf). There is no designed settlement for either.
  `tenebris-towns` would design one first.
- **A spawn beside a village.** Whether a new player should start in sight of
  a settlement is a question for the owner (Open Questions), not a default.

## Decisions

**1. Candidates are level-7 cells, and each kind has a footprint in metres.**
- A candidate is a level-7 cell's centre: 163,842 of them, about 45 m apart.
  Its id is that cell's index. Topology is versioned with the world
  (CLAUDE.md), so the id is stable for the life of a save.
- Each kind declares in `sites.ron`:
  - a footprint radius, in metres;
  - a flatness limit, in metres of surface range;
  - a spacing, in metres;
  - a target count.

  Starting values, set on the mockup:

  | kind | radius | flatness | spacing | count |
  | --- | ---: | ---: | ---: | ---: |
  | walled town | 75 m | 8 m | 2,500 m | 6 |
  | village | 35 m | 4 m | 700 m | 20 |
  | desert town | 60 m | 6 m | 2,000 m | 4 |
  | tundra camp | 30 m | 4 m | 700 m | 4 |
  | jungle village | 35 m | 10 m | 700 m | 6 |
  | swamp village | 35 m | 3 m | 700 m | 3 |
  | harbour | 60 m | beach to 12 m | 1,500 m | 6 |

  The jungle's limit is loose because it is built on platforms. The swamp's
  is tight because its houses stand on stilts over the water.

**2. The rules screen coarsely, and each site is checked fully when it is
kept.**
- **The screen.** For each candidate and each kind whose biome is at its
  centre, sample seven points (the centre and a ring at the footprint's
  radius). Candidates whose range is over the limit, or that touch the sea
  (except a harbour), are dropped. A candidate within the footprint plus two
  cells of a pentagon is dropped.
- **The score.** The survivors get
  `flatness + river nearby (villages and towns) + seeded jitter`. The jitter
  is from a hash of the seed and the id. It stops every town from crowding the
  single flattest plain.
- **The greedy pass.** Candidates are sorted by score, then by id, and kept in
  that order when three things hold: the spacing is clear of every site
  already kept, the kind's count is not yet reached, and a full check of the
  footprint at level-11 spacing passes (the spec's "flat, dry ground").
- The sort is total and the hash is fixed, so the list is the same on any
  thread count.
- The cost is measured by the instrument (task 1.1) before the game runs it.
- *Alternative:* Poisson-disc sampling over the sphere. Rejected: it places
  first and checks after, so a count is hard to hit without retries, and the
  retries are order-sensitive.

**3. A harbour is found from the water.**
- A harbour candidate is a beach cell with fields within its footprint. Along
  the line from its centre to the nearest deeper water, the depth must shelve
  down from the shore to Shelf (6 m to 40 m) within 60 m. That is the
  mockup's twelve rows of water plus the approach.
- This uses the water classes `pbd_core::fauna` already defines, so a harbour
  stands where the fish maps show the inshore fish.

**4. Names come from tables per people, seeded by the id.**
- `sites.ron` holds syllable tables for five peoples:
  - the fields and harbour folk;
  - the desert people;
  - the tundra people;
  - the jungle people;
  - the swamp people.

  Each has onsets, nuclei, codas and kind-specific suffixes (a "-ford" for a
  village on a river, a "-haven" for a harbour).
- A name is drawn from a hash stream of the seed and the id. A collision takes
  the next draw from the same stream, resolved in list order.
- A pinned name is reserved first, so no generated site takes it.
- *Alternative:* hand-written names per site. They are what the override list
  is for. The generator is for the other forty.

**5. The overrides are part of the input, applied before the greedy pass.**
- Pinned sites are kept first and are not held to the rules, except for the
  pentagon and the sea. Those two are refused at load, with the line named,
  because nothing can be built there.
- Struck ids are removed from the candidates.
- The mockup's site editor writes the same RON, so the owner's hand-placed
  cities go straight into the file.

**6. A world stores its resolved list, as records.**
- Sites are `site` records in `world-persistence`'s store: its "facts are
  stored" rule, as the owner asked for towns and landmarks. When a world is
  made, or an older world is first opened, the list (id, kind, anchor, name)
  is generated for the whole planet. It is written through the durable path
  before any site is shown. That is about 50 sites of around 40 bytes.
- After that the world reads the list from its save. Keeping every old rule
  set alive to regenerate old lists is not needed.
- An old save that predates sites gets them on its next open, like a new world.
- *Alternative:* store only a sites version and keep every version's rules
  and overrides forever. Rejected: that is the same promise at a much higher
  cost, and a changed override file would break it.

**7. The map layer uses the `world-map` registry.**
- A marker layer of icons per kind. Labels are culled by size and zoom. The
  footprint outline is drawn as a circle through `geo`'s projection.
- The site list is built on the async pool and is ready before the map can be
  opened. Until it is, the legend shows the layer as "surveying".

## Risks / Trade-offs

- [The fine check is too slow] → The screen drops almost everything first.
  The instrument times it (task 1.1). If it is still slow, the fine check
  samples a ring every 10 m rather than every cell.
- [A seed with too little of one biome for its kind] → The count is a target,
  not a promise. The site list logs each kind's shortfall, and the "every kind
  occurs" test runs on the shipped seed only.
- [Sites generated before `bigger-biomes` land in the wrong biome once it
  changes] → The site rules read the world's own generator version, so they
  follow the biomes the world has. A saved list is never regenerated.
- [A pinned site on terrain that will not hold the settlement] → The
  pentagon and the sea are refused. Steep ground is allowed, because the
  owner asked for it there. `cities-in-the-world` levels the ground under a
  footprint, and the mockup shows the owner the range before they pin.

## Migration Plan

- Sites are new records. An old save gets its list on the next open.
- Rollback is the previous build: the records are ignored, and no terrain has
  changed.

## Open Questions

For the owner, at the mockup:
- The counts and spacings, and whether ~50 sites is the right population for a
  30 km planet.
- Whether a new world should start the player within walking distance of a
  village.
- Whether any site should be pinned by hand from the start (the owner's own
  capital, say).
