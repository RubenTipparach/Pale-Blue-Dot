# Proposal: a landing dissolves from the records it replaces

## Why

The owner (2026-11-13): "several visual popins: clouds, terrain, and baked
lighting" and "we need some way to better transition those". Priority 1 in
`CLAUDE.md` is "No pop-in": trees and terrain blocks dither-fade in and out;
they never appear or vanish in one frame. This change is the terrain's part
(the ground's hex columns and blocks, and the trees drawn with them); the
clouds and the baked lighting are other changes.

## What was found (measured, design section 1)

A landing (a new fine set swapped in) dissolves from the old partition of the
ground into detail levels to the new one over `lod_fade_s` (`detail-fade`).
The old half of that dissolve is drawn from the NEW set's records, because the
GPU holds one copy of the fine records and the landing overwrites it. Where
the new records do not hold a cell the old partition drew, the old half would
be a hole, so the visibility pass draws the new partition there whole
(`detail-fade` section 4, `dissolve-coverage`): that ring switches in one
frame, by design, and logs `LOD_FADE partial`.

On this base (`claude/perf-tuning` at `40040db`), fixed-step captures:

| Run | Landings | `LOD_FADE partial` |
| --- | ---: | ---: |
| scenic flight (`--route scenic --time 10`), two runs | 33, 33 | 6, 6 |
| 1 km walk (`--walk-distance 1000`) | 38 | 1 |

The flight's six are the climb past 300 m, where the new set lays no level 11
at all (the whole disc of finest ground under the camera switches to level 10
at once), and five landings in the descent, where the records widened to
cover the old partition stop 1 to 30 m short of it: the widening is capped at
each level's margin on purpose (`distance-lod-fade`, "A runaway fit"). The
walk's one is its first set after the spawn, 443 m from the startup set.
The log line is a lower bound: it measures from the old anchor and checks one
level, while the shader's own test (`old_held`) works round the old fade
centre and checks both levels across a band's ring.

Five more ways a landing switches ground in one frame, none of them logged:

1. a set landing while the previous dissolve still runs replaces it, and what
   was left of the old partition switches at once;
2. a new version that does not move the partition (an edit, the dusk relight)
   cancels a running dissolve;
3. the first set after an empty one (descending from above 2.5 km) does not
   dissolve at all;
4. a tree both partitions draw is listed once and drawn with the new
   partition's band fades, so its dither density jumps where the two
   partitions put it at different depths in a band's ring;
5. the dissolve's clock is the wall clock, so in a fixed-step capture it does
   not run over the time the player would see it run over.

## What changes

1. **The old set stays for its dissolve.** The cell buffer holds two sets of
   the four fine regions. A landing writes the new set into the half the
   dissolve is not drawing from; the old half keeps the records the old
   partition was drawn from until the dissolve ends. The old partition draws
   only from the old half, the new one only from the new half, both from the
   base. Everything the old partition drew the frame before a landing is drawn
   in its half of the dissolve, whatever the new set holds: no coverage
   condition, no ring drawn whole. `old_held`, the records' rings in the
   uniform and `fade_covered` retire.
2. **A dissolve is never cut short.** A set that finishes building while the
   last landing's dissolve runs waits (at most `lod_fade_s`) and is swapped in
   when it ends; an edit or a relight writes over the current half in place
   and leaves a running dissolve running.
3. **Every landing dissolves** but the set that ends startup (behind the
   loading screen) and the first set on a new generator; the first set after
   an empty one dissolves out of the base.
4. **Trees dissolve too** wherever the two partitions draw their cell
   differently.
5. **The dissolve runs on the game clock**, as the lamp bake's already does.

No generation, terrain height, saved record or digest changes. The records
a set lays are the same bytes as before.

## Cost

About 63 MB more GPU memory (50 MB of records, 13 MB of per-slot lists), a
visibility dispatch over 688k slots instead of 426k (the idle half returns on
its first test), and for `lod_fade_s` after a landing the cells the two
partitions draw differently are drawn twice, as they already are. Frame time
is not measured here (no perf suite in this session); the owner runs it.

## Out of scope

The voxel column tier (90 m round the anchor, rebuilt at every landing) and
the sea sheet still switch at a landing; each is measured in the design and
left to its own change.
