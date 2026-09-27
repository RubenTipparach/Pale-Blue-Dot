# Proposal: every edit applies to the cell it was made on

## Why

A saved edit names its cell by a 32-bit hash of the cell's lattice address
(`Edit.cell` is `metadata[3]`, which is `point_id` in `planet_lod.rs`).
Hashes collide.

This was measured while planning `world-persistence`, over every interior
lattice point, with a replica of `point_id` in a scratch script:

| level | cells | share their id with another |
| --- | ---: | ---: |
| 8 | 647,700 | 36 |
| 9 | 2,606,100 | 794 |
| 10 | 10,455,060 | 12,432 |
| 11 (the finest, where edits live) | 41,881,620 | **202,571 (0.48%)** |

That is what a random 32-bit hash gives: about 204,000 expected at level 11.
One finest cell in about two hundred has a twin somewhere else on the planet.
A dig or a placed block on one is applied to the other as well, when the
other's column is generated. The bug is in saves today, and every world that
has been played has some edits that also land on a twin.

**The owner (2026-09-27), asked whether this should ship before the lights:
"fix cell number yes".** So this is taken out of `world-persistence` (where it
was group 1) and made its own change, first in the plan.

## What Changes

- **An exact key.** A cell's key packs its lattice address into 31 bits:
  - face, 5 bits;
  - fine level less 8, 2 bits;
  - i, 12 bits;
  - j, 12 bits.

  A point on a face's edge takes its lowest-numbered face's address, so a
  shared point has one key. The key is unique, and it can be unpacked back to
  its cell, which the region store in `world-persistence` needs.
- **The ground looks the same.** The shaders use the same lane as the random
  seed for clutter rolls and texture variation. They unpack the key and apply
  `point_id`'s mix to it, so every cell gets exactly the seed it had. A test
  holds the Rust and WGSL mixes together. Trees keep their own `tree_id`,
  which does not change.
- **Old saves are migrated once.**
  - A hashed key that names one finest cell maps to it.
  - A key that names two is resolved by the edit: its layer's altitude against
    each candidate's surface, then the distance from the saved position.
  - What is still ambiguous is applied to both, which is today's behaviour,
    and named in the migration's log line.
  - The migrated log is a new file, `edits.v1.log`. The old `edits.log` is
    left as it was. An older build reads the old file and never sees an exact
    key it would misread as a hash.
- **The video gate.** The owner watches a cell in a colliding pair dug before
  and after the fix, with its twin shown far away.

## Capabilities

### New Capabilities
- `world/persistence`: this change introduces the capability with its first
  requirement, that an edit applies to its own cell and no other.
  `world-persistence` adds the rest of the save model to it later.

### Modified Capabilities
- None. `planet/rendering`'s requirements are unchanged, because the seed each
  cell rolls with is unchanged.

## Impact

- **`pbd-core`:** a `cell_key` module holding the pack, the unpack, the
  canonical face for edge points, and `point_id`'s mix. The mix moves here
  from `planet_lod.rs` so the migration and the tests can use it.
- **`pbd-app`:**
  - `planet_lod.rs` writes the key into `metadata[3]`;
  - `planet_column.rs`, `digging.rs` and `cracks.rs` are unchanged, because
    they compare and look up by whatever the lane holds;
  - `saves/` gains the migration and the `edits.v1.log` name.
- **Shaders:** `planet_surface.wgsl` and `planet_visibility.wgsl` derive the
  seed from the key. The shader-constant test gains the mix.
- **Saves:** one-way. After migration, an older build sees the world as it was
  at the moment of migration.
- **Performance:** one unpack and one mix per cell in two shaders, which is
  cheap but not measured in a cloud session (CLAUDE.md). Loading an old save
  once builds a reverse table of 42 million hashes. That is timed in a test,
  and only the hashes the log uses are kept.
