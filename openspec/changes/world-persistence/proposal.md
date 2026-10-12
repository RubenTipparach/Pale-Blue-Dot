# Proposal: world persistence: seeded, stored, edited

## Why

**The owner (2026-09-27), before the cities work begins:**

> "one of the things I want to call out before we begin is: Save persistence.
> the world on its own is just seeded; towns and special landmarks can be
> generated as part of the 'save' file, the same can be said about certain
> zones of species and such, I'll let you decide what is a better approach to
> this; in addition, the players influence: removing, or adding a block will
> add more saves to the game; in addition over time towns can grow, buildings
> can be abandoned, you can just prepare for this, no need to worry about city
> and population systems at this time. Just know that other factors besides
> players can affect the massive save world"

**A save today** (`crates/pbd-app/src/saves/`) is a directory with four files:
- `world.ron`: the seed, the pose and the clock, replaced whole on a timer;
- `edits.log`: one fsynced line per dig or place, carrying the hotbar;
- `weather.bin`: the atmosphere's state, replaced every 60 s;
- `vehicles.ron`: every craft, replaced whole when one changes.

One writer thread owns the disk and acknowledges an edit only after it
fsyncs. That meets CLAUDE.md's durability rule, and it is kept.

**What it cannot hold.** The cities plan needs the save to hold more than
this:
- **No version but the seed.** The main spec says the generator version is
  part of the world's identity, and the save does not record it.
- **Nothing generated is stored.** The plan's sites and towns have to outlive
  a retune of the rules that made them. They also have to be able to change
  later, as a town grows or a house is abandoned. Nothing in the save can hold
  a generated thing.
- **Only the player changes the world.** A log line has no author. The owner
  says towns will grow and buildings will be abandoned on their own, and
  those changes need the same durability and the same order as the player's.
- **The whole world is loaded at once.** Loading replays the whole log into
  memory, and compaction is held (`save-slots` design, "Held"). A planet whose
  towns grow, and whose players dig for months, outgrows that.

This change settles the model before any change writes new kinds of state.
The first to do so are `bigger-biomes` (the generator version) and
`city-sites` (the site list).

## What Changes

- **Three layers, one rule: laws are derived, facts are stored.**
  - **Derived, never written.** Anything the seed and the rules decide: the
    terrain, biomes, rivers, trees and clutter, water classes, fish habitat,
    and the candidates a site is chosen from. A retune of a rule changes
    these in every world. That is how a bug fix reaches old saves.
  - **Stored records.** Anything generated once that must then stay put or be
    able to change: sites, settlements and their buildings, landmarks, and
    species zones that have a name or a state. These are generated when the
    world is made and written to the save. From then on the save is their
    truth.
  - **Authored changes.** Every change after that, whoever makes it: a dig, a
    placed block, a placed or removed piece, a door, and the world's own
    changes (a building raised, a house abandoned). Each goes through one
    durable journal and names its author.
- **A world identity** records every version that shapes the world:
  - the seed;
  - the topology version;
  - the generator version;
  - each record kind's schema version.

  A save refuses to open under a version it does not have. A new version is
  never silently read as an old one.
- **A record store** in the save, holding kind-tagged records with stable ids.
  A new record kind is added without changing the save's format. A record of
  a kind the build does not know is kept, and written back unchanged.
- **An authored journal.** `edits.log` becomes the journal. Each entry names
  its author:
  - the player;
  - a world process, with the record it acted for;
  - the world's creation.

  The writer thread, its fsync and its high-water mark are unchanged.
- **The world yields to the player.** A world-authored change that would alter
  a cell, a piece or a record the player has changed is refused.
- **World processes are prepared, not built.** This change defines the
  interface and proves it with a test-only process:
  - a process is a pure function of the seed, the world clock's period, and
    the stored state;
  - its results are journaled like a player's, and replaying a save never
    re-runs a process;
  - catch-up after time away is exact.

  No town growth or abandonment is written here. That is a later change, and
  the owner has said there is no need for city or population systems yet.
- **A world that does not fit in memory.**
  - The journal is checkpointed into region snapshots: one per level-5 cell,
    10,242 regions about 181 m across.
  - Loading reads the regions near the player and the journal's tail.
  - Regions further away load as they are approached.
  - A checkpoint is crash-safe: it writes new snapshots, then moves the
    journal's start, and never deletes a line that is not yet in a snapshot.

## Capabilities

### New Capabilities
- `world/persistence`: what a saved world is made of:
  - the identity and its versions;
  - derived against stored;
  - the record store;
  - the authored journal;
  - the world yielding to the player;
  - world processes and their catch-up;
  - loading by region, and crash-safe checkpoints.

### Modified Capabilities
- None in the main specs. The durability requirements live in `save-slots`
  and `vehicles` (`player/vehicles`: "Vehicle records use the durable save
  path"), and they are kept as they are. `bigger-biomes`' change to
  `planet/terrain` ("The generator is versioned") is served by this change's
  identity, and its tasks are updated to use it.

## Impact

- **`pbd-core`:** a `world_store` module, engine-independent, covering:
  - the identity;
  - the record type and store;
  - the journal entry and its author;
  - the yield rule;
  - the process interface;
  - the region map from cell to level-5 region.

  The rules live in the core, so a future server runs the same ones.
- **`pbd-app`:**
  - `saves/format.rs` gains the identity, the record lines and the author
    field;
  - `saves/writer.rs` gains checkpoint jobs;
  - `saves/mod.rs` loads regions near the player first;
  - the LOD rebuild reads edits through the region store, not one map.
- **Save format:**
  - `world.ron` gains `identity`;
  - a `records/` file holds the records;
  - the journal's lines gain an author;
  - `regions/` holds the snapshots.

  An old save opens: its lines have no author and read as the player's, its
  versions read as the ones it was made under, and it has no records until
  the changes that make them run on it.
- **Tools:** `tools/save_inspect.py` prints a save's identity, records, the
  journal's authors and the region sizes. The owner and the tests can see
  what a world holds.
- **Performance:** loading by region bounds both load time and memory. The
  checkpoint runs on the writer thread. Neither is measured in a cloud session
  (CLAUDE.md). A synthetic save with a million edits is timed in a test.
