# Design: region cell keys

## Context

- A fine cell is a vertex of the subdivided icosahedron. It is the lattice
  point `(i, j)` of face `f` at level `L`, with `i + j <= 2^L`, made
  canonical by taking the lowest-numbered face the point lies on
  (`cell_key.rs`).
- Today's key packs the canonical address into 32 bits:
  ```text
  bit 31  30..26  25..24      23..12  11..0
    0     face    level - 8   i       j
  ```
  - Rung 7 has no `level - 8`.
  - Rung 12 needs a fifth level value and a 13th bit for `i` and `j`.
- `world-persistence` design 8 plans regions as level-5 triangles (20,480
  on the home world, about 2,000 cells and about 180 m each), written as
  `regions/<id>.bin`. `multiplayer` design 1 uses the same region as its
  checksum unit.

## 1. A region is 64 cells on a side, on every body

Within a face, a level-`L` lattice is a triangle `2^L` cells on a side.
Cut it the way a level-`(L - 6)` lattice would cut it. `(i >> 6, j >> 6)`
names a rhombus of 64 x 64 lattice steps, and the rhombus is two
triangles of side 64: the upright one, where `(i & 63) + (j & 63) < 64`,
and the upside-down one beside it. A **region is one of those
triangles**, about 2,100 cells. Which half it is takes one bit.

- **Same size in metres on every body.** A cell is 2.833 m on every body
  (CLAUDE.md), so a 64-cell region is about 180 m across on every planet,
  where a level-5 region would shrink and grow with the body.
- **At L11 it is `world-persistence`'s region.** On the home world
  `L - 6 = 5`, so the level-5 regions already planned are these, unchanged.
- **The far edge.** `i` runs to `2^L` inclusive, so `i >> 6` reaches
  `2^(L - 6)`: one extra row of regions holding only the face's closing
  edge. It is kept as a real index (a narrow region), not special-cased,
  so the arithmetic stays shifts and masks.

Alternative rejected: regions as a fixed level (level 5) on every body.
On a 300 m body (L7) that is about 16 cells a region, and on a 9,600 m
body (L12) about 8,000. Both the save's unit and the network's would
change size between planets, which is the same mistake CLAUDE.md's
"cell is the same size on every body" rule forbids for cells.

## 2. The two addresses

```text
region (u32):  bit 31..27  26..23   22..12   11..1   0
               face        level    ri       rj      half   (ri, rj = i >> 6, j >> 6; half = (i & 63) + (j & 63) >= 64)
place  (u16):  bit 15..12  11..6    5..0
               0           li       lj                 (li, lj = i & 63, j & 63)
```

- **Region.** 4 bits of level cover L7 to L22 (level stored as `L - 7`).
  `half` says which of the rhombus's two triangles. `ri` and `rj` take 11
  bits each, room for 2,047 regions a side, which is
  L17 (300 m x 2^10, a 307 km radius) with the closing edge.
- **Place.** 12 bits on every body. A region's edits share its address
  once and carry only these 12 bits each.
- **The whole key** is the pair: a `u64` `(region << 16) | place` where one
  number is needed, for a map key, a record or the wire. It sorts by
  region, then place, which is the order the region digest already wants
  (`multiplayer` design 2: "by cell key ascending").
- **Canonical first.** The face is chosen exactly as today, before the
  split, so a point on a face's edge has one region and one place.

## 3. What the save writes

A region file (`world-persistence` 8) is:
- its header: the region address and the generator version;
- its edits, ordered by place, each with its 12-bit place, its layer and
  its material.

Today's flat map of keys becomes a map of regions, each holding a map of
places. Nothing iterates the outer map. The inner order is by place, so
the rule against random iteration order still holds.

## 4. Old saves

A today's key `k` names face, `L` and `(i, j)`
(`cell_key::address_of`). The new form is `(region(face, L, i >> 6,
j >> 6, half), place(i & 63, j & 63))`. That is a pure function with no loss
and no collisions, because both sides are exact addresses.

- **On open**, a save in the old form is read by the old reader, kept
  forever, and written back in the new form. This is the path
  `exact-cell-keys` took from the hash.
- **Pinned** by a test over every old key a sample world's edits use: old
  key to address to region and place, then back to the same address.

## 5. What stays 32 bits

The GPU record's `metadata[3]` and the shaders' clutter and texture seeds
stay the old hash, which `cell_key.rs` already keeps "so the ground looks
exactly as it did". No shader reads the region key, so no picture changes.

## Risks

- **Two forms in the code during migration.** The old key type is kept
  only behind the old-save reader, and everything else takes the new pair.
  The migration test pins the two together.
- **Bigger keys in memory.** About 8 bytes against 4 for an edit's key in
  memory, and fewer on disk, since a region's address is written once.
  Edits are sparse, so this is not a frame cost. It is measured with the
  bench in `planet-sizes` 3.1.
