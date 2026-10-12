# Proposal: trees and ground detail fade in and out instead of popping

## Why

The owner: "we should do something about detail popin, some sort of dither
fade in or fade out would be nice, for trees and LOD blocks"; then, after a
flight, "still getting tree pop outs". Queued as item 8 in `CLAUDE.md`,
brought forward: "cross fade lod and trees next".

## What was found

1. **Trees are cut at a hard line.** The visibility pass
   (`planet_visibility.wgsl` `has_nearby_foliage`) submits a tree only when its
   cell's centre is within `FOLIAGE_DRAW_DISTANCE` (1200 m, the finest band's
   reach) of the camera, and the whole foliage draw is switched off above
   `FOLIAGE_DRAW_CUTOFF_ALTITUDE` (1800 m). A tree crossing 1200 m appears or
   vanishes whole, in one frame. The altitude cut adds no pop of its own: at
   1800 m up every tree is already more than 1200 m away (the highest summit
   is under 600 m).
2. **Ground detail switches level at a moving line.** Each fine band is the
   set of cells within `band_cos` of an anchor, `params.lod.xyz`, which is the
   fine set's anchor and jumps when a new set lands (every `regen_m` of travel).
   Every cell between an old band edge and the new one changes level in that
   frame, so a ring of blocks at each band edge pops to a new shape. The
   partition also decides geometry in the vertex shader (the wall a cell drops
   to a finer neighbour's floor, the cut wall of a split midpoint cell), so a
   fragment cannot simply choose between two partitions: it would be drawing
   one partition's surfaces with the other's walls.

## What changes

1. **Trees dither out over their last `tree_fade_m`** (150 m, `scatter.ron`)
   before the 1200 m line, with an ordered (Bayer 4 x 4) screen-door mask: a
   tree thins out pixel by pixel and is fully gone before the visibility pass
   drops it, so crossing the line changes nothing on screen. It needs no
   blending or sorting and suits the point-sampled pixel look.
2. **A landing cross-fades the ground detail** (a second stage, designed
   here and built after the trees): for `lod_fade_s` after a new set lands, the
   previous partition is drawn as well, from the previous anchor and bands,
   and each pixel shows one partition or the other by the same ordered mask
   against the fade's progress. Cells that are the same under both partitions
   are drawn once.

## Out of scope

The clutter already shrinks out over `clutter_fade_m`; the column tier and the
water sheet's own band edges are left as they are unless the cross-fade shows
them popping.
