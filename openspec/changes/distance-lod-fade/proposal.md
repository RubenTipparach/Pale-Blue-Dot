# Proposal: trees and terrain cross-fade by distance, visible in flight, never popping

## Why

The owner (2026-09-25/26): "tree and terrain fade should be visible when
flying and shouldn't pop in", "I don't see any cross fade currently with
trees", "we should see trees fade in flying mode", with the reference
<https://github.com/keijiro/CrossFadingLod>: Unity's LOD Group in "Cross Fade"
mode with a fade transition width of 0.5, both LODs drawn with complementary
screen-space dither masks (`UnityApplyDitherCrossFade`), the fade driven by
the object's size on screen, so it is continuous with camera motion, and the
last LOD fading to nothing. Priority 1 in `CLAUDE.md`.

## What was found (read off the code)

1. **The terrain fades on a clock, not with distance.** The only terrain fade
   is the 0.6 s after a landing (`planet.rs`), centred on the fine set's
   anchor (`planet_lod.rs` `LodParams::of`), which jumps 40-210 m per landing.
   Between landings nothing changes: a walker sees a short shimmer in a thin
   ring, a flyer a dissolve that reads as a pop.
2. **Trees fade where no one looks.** Their only distance fade is at the
   level-9 band's edge (`tree_shown`), about 1.2 km out on the ground: below a
   walker's 124 m horizon; from 600 m up it changed 88 pixels of a frame.
3. **Trees rearrange at the 300 m and 600 m edges.** A coarse cell rolls its
   own tree from its own id at the density times 4^n
   (`planet_visibility.wgsl` `has_nearby_foliage`), at positions unrelated to
   the fine trees, each twice as wide (its crown shrunk from the coarse
   cell's own corners): forests thicken and reshuffle at every edge, and only
   at landings.
4. **Past level 9 there is no tree at all**: no lower LOD to fade into.

## What changes

1. **One partition per pixel (terrain).** Each fine band's edge becomes a
   ring of width `lod_fade_width` of the band (0.15 to start; Unity's 0.5
   width is about 0.33 on this band ladder), round a centre that follows the
   live camera within what the records hold. A pixel's Bayer value picks a
   radius inside each ring; the pixel keeps a fragment when the partition at
   that radius draws its cell. Each of the 16 Bayer classes sees one complete
   partition, so there is no double draw and no z-fight; outside the rings
   the result is today's partition. Moving through a ring is continuous. The
   landing's timed cross-fade is retired for terrain (kept as a fallback when
   the centre must jump).
2. **Trees nest across levels and fade with distance.** A coarse cell's tree
   is the fine tree standing at its centre (a coarse cell's centre is a cell
   centre at every finer level): same point, same id, same tree. Across a
   ring the survivor's width blends and its three siblings dither out, so
   cover is kept. Trees go on level 8 too, and fade to nothing across level
   8's outer ring (about 2.4 km on the ground), as Unity's last LOD does.

## Out of scope

The voxel column tier and the water sheet still switch at a landing; a canopy
tint on the base level beyond the last tree ring.
