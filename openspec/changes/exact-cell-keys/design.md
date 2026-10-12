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

**2. The key rides the record's spare lane, and the seed stays where it is.**
Revised at implementation (2026-09-27), before any code:
- The record already carries a lane nothing uses: `spare.y` (`GpuCell`,
  `planet.rs`, "yzw spare"). The key goes there.
- `metadata[3]` keeps the old hash, which the shaders read as the seed for
  the clutter rolls and the texture variation. So no shader changes, and
  every flower, pebble and variation is the value it was, by construction.
- The Rust readers (`planet_column.rs`, `planet_lod.rs`, `digging.rs`,
  `cracks.rs`) read the key through one accessor, `GpuCell::key()`.
- `point_id`'s mix moves into `pbd_core::cell_key`, where the collision test
  and the migration use it. `planet_lod.rs` calls it from there.
- The record's size is unchanged, since the spare lane is already uploaded.
- *The design as first written* put the key in `metadata[3]` and had both
  shaders unpack it and re-apply the mix. That gives the same seeds, but it
  changes two shaders and needs a Rust-WGSL agreement test for the mix. It
  also has to treat the base level's records, whose lane holds a plain index,
  differently from the fine levels'. The spare lane needs none of that.
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

## Implementation notes (2026-09-27)

Measured on the 4-core cloud container, in the workspace's test profile
(`opt-level` 1):
- **The collision test** (task 1.1) hashes every interior finest cell and
  counts the repeats. It gives 202,571 of 41,881,620, which is the scratch
  measurement exactly. It runs in 2.05 s.
- **The uniqueness test** (task 2.1) keys every address of every finest
  point, 42,004,500 addresses. It finds exactly 41,943,042 distinct keys,
  which is `10 · 4^11 + 2`, the number of points. So no two points share a
  key, and a seam point gets one key from every face it lies on. It runs in
  2.40 s.
- **Two anchors across a seam** (task 2.2): the finest bands laid 80 m either
  side of the edge between faces 0 and 4 share 39,635 cells, 261 of them on
  the seam, and every shared cell has one key in both.
- **The migration** (task 4.1) of a synthetic 100,000-edit log takes 1.97 s,
  most of it the one pass over the 42 million points that builds the reverse
  table. Of the 100,000 edits:
  - 98,953 had a hash naming one cell;
  - 193 were settled by the ground;
  - 854 were settled by the saved position;
  - none was left ambiguous.

  The release build is faster, and was not measured.
- **The barrier.** The migrated log goes through the save thread as a
  whole-file replacement, which is written beside the file, synced and
  renamed. `WorldSave::open` waits for it with the thread's `drain` before it
  returns, and so before the world is shown. A failed write is the writer's
  reported failure, so `accept` refuses edits from then on, as it does for
  any failed write.

**The gate video** (task 5.1) was made by hand from captures, as decision 4
says:
- The pair is not the one nearest the spawn. That pair's two cells stand on
  ground 31 m apart, so an edit at one's surface is in open air at the
  other's, and the old build shows nothing at the twin. The video uses the
  nearest pair on one ground layer: cell A 303 m from the spawn, cell B
  1,359 m, both on ground at 98 m.
- Each shot opens the world with the saved pose moved beside the cell. The
  old build is `ecabf4f`.
- `--dig 1 --place 3` digs A and stacks three stones. The old build shows
  them on B as well, and the new build does not.
- The meadow and surface views match the old build to the pixel.

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
