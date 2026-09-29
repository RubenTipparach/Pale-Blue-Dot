# Design: city sites

## Context

See `proposal.md` for why. What the design has to work with, observed on
`main` and in the planned changes (2026-09-27):

- **The settlements are designed, not built.** `tenebris-towns` (its design
  section 10 and the `docs/mockups/towns.html` prototype) has nine kinds. Each
  takes its materials from one biome and no other:
  - walled town and village (fields);
  - desert town;
  - tundra camp;
  - jungle village;
  - swamp village;
  - fishing harbour (beach, with the fields behind);
  - cliff village and cave town (mountains; the ninth mockup round, placed
    on the map at the owner's word, survey T12).

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
- **Ocean settlements** (a stilt town on the shelf). None is designed. The
  mountains' two are in (survey T12).
- **Moving the spawn.** The spawn stays where it is. A small town is placed
  near it instead (survey C2 and C3).

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
  | cliff village | 50 m | a rise of 9 to 25 m | 1,500 m | 4 |
  | cave town | 50 m | rock 16 m or more over the chamber | 2,500 m | 2 |

  The jungle's limit is loose because it is built on platforms. The swamp's
  is tight because its houses stand on stilts over the water. The mountain
  two are the opposite of flat, and decision 5 screens them by their rock.

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
- `sites.ron` holds syllable tables for six peoples:
  - the fields and harbour folk;
  - the desert people;
  - the tundra people;
  - the jungle people;
  - the swamp people;
  - the mountain people (survey T12).

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

**5. The mountain kinds are found by their rock, not their flatness** (the
owner, survey T12: add both now).
- **A cliff village** needs ground that climbs through its terraces. The
  mockup's village climbs 15 m in five terraces 3 m apart over about 70 m. Its
  screen samples the footprint along the downhill direction at the centre.
  The surface must rise between 9 and 25 m across the footprint, and never
  by more than 6 m between two samples 10 m apart (a sheer drop cannot be
  terraced). The downhill side must be dry: it is where its lanes face.
- **A cave town** needs a mass of rock over its chamber, and a face to enter
  it from. The mockup's chamber is up to 93 by 37 m, 10 m high, under rock 4 m
  thick or more. Its screen needs the surface at least 16 m above the
  entrance's level over the whole chamber. Within 30 m of the chamber's edge,
  the ground must fall to the entrance's level, which is where the tunnel's
  mouth goes. The chamber itself is not in the terrain. The town makes it
  (`cities-in-the-world` decision 3, survey T13), so the screen asks only
  that there be rock to make it in.
- Both are in the Mountains biome, and no other kind may stand there.
- A sixth name table, for the mountain people, joins the other five.

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

## Decided by the owner (survey, 2026-09-27)

- **C1 (recommendation accepted):** about 50 sites, as the table in decision
  1 lists, tuned on the mockup.
- **C2, "yes":** a new player starts within walking distance, about 500 m, of
  a settlement.
- **C3, "small town nearby, place capital somewhere else. I also hope to see
  the lights on cities contribute to LOD hexes on the night side of the world
  too":**
  - the settlement near the spawn is a small town (a village or a small
    walled town), chosen by the rules and kept in the site list like any
    other;
  - one walled town is marked the capital. It is placed away from the spawn,
    on a different land mass where one can hold it. The rules choose it, and
    the owner can move it on the mockup;
  - the lights are `cities-in-the-world`'s: its night lights also light the
    coarse hexes of the far terrain.

## Built (2026-09-29): the rules in the core, and the mockup's sites

The owner, 2026-09-29: "commence 2b, lets get these damn cities up to play!"

- **One implementation.** The rules are `pbd_core::sites`, and
  `assets/config/sites.ron` holds the table, the names and the overrides.
  - The mockup's instrument is `examples/sites.rs`. It calls `generate`, so
    task 2.4's "the same list from the instrument and the core" holds by
    construction. It writes `docs/mockups/world-map/sites.json` and prints
    each stage's time.
  - The page's own placeholder rules are gone.
- **Measured on the shipped seed**, generator 6 with half the desert, on the
  cloud container's 4 threads:
  - the 163,842 level-7 cells take 0.29 s to build;
  - the screen, the full checks and the scores take 5.9 s, and keep 10,468
    candidates;
  - the pins, the home town, the greedy pass, the capital and the names take
    0.38 s.

  The design's risk of a slow fine check does not arise.
- **The list:** 55 sites, every kind at its target.
  - The small town near the spawn is Holford, a village on a river, 133 m
    from the game's default spawn direction.
  - The capital is Ashingstead, a walled town 10.3 km away, on another land
    mass.

**Findings on the way, and what was done about each.**

1. **The flatness limits were set against the coarse screen.** At the
   terrain's own 2.833 m cells, which the spec's "flat, dry ground" samples, a
   footprint's range reads 1.2 to 1.4 times the seven-sample screen's (median
   1.36 for a walled town, 1.27 for a village).
   - On the table's numbers, only 8 cells held a walled town, 3 a desert town
     and 1 a swamp village. No village stood within 500 m of the spawn.
   - Each limit in `sites.ron` is now the table's times 1.4: walled town
     11 m, village 5.5, desert town 8.5, tundra camp 5.5, jungle village 14,
     swamp village 4. That accepts, at the fine spacing, the ground the
     table's limit accepted on the mockup's screen.
   - The owner judges the result on the mockup (task 1.4).
2. **One kind per cell, chosen by its own ground.** A field cell is a walled
   town if its footprint passes the town's screen and full check, and a
   village only if not. In the mountains, a cave town comes before a cliff
   village.
   - So a change of counts never turns a kept site into another kind (the
     spec's "Fewer sites keeps the same ones").
   - The full check, the costly part, runs on the screening threads.
3. **The swamp is coastal lowland.** In 723 of its 851 cells the footprint
   reaches the sea, and the swamp village's houses stand on stilts over the
   water.
   - Its footprint may hold water as deep as the shallows (6 m).
   - Its flatness is its dry ground's, and its anchor is dry.
4. **Spacing, as the mockup showed it:** a kind keeps its own spacing from its
   own kind, and half the smaller of the two from any other, between the
   footprints' edges.
   - The spec said "the spacing for the larger of the two". With walled
     towns 2,500 m apart, that would have kept every village 2.5 km from
     every town, and cleared most of the fields.
   - The spec delta now says what is built.

**The mockup (tasks 1.2 and 1.3).**
- `docs/mockups/world-map.html` draws the core's list, with a footprint
  outline once it is bigger than its marker.
- Its site editor moves, adds, strikes, renames and crowns sites, and copies
  the edits as `sites.ron`'s pins and strikes.
  - A generated site the owner changed becomes a pin, and its id is struck.
  - A pin at sea or on a pentagon is refused on the page, as the game
    refuses it.
- `tools/mockup_sites_test.js` drives it headless: the list loads, the
  capital is dragged, and the RON pins it at its new place and strikes its
  old id.
- The base map, its finer tiles and the biome layer are redrawn on version 6
  with half the desert, so the sites stand on the ground they were placed on.
  The four desert buttons are retired, since B6 is decided.

**Not yet.** Groups 3 and 4 need the save's record store first
(`world-persistence` group 3): storing the list in a world, and the game's
map layer. The game's spawn for a new version-6 world now moves to level
ground (`taller-mountains` decision 8). When the game makes the list, it
passes that start, not the default direction the instrument uses; the small
town is within 500 m of both.
