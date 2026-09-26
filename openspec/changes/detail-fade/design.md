# Design: detail-fade

## 1. Trees

In `planet_surface.wgsl`'s fragment, for a tree (`kind == 2`):

```text
fade = clamp((range - margin - distance) / tree_fade_m, 0, 1)
discard where bayer4(pixel) >= fade
```

`range` is `settings.w` (the foliage range, zero when foliage is off);
`margin` is 4 m, over a cell's half-width, so a tree is gone before its
centre crosses the line the visibility pass tests. `tree_fade_m` rides the
spare lane `clutter_more.w`, from `scatter.ron`, validated to lie in
`0..=FOLIAGE_DRAW_DISTANCE` (0 draws the old hard line). The mask is fixed to
the screen, so a tree standing still at the fade distance holds a still
pattern rather than shimmering.

## 2. The landing cross-fade (stage two)

Uniform additions: the previous anchor and fade progress (`lod_prev`: xyz,
w = 0..1 through the fade) and the previous bands (`bands_prev`).

- **Visibility:** a cell is submitted when the new partition draws it, and,
  while the fade runs, when the old one does. A cell whose every partition
  test (its level's band, its owners, its neighbours' coverage for the walls)
  agrees under both is submitted once, marked BOTH. One that differs is
  submitted once per partition that draws it, marked NEW or OLD in the high
  bit of its visible-list entry.
- **Vertex:** the partition tests use the marked partition's anchor and
  bands; BOTH uses the new ones.
- **Fragment:** NEW keeps the pixel where `bayer4 < progress`, OLD where
  `bayer4 >= progress`, BOTH always; the split midpoint cell's per-fragment
  test uses the marked partition.

**The walls' floors.** A cell's wall down to a finer neighbour's floor reads
that floor from the record (`floor_of`), and the CPU bakes floors only for the
sides its own partition needs (`floor_rule`: a neighbour inside the next finer
level's complete radius, plus two tiles). The old partition's band edges lie
elsewhere, so a set built for the new anchor alone lacks the floors the old
partition's walls read, and the ring would show cracks for the fade. The
request therefore carries the partition it will replace (the resident set's
anchor and complete radii), and `floor_rule` bakes a floor where EITHER
partition needs one. An extra floor is only read where a wall asks for it, so
the new partition draws exactly as before.

**Coverage, checked at each landing:** each fine level's records are a ring
round the new anchor (`lay_band`: from the next finer band less the margin to
its own band plus the margin, the margin `regen_m` plus three tiles, or less
where capacity truncated it). The old partition draws level `k` between the
old complete radii of `k + 1` and `k` round the old anchor. With the anchors
`d` apart, the old ring lies inside the new one when
`old_outer + d <= new_outer` and `old_inner - d >= new_inner` (or both inner
radii are zero). The upload checks every level; if any fails, that landing
does not fade (it pops as it does today) and says so in the log, so a flight
that outruns the margin cannot draw holes.

**The risk this settles:** the old partition's fine cells
must still be in the NEW set's records. The new set is laid out to its live
bands plus the regeneration margin (`regen_m`) round the new anchor, and the
old anchor is about `regen_m` from the new one, so the old bands should fit;
but the bands also change with height, and the player moves on while a set
builds. The first step of stage two is an instrument that counts, at each
landing on the `walk` and `far-side` scenarios, the old partition's cells the
new records do not hold. If any are missing, the fade runs only for the
levels whose old cells are all present.

## Verification

- Trees: a still at 1150 m from a tree line (dithered), and a walk and a
  fly-through recording for the owner.
- Frame time: the perf suite, old exe against new, on `walk`, `far-side` and
  `cloud-hop`. The discard costs early depth testing for trees only.

## What building it found

- **Trees fade at the edge of their band, not at 1200 m.** Trees stand on the
  three finest levels and only on cells the partition draws, so the edge of
  the tree region is the outermost tree level's band edge round the anchor,
  and from height (where `live_bands_m` shrinks the bands) it is far inside
  1200 m. Faded at the camera distance alone, the fade never met a tree (a
  capture at 600 m up changed 88 pixels). The fragment now fades toward the
  nearer of that band edge (in the tree's own partition) and the foliage range.
- **Between landings nothing about the trees changes.** The band edges are
  fixed to the anchor, so a tree appears, vanishes or changes size only when a
  set lands: the landing cross-fade is the fix for the tree pop as much as for
  the blocks'.
- **Coverage on real runs** (the `LOD_FADE skipped` log line): a 1 km walk
  faded every landing (about thirty); the far-side flight skipped the few
  landings where its bands resized in the climb, and the landing that first
  replaces the startup set (built with no partition to replace) is skipped.
- **Holes:** none. The route frame captured with the fade stretched to five
  seconds, with `PBD_NO_SKY` (the magenta hole detector), shows no magenta
  below the horizon with the fade on or off.
- **Tests:** `a_landing_fades_only_when_the_new_records_hold_the_old_partition`
  (the coverage rule), and `actual_gpu_cross_fade_lists_each_partition_it_draws`
  on the real visibility shader (NEW, OLD and unmarked entries, and no
  consultation of the old partition outside a fade). The terrain and foliage
  lists are twice the cell count so a cell can be listed once per partition.

## 3. The owner's review: it still cuts (2026-09-25)

The owner, on the merged build's recordings (`964ddf4`, 1440x900): "tree and
terrain fade in / fade out dithering is not done, there's a hard cut
transition". Made priority 1 in `CLAUDE.md`.

**Measured.** The recordings' game logs count the landings and the
`LOD_FADE skipped` lines:

| recording | landings | fade skipped |
| --- | ---: | ---: |
| cloud-hop | 31 | 15 |
| scenic clouds | 30 | 10 |
| storm | 29 | 10 |
| far-side | 19 | 6 |
| walk | 9 | 0 |

In flight a third to a half of the landings pop. The misses are small: "level
10: the old band reaches 659 m from the new anchor, the new records 658 m";
"level 11: ... 360 m ..., 355 m"; "level 9: ... 1292 m ..., 1278 m"; and on
the inner edge "level 9: the old band starts 509 m from the new anchor, the
new records 512 m".

**Why.** A set's records are laid round its own anchor out to its live band
plus `regen_m` and three tiles (`lay_band`), which covers the old partition
only while the anchor has moved less than that margin and the bands have not
changed size. In flight both fail: the player keeps moving while a set
builds, and `live_bands_m` resizes the bands with height. So coverage was
hoped for, not built, and `fade_covered` then refuses the whole landing on
the first level that falls short, by however little.

**The fix: lay the records to cover what they replace.** The request already
carries the partition it will replace (`Replaced`: the resident set's anchor
and complete radii) and bakes the walls' floors for both partitions from it
(`floor_rule`). The rings take the same input: level `k`'s ring is widened to
hold the old partition's level `k` round the new anchor,

```text
d          = great-circle distance between the old and new anchors
need_outer = old.complete[k] + d + 1 m
need_inner = max(old.complete[k + 1] - d - 1 m, 0)
ring       = [min(inner, need_inner), max(outer, need_outer)]
```

for every level the old partition drew. The metre is over `fade_covered`'s
half-metre rounding slack, so a ring laid this way passes the check by
construction. The new partition is unchanged: it hides and draws by the
complete radii, not the rings, and the extra records past its bands are
drawn only by the old partition during the fade, as the existing margin's
records already are. Capacity still wins: a ring truncated at
`FINE_CAPACITY` reports its true outer edge, the check fails for it, and
that landing pops and logs why, as before.

**Cost.** Only a landing whose anchor moved past the margin, or whose bands
resized, lays more cells, and only the difference. Walking, where the anchor
moves `regen_m`, is unchanged.

**How far short, and when** (the landings' request and landing lines, paired):
the misses run from 1 to 135 m, most 1 to 35 m, the large ones in the climb
after lift-off and on the descent. The player is 90 to 211 m from the anchor
by the time a set lands (100 to 148 m/s over a 0.7 to 1.4 s build, against a
`regen_m` of 55 to 110 m at 50 to 230 m up). Climbing fails the outer edge
(the finest band shrinks with height); descending fails the inner. 69 of the
70 skips across the recordings are a short ring; one is a level the new set
no longer lays at all (a climb past its height), which the ring cannot fix.
No set was truncated at capacity in any log.

**Trees snapped even on landings that faded.** A tree both partitions draw is
listed once, unmarked, and its fade was measured from the NEW anchor alone,
so at every landing its coverage jumped by the anchor's move over
`tree_fade_m`: about a quarter of the fade walking (41 m of 150), and more
than the whole fade in flight (90 to 210 m), where the tree line ahead went
from dithered to solid in one frame. **Fix:** such a tree's coverage is the
old partition's eased to the new one's by the fade's progress. Outside a fade
the old partition is the new one, so nothing else changes.

**Still cutting after this, each its own change:**
- the voxel column tier (90 m round the anchor) is rebuilt at every landing
  and drawn for the new partition only, so caves, mouths and voxel light in a
  crescent 50 to 90 m ahead of a walker change in one frame;
- the water sheet is drawn for the new partition only, so a coastline at a
  band edge snaps;
- the finest level's capacity (65,536 cells, about 380 m round the anchor)
  can truncate a ring widened by a large miss, low and fast; that landing
  still pops, and says so in the log;
- the first set after an empty one (descending from above the coarsest band)
  does not fade, like the startup set.

**Not changed here:** the first landing after startup (it replaces the base
alone, which has no fine partition to fade from) still switches at once, and
`lod_fade_s` stays 0.6 s until the recordings show whether a fade that runs
reads as a fade.


## 4. Fade every landing: only the ring the records lack pops

After section 3, the landings that still skip their fade are those where the
finest levels' capacity cut the widened ring (cloud-hop 6 of 28, scenic
clouds 5 of 29, every one but one a capacity cut). `fade_covered` refuses the
WHOLE landing for one short level, so four levels pop for one.

**The fallback: fade every landing, and draw whole what the old partition
cannot.** The uniform carries the new records' ring per fine level, as
cosines round the new anchor, each shrunk by one of that level's tiles
(`records_in`, `records_out`; a level not laid holds nothing). The visibility
pass, while a fade runs, finds the old partition's level at a cell's centre
(the finest old band that covers it) and whether that level's ring holds it.
Where it does not, the old partition's cells there are not among the records
and would be holes in the old half of the dither; so a cell the NEW partition
draws there is listed unmarked (drawn whole), as its trees are. That ring
switches at the landing, as the whole landing did before; everything else
cross-fades. The shrink by a tile errs toward drawing a new cell whole where
an old one might still be held, which costs an overlap on a ring's edge for
the fade's 0.6 s, never a hole.

`fade_covered` stays as the diagnostic: it names each level that falls short
(`LOD_FADE partial: level N ... that ring switches at once`), and the fade
runs.

**Tests.** The coverage rule's test says which levels fall short rather than
that the landing is refused; the GPU cross-fade test gains a case where the
records' ring excludes part of the old partition, and the new cells there are
listed unmarked while the rest stay NEW or OLD.
