# Design: the map's rivers hold water

## Cause

The map's raster has one water rule, the sea's:

- `pbd_core::map::base_texel` (`crates/pbd-core/src/map.rs:47`) sets
  `sea: altitude_m < cfg.sea_level_m`, and the cache's `unpack`
  (`crates/pbd-app/src/world_map.rs:310`) rebuilds it the same way.
- `world_map::colour_block` (`crates/pbd-app/src/world_map.rs:229`) paints a
  texel blue only `if t.sea`; every other texel is its top block's tile
  colour times the hillshade (`:245`).

A river (generator version 8, `ocean-environments` design 13) is water
standing OVER the sea's level: its reach's level is whole metres above the
sea, and `rivers::carve` cuts its bed two layers under that water. So the
bed's altitude is still above the sea's level, `sea` is false, and the map
paints the bed as the meadow or sand it is, with the hillshade picking out the
carved valley as a dark groove. That is the owner's picture. The water itself
is never in the texel: the column adds it afterwards
(`column::generate_solid` through `rivers::water`), and the map never asks.
The inlets that are blue are the river mouths, where the carve goes under the
sea's level.

## The river test

The game's river water is `rivers::water(cfg, d)`; `rivers::floods(cfg, d)`
is the same answer without building the list, and it is what the far records
(`planet_terrain`) and the eye-under-water test (`planet_water`) already use.
The column skips the river in a town's footprint
(`settlement::ground::top(..).is_some()`), and so does the map. So a map point
is river water exactly when the column there holds river water.

## Width

A river's water is 4.4 to 18 m across (`half_width_m` 2.2 to 9 m, wider down
its catchment, plus the bank water where the valley's ground is under the
reach's level). The map's pixels are 11.3 m (the cached base), 5.7 m and
2.83 m at the equator. A point test per pixel would draw a stream narrower
than the base's pixel as a dotted line, or not at all. So each texel takes
the share of its pixel the water covers: the river test at a 3 x 3 grid of
points across the pixel (its east-west width narrowed by the latitude, as the
raster's pixels are), quantised to thirds: none, a third, two thirds, all.
The grid is only read where `rivers::near` says a river's banks reach within
the pixel's half diagonal, so the rest of the planet pays one lattice read.

Thirds because the cache has two spare bits: the material code's top bit
(codes stop at 79) and the place nibble's top bit (seven places use three
bits). The fresh build quantises the same way, so a cached base and a fresh
one paint alike.

## Colour

A river texel's colour is the shaded ground mixed toward
`sea_colour(RIVER_DEPTH_M)` by its share, `RIVER_DEPTH_M` being 2 m, the
depth `rivers::carve` cuts the bed under the water. That is the sea's own
shallows ramp, so a river reads as the same water as the coast it runs to.
The sea's texels are painted exactly as before; the river test is false at
the sea, and a sea texel's colour does not read the river field.

## The cache

`p1` becomes `p2` in `cache_name`. An old cache has no river bits, and read
as `p2` it would draw every river dry again, so it is not read; the base is
rebuilt once from the generator. Saved games are untouched: the cache is
derived, and the world's terrain, its generator version and its records do
not change.

## Not in this change

- The layers (biomes, the sea's places) are unchanged: a river is not a
  biome.
- No map shader changes; `map_image.wgsl` greys the base under an overlay as
  before, rivers included.
