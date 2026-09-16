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

## Open decision: the radius

Scaling the avatar instead is no longer an option - the hex size is spec now, so
the world moves, not the player. What is still open is which radius, because
holding the tile at 2.833 m locks the radius to the level:

```text
R = 300 m * 2^(L - 7)
```

| level | radius | diameter | cells | topology at 128 B | eager build |
| ---: | ---: | ---: | ---: | ---: | --- |
| 8 | 600 m | 1.2 km | 655,362 | 80 MiB | today's exact budget |
| 9 | 1,200 m | 2.4 km | 2,621,442 | 320 MiB | comfortable |
| 10 | 2,400 m | 4.8 km | 10,485,762 | 1.3 GiB | tight |
| 11 | 4,800 m | 9.6 km | 41,943,042 | 5.0 GiB | no |

The authored catalog wants 4 km radii and 5-12 km diameters, which needs level
11 and is not eagerly buildable. So the whole-globe preview survives only if the
planet shrinks, and the catalog's sizes wait for `voxel-engine-foundation`.

One lever before choosing: 92 of the 128 bytes per cell are pure topology - the
direction and the six corner rays - identical for every body at a given level,
and each corner ray is shared by three cells. Storing corners once and indexing
them gets a cell to roughly 50 bytes, which buys about one level: level 10 at
around 520 MiB, so 4.8 km diameters come into range. It does not reach a 4 km
radius.

## Non-goals

- Binding `water.wgsl`. It needs scene colour and depth inputs and its own
  pipeline; `docs/shader-port.md` already carries it as outstanding.
- Interpolating skylight per corner. It is the right fix for the flat-tile
  lighting and it is upload-side work, not a shader edit.
- Any look change landing without a before-and-after capture the owner has seen.
  Green tests do not settle a look.
