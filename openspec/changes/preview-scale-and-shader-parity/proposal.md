# Proposal: bring the preview's scale and its shader back toward Tenebris

## Why

Two findings, measured rather than felt, both in
`docs/tenebris-comparison.md`.

**The world is 6.7x too big for its avatar, and the hex size is now a fixed
spec.** The owner's decision: a cell is the same size on every body, because a
cell is a unit of material and a unit that changes size between worlds is not a
unit. `tenebris-rs` is the definitive spec and its **main Tenebris planet**
(radius 300 m, level 7) is the gold standard - **2.833 m tile width, 1.000 m
cell height**. Its other bodies are prototype stage and Sequoia is still under
development; neither is a reference.

This preview holds neither number. Measured off the real `dual_sphere(8)`:
tiles are 18.883 m against the standard's 2.833 m, and the elevation step is
6 m against 1 m. The avatar is Tenebris's at 1:1 - same 1.6 m eye, same 8 and
14 m/s, same 12 m/s jump - so read in
eye heights, a hexagon is 1.8 people wide there and 11.8 people wide here, and a
terrain step goes from something you walk up (0.63 of eye height) to a wall over
twice your height (3.75). The same jump clears 2.9 blocks in Tenebris and 1.3
steps here, so terrain Tenebris lets you hop over is a cliff.

**The faithful shader is the one that is not running.** `hex_terrain.wgsl` is a
term-for-term Tenebris port with every knob a uniform, and it is bound to no
pipeline. The live `planet_surface.wgsl` reimplements a subset with every knob a
literal, and it drops the night-side rim floor, the ambient floor, torch light,
underwater absorption and the Bayer cutout. Two of those show in a still frame:
the night limb falls to black, and lighting is flat across a whole 18.9 m tile.

## What changes

Two things that are cheap and do not touch topology or uploads, and one that
does:

1. Restore the night-side rim floor in the live shader, matching Tenebris's
   configured `distant_rim_floor` and our own unbound port.
2. Lift the live shader's hard-coded rim, fog, terminator and light-tint
   numbers into the params uniform it already fills, so a second planet becomes
   possible without editing WGSL.
3. Decide the preview body's scale deliberately. Tile width is
   `1.209 * R / 2^L`, so at level 8 a 600 m radius lands Tenebris's 2.83 m
   tiles and 250 m lands this project's own 1.18 m target.

## Decided: radius 4,800 m at level 11, which makes LOD load-bearing

The owner's call, in two steps: keep a ~4 km-class body rather than shrinking to
fit a uniform subdivision level, then take **4,800 m** once the ladder showed
that 4,000 m cannot hold the standard at any level. The working table below is
kept because it is the reasoning, not just the answer.

The planet stays the size it is, near enough. That settles the trilemma by dropping the third
corner - the eagerly-built whole globe - and it makes `hexagon-lod` a
prerequisite rather than an optimisation. A uniform level cannot serve a 4 km
body at the gold standard: level 11 is 42 million cells and 5 GiB of topology.

### One correction that follows: 4,000 m is not on the ladder

Holding the tile at 2.833 m locks radius to level as `R = 300 m * 2^(L - 7)`,
and 4,000 m sits between two rungs. Measured:

| radius | level | tile width | verdict |
| ---: | ---: | ---: | --- |
| 4,000 m | 10 | 4.721 m | too coarse |
| 4,000 m | 11 | **2.361 m** | 17% under gold, outside the 2.595 - 3.101 m spread |
| 2,400 m | 10 | **2.833 m** | exact |
| **4,800 m** | **11** | **2.833 m** | **exact** |

**4,800 m at level 11 is the decision.** It is *larger* than today's 4,000 m, so
it keeps the intent rather than bending it; it lands the gold
standard to the digit; and its 9.6 km diameter sits inside the authored
catalog's own 5 - 12 km range, which 4,000 m's 8 km also does. Nothing is given
up by moving up 20%.

Level 11 is the level underfoot, not the resident level. What is resident is a
cap around the player sized by render distance, and the rest of the sphere is
coarser tiers costing `4/3` of one level in total. That is the whole of what
`hexagon-lod` buys.

### What still moves with the radius

Unchanged from before: the 4,800 m atmosphere shell, the 4,600 m cloud layer,
the 2,300 m foliage range, the 3,200 m draw-budget switch and the terrain
amplitude all scale with the body, and `ELEVATION_STEP` goes from 6 m to the
standard's 1 m. At a 4,800 m radius the scale factor is 1.2 rather than the 0.15
a 600 m body would have needed, so the relief stays broadly as authored.

## Non-goals

- Binding `water.wgsl`. It needs scene colour and depth inputs and its own
  pipeline; `docs/shader-port.md` already carries it as outstanding.
- Interpolating skylight per corner. It is the right fix for the flat-tile
  lighting and it is upload-side work, not a shader edit.
- Any look change landing without a before-and-after capture the owner has seen.
  Green tests do not settle a look.
