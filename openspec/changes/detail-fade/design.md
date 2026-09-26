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
