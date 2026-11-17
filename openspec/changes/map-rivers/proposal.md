# Proposal: the map's rivers hold water

## Why

The owner, 2026-11-17, on the M key's map near Holbrook (about 29.57 N,
1.37 E): "i see you carved rivers in here, but we need to fill it with water
on this map too!" The rivers show on the map as thin dark grooves across the
meadow and the sand, drawn as land. Only the sea and the coastal inlets that
dip under the sea's level are blue.

## What changes

- Each map texel carries how much of its pixel a river's water covers, read
  with the game's own river test (`pbd_core::rivers::floods`, the answer the
  column's river water and the far records' river beds already give), and
  none in a town's footprint, where the column puts no river either.
- The map paints that share of the pixel in the map's own water colour
  (`world_map::sea_colour` at a river's depth) over the shaded ground, so a
  river is as wide on the map as its water is in the world, and a stream
  narrower than a pixel is a faint continuous line rather than a dotted one.
- The sea is painted exactly as before.
- The base map's cache carries the river in two spare bits and its name moves
  from `p1` to `p2`, so a cache made before this is not read (it is rebuilt
  once from the generator, as any new cache is). The map is derived; nothing
  in a save changes.

## Impact

- `crates/pbd-core/src/map.rs` (the texel, the river's cover),
  `crates/pbd-core/src/rivers.rs` (a public "is a river near" read for the
  cover's pre-check), `crates/pbd-app/src/world_map.rs` (colour, cache).
- No generator version change: terrain, rivers and columns are untouched.
- Frame cost: none during play; the map's base build gains the river reads
  near rivers only. Not measured on a GPU (cloud session, lavapipe).
