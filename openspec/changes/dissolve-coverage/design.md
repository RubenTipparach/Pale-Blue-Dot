# Design: dissolve-coverage

## 1. What the dissolve assumes

A landing's dissolve (`detail-fade`) draws each pixel from the new partition
or the old one by an interleaved-gradient mask. A cell both partitions draw
alike is listed once; any other is listed once per partition that draws it,
marked NEW or OLD, and the fragment drops a NEW cell in the old half and an
OLD cell in the new half. That is sound only where the old partition's cells
are among the records: a NEW cell standing aside for an OLD cell that is not
there is a hole.

`detail-fade` section 4 guards this with `old_held(p)`: the old partition's
level at `p`, taken as **the finest old band that covers `p`**, must lie in
that level's ring of the new records. Where it does not, the new cell keeps
the pixel and the old one gives it up (judged per pixel since
`town-flyover`; it was judged at the cell's centre before).

## 2. What the band rings changed

`distance-lod-fade` added a cross-fade ring at each band's edge: between
`IN_L` and `OUT_L` round the centre a pixel draws level `L` or level `L - 1`
by the Bayer mask. So across old band `L`'s ring the old partition draws
**two** levels, and `old_held` checks only `L`.

The new set's records are laid to cover the old partition (`cover_ring`):
level `L - 1` inward to the old band `L`'s inner ring edge, less the distance
between the anchors. That distance is capped at the level's margin (the
rebuild distance and three tiles), so after a move longer than the margin,
which fast flight makes at every landing, level `L - 1`'s records stop short
of the old ring's inner part. There `old_held` finds level `L` held, the new
cell stands aside, and the old partition's level `L - 1` pixels have no cell.
That is a dithered hole the size of a band's ring, at most of the landing's
pixels while it runs, which is the shape of the owner's report (the land
past the harbour half-transparent in take 1 at 105 s).

This is the hypothesis. Section 4 measures it before anything is changed.

## 3. The rule

`old_held(p)` is true when, for **every** fine level the old partition can
draw at `p`, that level's ring of the records holds `p`. Level `L` can draw
at `p` when `p` is inside `OUT_L` and outside the next finer level's inner
ring edge `IN_{L+1}` (round the old partition's centre). The base is always
held.

The fragment decides a cell's level by the band fade at the cell's owner and
at its centre, not at the pixel, so a level reaches up to a tile of the level
below past `OUT_L`, and a tile of its own inside `IN_{L+1}`. The rule widens
each level's reach by two such tiles. That errs toward the new cell keeping a
pixel the old one could have drawn, which is an overlap for the fade's second,
never a hole. The records' rings are already shrunk by a tile
(`records_cosines`), so a pixel inside one belongs to a cell whose centre is
inside it.

The visibility pass uses the same function at a cell's centre to list a NEW
cell unmarked (drawn whole) where the old partition is not held; the surface
pass uses it per pixel. The two copies are the same text and a test holds
them so.

## 4. Measurement before the change

A GPU test runs the shipped `old_held` (the visibility shader's copy, with a
probe entry point added in the test) over a grid of points round a landing:

- an old fine set laid at anchor `A` and a new one at `B`, `d` metres on,
  laid to replace it (`generate_fine_live` with `replacing`), at the band
  ring width the game ships (`lod_fade_width` 0.3) and the live bands of a
  height `h`;
- the old partition's centre at its fit from `A` toward `B` (the fade centre
  trails the camera);
- the uniform's rings as `records_cosines` makes them.

At each point it asks which fine levels the old partition can draw there
(section 3's reach), and whether the new records hold a cell of that level
containing the point (the nearest record's own hexagon). A **hole** is a
point `old_held` calls held where a level it can draw has no cell.

Cases: `h` 0, 60 and 150 m; `d` 20, 80, 200 and 400 m.

Results: see "What measuring found".

## 5. Verification

- The GPU test above, made to assert no holes on every case once the rule is
  changed.
- The two `old_held` copies are identical.
- On the owner's word that the computer is free: take 1's path flown again
  with `PBD_NO_SKY=1 --snap-every 0.1`, and the hole detector's worst frame
  down to the false-positive floor (about 6,000 px).

## What measuring found

`dissolve_coverage_on_real_fine_sets` (`planet_visibility_tests.rs`), on an
RTX 3070, 412,865 points on an 8 m grid out to 2.9 km round the new anchor.
"Held" is the points where the new ground may stand aside; a hole is a held
point where a level the old partition draws at that pixel has no record
(strict), or where a cell of it can reach (slack: a tile and a half past the
band, inside the rule's two).

**The shipped rule** (finest covering level only):

| h m | d m | held | holes | where |
| ---: | ---: | ---: | ---: | --- |
| 0 | 20 / 80 | 412,865 | 3 / 1 | instrument rounding (below) |
| 0 | 200 | 403,021 | 1,560 | L8 at 800 m, L9 at 400 m, L10 |
| 0 | 400 | 376,328 | 6,912 | L8 at 800 - 1,000 m, L9 at 400 - 500 m |
| 60 | 200 | 408,731 | 476 | L8 at 800 m, L9 at 400 m |
| 60 | 400 | 382,790 | 5,798 | L8 at 800 - 1,000 m, L9 at 400 - 500 m |
| 150 | 400 | 390,806 | 3,874 | L8 at 800 - 900 m, L9 at 400 - 500 m |

The holes are exactly section 2's: the coarser level on the inside of the
finer band's ring (L9's inner ring edge is 840 m, L10's 420 m), and only once
the move outruns the margin. All twelve cases: 18,636 strict holes.

**Section 3's rule:** 0 strict and 0 slack holes in all twelve cases. Its
cost is the share of the ground drawn whole at a landing instead of
dissolved: held falls from 376,328 to 358,899 points at `h` 0, `d` 400 (91%
to 87%), and is unchanged below `d` 80 at 60 m and 150 m.

Two things building the instrument found:

- The rule's inner reach on the finest level is zero, and `from_old > 0`
  failed at the old centre itself, so a pixel exactly there skipped the
  finest level's check. It is `>=`.
- The instrument's own errors, fixed before the numbers above: the nearest
  record centre is not always the cell a point lies in on the geodesic dual
  (it reported 3% of every band as holes), and a point on a corner where two
  of a hexagon's corners nearly meet falls outside all of them by a
  centimetre in `f32` (a few cm of tolerance).

Not yet measured: the game. Take 1's path flown again with the hole detector
(section 5) waits on the owner's word that the computer is free.
