# Design: world persistence

## Context

See `proposal.md` for the owner's words and why. What the design has to work
with, observed on `main` (2026-09-27):

- **A slot is a directory** (`crates/pbd-app/src/saves/`) holding four files:
  - `world.ron` holds the name, the seed, times, the pose, the selected slot
    and the world clock. It is replaced whole every 5 s, when the menu opens,
    and on exit.
  - `edits.log` is append-only, one line per record:
    - an edit (cell, layer, material) with the hotbar after it;
    - a kit grant;
    - a catch;
    - a change of tool.

    A torn write costs the last line.
  - `weather.bin` is the atmosphere's state, replaced every 60 s.
  - `vehicles.ron` holds every craft, replaced whole when one changes.
- **One writer thread** owns the files. It appends or replaces, fsyncs, and
  publishes a high-water sequence mark. An edit is acknowledged only at the
  mark, which is how CLAUDE.md's durability rule is met without blocking a
  frame. This is kept unchanged.
- **Loading** checks the seed, replays the whole log into `Edits` (a
  `HashMap<u32, Vec<(layer, material)>>`), and rebuilds the tier. Every edit
  in the world is in memory, because the fine LOD rebuild reads them.
  Compaction and a seed per world are held in `save-slots`' design.
- **Versions.**
  - `GENERATOR_VERSION` (4) is in `terrain.rs`, `RECORD_VERSION` (1) for
    vehicles, and `KIT_VERSION` for the starting kit.
  - None of the three is written to `world.ron`.
  - There is no topology version constant, though CLAUDE.md says topology and
    generator versions are part of a saved world's id.
- **An edit's cell key is a hash, and it collides.**
  - `Edit.cell` is the LOD record's `metadata[3]`, which is `stable_id`, which
    is `point_id` (`planet_lod.rs`): a 32-bit mix of the lattice address
    (face, i, j, level).
  - It was measured over every interior lattice point with a replica of
    `point_id`, in a scratch script that is not committed. Task 1.1 makes it
    a test.

    | level | cells | share their id with another |
    | --- | ---: | ---: |
    | 8 | 647,700 | 36 |
    | 9 | 2,606,100 | 794 |
    | 10 | 10,455,060 | 12,432 |
    | 11 (the finest, where edits live) | 41,881,620 | **202,571 (0.48%)** |

    That is what a random 32-bit hash gives: about 204,000 expected at level
    11.
  - One finest cell in about two hundred has a twin somewhere else on the
    planet, and an edit to either is applied to both when the other's column
    is generated.
  - A hash also cannot be turned back into a place, which a region store
    needs.

## Goals / Non-Goals

**Goals:**
- One model for everything a world holds: derived, stored, or authored.
- A save that keeps what it must through retunes, and lets the rest follow
  the rules.
- The player is not the only author: the world can change itself, durably
  and in order, and never over the player's work.
- A world whose history grows for months still loads quickly, in bounded
  memory, and never loses a committed entry.
- Every edit applies to exactly the cell it was made on.

**Non-Goals:**
- **Town growth, abandonment, population or economy.** The owner: "you can
  just prepare for this, no need to worry about city and population systems
  at this time." This change builds the interface and a test process only.
- **Multiplayer.** The journal is shaped to be the replication stream, and
  processes to run only on an authority. No network code is written.
- **Moving vehicles or weather into the record store.** Vehicles are records
  in all but file. Weather is simulated state, replaced whole, and stays so.
  Folding vehicles in is a follow-up with no behaviour change.
- **Time passing while the game is closed.** The owner (2026-09-27): "no world
  time is only in game". World processes run on the world clock, which
  advances only while the world is played. A town grows while you play,
  wherever you are on the planet, and not while the game is closed.

## Decisions

**1. Laws are derived, facts are stored.** The owner asked me to decide where
the line goes. The test is whether a thing must survive a change to the rules
that made it, or can change on its own.

| thing | kept as | why |
| --- | --- | --- |
| terrain, biomes, rivers, caves | derived | the seed and the generator version decide them; a fix to the generator reaches every world of that version |
| trees, clutter, texture variation | derived | cosmetic and tuned often |
| water classes, fish habitat, where a school can spawn now | derived | read from the live climate and the species rules, which are tuned often; storing them would freeze a tuning mistake into every save |
| the climate normals | a shipped asset | derived offline, checked against the world (`climate-and-fish-maps`) |
| site candidates | derived | only the chosen sites are facts |
| **sites** | **record** | must not move when the rules are retuned (`city-sites`) |
| **settlements and their buildings** | **record** | will grow and be abandoned; must not change under a played world |
| **landmarks** (a named ruin, a shrine, a wreck) | **record** | a named place the player may return to; none is designed yet |
| **species zones with a name or a state** (a named fishing ground, a stock that can be fished down, a legendary creature's lair) | **record** | once a zone can change or be named it is a fact; the habitat under it stays derived |
| player edits, placed pieces, doors | **journal entries** | authored changes |
| world changes (a building raised, a house abandoned) | **journal entries** | authored changes by a process |

- Species zones that are only habitat stay derived. That is most of the fish.
  A zone becomes a record only when something about it has to be remembered.
  The store can hold one today, and no change in this plan creates one.

**2. Records are generated for the whole planet when the world is made.**
- Sites and settlements are made at world creation, or on the first open of
  an older world under the new version, and written before anything is shown.
- A settlement stores each building as its own definition, not as pieces.
  The definition is:
  - the plot's cells;
  - the walls and their openings;
  - the stairs;
  - the storeys;
  - the kit;
  - the roof;
  - a state (standing, abandoned, ruined).

  It is not a reference to a template, which could change. Pieces are derived
  from the definition by the cut rules, so a fix to a stair reaches every
  stored building, and each building keeps its identity.
- The size is estimated, not measured: about 50 settlements × 20 buildings ×
  a few hundred bytes is roughly 500 KB, plus 2 KB of sites.
- *Alternative:* generate a settlement when the player first comes near.
  Rejected: a town must be able to grow while nobody is there, and the map
  shows every town from the start.

**3. The identity is its own file, written once.**
- `identity.ron` holds the seed, the topology version, the generator version,
  each record kind's schema version, and the cell-key version.
- It is written with a barrier at creation, and again only when a world is
  deliberately upgraded.
- It is not in `world.ron`. That file is rewritten whole every 5 s, and a
  build that predates this change would rewrite it without the identity.
- A slot with no `identity.ron` is a save from before this change. Its
  identity is written before anything else happens: generator 4, topology 1,
  and cell-key version 1 if `exact-cell-keys` has migrated it to
  `edits.v1.log` (0, the hash, otherwise).
- `TOPOLOGY_VERSION` becomes a constant beside `GENERATOR_VERSION`, starting
  at 1.

**4. Cell keys become exact.** This is now its own change,
`exact-cell-keys`, which ships first (the owner: "fix cell number yes"). The
summary stays here, because the region store depends on it.
- A finest cell's key packs its lattice address: face (5 bits), and i and j
  (12 bits each, since a face's corner sits at 2,048). That is 29 bits, for
  the finest level, which is where edits live.
- A cell on a face's edge takes its lowest-numbered face's address, so a
  shared point has one key.
- The key is exact and reversible, so the region store can find a cell's
  region from its key. It fits the GPU record's `metadata[3]` as the hash did.
- Tree and texture rolls keep their hash: a collision there only repeats a
  look.
- **Migration** (detailed in `exact-cell-keys`). An old log's hashed keys
  are resolved by enumerating the finest cells once, about 42 million hashes.
  A hash that names one cell maps to it.
- A hash that names two (0.48% of cells) is resolved by the edit itself: the
  layer's altitude against each candidate's surface, and the saved position.
  What remains ambiguous is applied to both, as today, and listed in the
  migration's log line.
- The migrated log is `edits.v1.log`, and the old `edits.log` is kept
  untouched, so an older build never misreads an exact key as a hash. This
  change's journal takes `edits.v1.log` as its first segment.

**5. The journal carries authors, and records change through it.**
- `edits.log` becomes the journal. Every line gains an author token:
  - `@p` for the player (player 0 in single player);
  - `@w<process>:<record>` for a world process;
  - `@c` for the world's creation.

  A line with no author token is an old line, and reads as the player's.
- A new line kind, `rec <id> <body>`, carries a record's new value whole. A
  record is small, and a whole value replays with no ordering question within
  it.
- Record ids are stable: a site's is its anchor cell, and a building's is its
  settlement's id with the plot.
- The writer thread, the fsync and the high-water mark are unchanged.

**6. The world yields to the player.**
- Before a world-authored entry is written, it is checked against a set of the
  cells, pieces and record fields the player has touched. That set is kept
  per region, and rebuilt from the journal and the snapshots at load.
- A refused proposal is returned to its process as refused, never written in
  part.
- This is the same rule as `cities-in-the-world`'s "a settlement never
  overwrites the player's work", applied to every later change a world makes.

**7. World processes are pure, periodic and journaled.**
- A process has an id, a period in world-clock seconds, and
  `evaluate(seed, period, store) -> proposals`.
- A steward runs each process's due periods in order: at load, and each
  in-game hour. It writes the accepted proposals as authored entries, and
  records `evaluated_through` per process in the record store.
- Loading replays the journal and never runs a process.
- Catch-up after a long absence runs every missed period in order. A process
  may offer a closed form for many periods, and the spec's catch-up test holds
  it to the stepwise answer.
- `--age-world <days>` advances the clock through the steward, for tests and
  for the video.
- Only a test process exists in this change. It abandons a test building
  every seventh period. Town growth and abandonment are written later, as
  processes behind this interface.

**8. A world larger than memory loads by region.**
- A region is a level-5 triangle of the icosahedron. That gives 20,480
  regions of about 2,000 finest cells, about 180 m across. The region is
  found from a cell's key by shifts alone: face, i >> 6, j >> 6, and which
  half of the rhombus.
- **The journal is segmented** (`journal/000123.log`). A checkpoint runs on
  the writer thread when the live segment passes a threshold (starting at 8
  MB) and on a clean exit:
  1. Write each region touched since the last checkpoint to
     `regions/<id>.bin`, through a temporary file and a rename. A snapshot
     records the last sequence number it includes.
  2. Write `records.bin` the same way.
  3. Write `manifest.ron` through a temporary file and a rename, naming the
     segment replay starts from.
  4. Only then delete the older segments.
- A crash before step 3 leaves the old manifest and every segment. A crash
  after it leaves the new manifest, and replay skips any entry a snapshot
  already holds by its sequence number. No committed entry is lost or applied
  twice.
- **Loading** reads the manifest, the identity, the records, the regions
  within the fine set's reach plus a margin, and every segment after the
  checkpoint. Entries for regions not yet loaded are held until they are.
  Regions further away load on the pool as the player approaches, before the
  fine set reaches them.
- The coarse levels far away never read cell edits today. A change big
  enough to see from afar, such as a new building, comes from the records,
  which are global.

**9. `tools/save_inspect.py`** prints:
- the identity;
- each record kind's count;
- the journal's entries by author;
- the region count and sizes;
- the checkpoint's position.

It is how the owner, and the video, see what a world holds.

## Risks / Trade-offs

- [Migrating hashed keys guesses wrong for an ambiguous edit] → Ambiguity is
  resolved by the surface and the saved position first. The rest is applied
  to both, which is exactly today's behaviour, and listed. Nothing is lost
  that is not already wrong.
- [The checkpoint is the most delicate code in the save] → It is tested by
  killing the writer at each step in turn (a fault-injection hook in the
  writer, test-only), and by replaying every resulting directory.
- [Regions add load latency when the player moves fast, in a ship] → Regions
  are loaded ahead of the fine set's reach, sized by the fastest craft's
  speed. The ship test flies the far-side route over a save with a million
  edits and asserts no fine cell is built before its region is loaded.
- [The yield set grows with everything the player touches] → It is per region
  and loaded with the region. A player who digs everywhere pays in the
  regions they dug, not globally.
- [Timing] → The load of a million-edit save is timed in a test. Frame cost is
  not measured in a cloud session (CLAUDE.md), so the owner runs
  `tools/perf_suite.py` before the region loader merges.

## Migration Plan

1. An old slot gains `identity.ron` (generator 4, topology 1). Its key
   version is 1 once `exact-cell-keys` has migrated it to `edits.v1.log`.
2. `edits.v1.log` becomes the journal's first segment. The old `edits.log`
   stays untouched for older builds.
3. Records are generated by the changes that introduce them (`city-sites`,
   `cities-in-the-world`) on the first open under their versions.
- Rollback to a build before this change opens the old `edits.log` if it
  still exists, and nothing after it. The format comment and the release note
  say so. A world played under this change is not readable by an older
  build, which is the usual one-way limit.

## Decided by the owner (2026-09-27)

- **"fix cell number yes":** the exact keys ship first, as `exact-cell-keys`.
- **"no world time is only in game":** the world clock, and every process on
  it, advances only while the world is played.
