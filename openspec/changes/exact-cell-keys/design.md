# Design: exact cell keys

## Context

See `proposal.md` for the measurement and the owner's answer. What the
design has to work with, observed on `main` (2026-09-27):

- **The key.**
  - `planet_lod.rs`: `stable_id(cell)` is `point_id(cell.point)`, a mix of
    `(face, i, j, level)` into 32 bits.
  - It is written to the record's `metadata[3]` for every fine level (8 to
    11).
  - `tree_id` is a separate mix of the finest address at a cell's centre, and
    it goes in its own field.
- **Readers of `metadata[3]`:**
  - **Rust:** `planet_column.rs` (`edits.for_cell`), `planet_lod.rs` (the
    same, for adopting a column), `digging.rs` (the edit's `cell`, and finding
    the record again) and `cracks.rs` (finding the record being broken). All
    four compare or look up. None does arithmetic on it.
  - **WGSL:**
    - `planet_surface.wgsl` uses `metadata.w` as `id` for the clutter rolls
      (`roll(id, SALT_*)`), and as `seed` for `cell_variation`
      (`random(seed)`).
    - `planet_visibility.wgsl` uses it for the same rolls, to decide
      eligibility.
    - Both go through `hash(x)`, a PCG-style mix, so any 32-bit input rolls
      well. But a different input rolls differently: the flowers would move.
- **Lattice addresses.**
  - At level L, face f's points are `(i, j)` with `i + j <= 2^L`, so i and j
    reach 2,048 at level 11 and need 12 bits.
  - A point on a face's edge has an address on each face it lies on.
    `LocalDual` keeps the first address it meets while deduplicating
    vertices, in triangle order.
- **The log.** `edits.log` has one line per edit: `cell layer material`, plus
  the hotbar. `parse_line` returns `None` for a line it cannot read, and the
  loader counts it as damaged and skips it.

## Goals / Non-Goals

**Goals:**
- An edit lands on exactly its cell, today and in every old save that can be
  resolved.
- The ground looks exactly as it did: the same flowers, pebbles and
  variation.
- A key the region store can turn back into a place.

**Non-Goals:**
- **The rest of the save model** (identity, records, journal authors,
  regions). That is `world-persistence`.
- **Changing tree placement.** `tree_id` stays a hash. A collision there only
  gives two far-apart cells the same tree roll.

## Decisions

**1. The key is the address, packed.**
- `key = face << 26 | (level - 8) << 24 | i << 12 | j`. That is 5 + 2 + 12 +
  12 bits, and bit 31 is left clear.
- Coarser fine levels (8 to 10) get keys too, since they carry `metadata[3]`
  today. Only level 11 is ever edited.
- **Edge points.** A point on a face's edge is re-addressed on the
  lowest-numbered face it lies on before it is packed. That is a pure
  function of the address: an edge point `(f, i, j)` maps to its neighbour
  face's `(i', j')` through the icosahedron's edge table.
- So the key does not depend on which triangle the cap met first. A test asks
  for the same cell from two anchors on different faces and gets one key.

**2. The seed is the old hash, recomputed from the key.**
- The shaders unpack `(face, level, i, j)` from the key and apply
  `point_id`'s mix. The result is the exact value `metadata[3]` used to hold.
  `roll` and `random` are then unchanged.
- The mix moves into `pbd_core::cell_key`, and the WGSL copy is checked
  against it on sample keys by the shader-constant test family. The test
  fails if either copy changes (CLAUDE.md: validate the actual artifacts).
- *Alternative:* feed the key straight into `hash`. Rejected: every flower,
  pebble and variation would move once, and the owner would see a change this
  fix should not make.

**3. The migration resolves by the edit's own evidence.**
- On opening a save that has `edits.log` and no `edits.v1.log`:
  1. Collect the hashes the log uses.
  2. Enumerate the finest cells once, and keep a reverse table for only those
     hashes.
  3. For a hash with one cell, write that cell's key.
  4. For a hash with two or more cells, prefer the candidate where the
     edit's layer lies within a few layers of the generated surface. A dig
     60 m under one twin's ground and 1 m under the other's settles it.
  5. If that does not settle it, take the one nearest the saved position.
  6. If nothing tells them apart, write one line for each candidate, which is
     exactly what the game does today, and name the hash in the log.
- The result is written to `edits.v1.log` through the writer thread, with a
  barrier, before the world is shown. `edits.log` is never written again.
- *Alternative:* rewrite `edits.log` in place. Rejected: an older build would
  read exact keys as hashes and put the edits on the wrong cells.

**4. The gate video is made by hand, since `step-videos` is not built yet.**
- A capture sequence (`--capture` stills along a scripted walk) is stitched
  with Playwright's ffmpeg, as CLAUDE.md allows until the tooling exists.
- The pair is the colliding pair nearest the spawn, found by the instrument
  that measured the collisions.
- Shots:
  - dig cell A;
  - go to its twin B on the old build and show it dug;
  - the same on the new build, with B untouched;
  - a reload;
  - a still of the same view on both builds, showing the ground unchanged.

## Risks / Trade-offs

- [The edge canonicalisation is wrong for some edge, and a cell gets two keys
  from two anchors] → Tested on every edge of the icosahedron, both
  directions, at level 11.
- [The migration mis-resolves a pair] → It can only do so where the surface
  and the position both fail to tell them apart. Then it applies the edit to
  both, which is today's behaviour. Nothing is lost that is not already
  wrong.
- [Enumerating 42 million cells at load] → It happens once per old save, and
  only the used hashes are kept. It is timed in a release test. If it is slow,
  the enumeration is limited to faces whose hash ranges the log touches.
- [WGSL cost] → An unpack and one extra mix per cell, per pass that rolls.
  Not measured in the cloud (CLAUDE.md). The owner's `perf_suite.py` run
  covers it.

## Migration Plan

- One-way and automatic on first open. `edits.log` is kept untouched beside
  `edits.v1.log`.
- Rollback: an older build opens `edits.log`, the world as it was at
  migration, with its old twin behaviour.
