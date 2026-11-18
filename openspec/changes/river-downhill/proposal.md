## Why

The owner, 2026-11-18: "river appears to go up hill a bit. the flow is
correct, but it kind of hinders my logging idea for logging at the top of
the river and floating logs down the river ... the river should always go
downhill or stay on elevation".

**Measured** down the owner's river (`--at 19.25 -6.75`, 1,593 m, the water's
top every 2 m down its line; instrument
`river_walk::uphill_probe::where_the_river_runs_uphill`): **49 places where
the water rises going downstream, 54 m in all, the worst 6 m.** Every one is
inside a reach whose own level does not change (the drainage map's levels
never rise; `every_river_cell_drains_to_the_sea` holds that). The rises come
from the bank clamp in `rivers::top_here`. A point's water is held a layer
under the lowest ground of the ring round it (its banks), so where a bank
dips the water drops, and downstream of the dip it climbs back to the
reach's level.

## What changes

- **Generator version 11: the water's top is a running minimum down the
  river.** A river's water at any point is no higher than it was anywhere
  upstream: the lowest bank clamp met along the river's line from its spring,
  carried down every reach and across every confluence. Each cross-section
  is flat, and the water only steps down or stays level. The bed is carved
  under it as before, so the ground in a river's valley changes.
- **A new version for new worlds** (CLAUDE.md, "Saved games survive every
  change"): versions 8 to 10 keep their rivers cell for cell; a world names
  its version and opens with it.
- **Its climate** is settled for version 11 and shipped beside the others
  (`assets/climate/settled-g11-*`), as every version's is.

## Impact

- `pbd_core::rivers`: `RiverSettings::downhill`, and a per-river-cell table of
  ceilings along its reach, made with the map (in parallel, about 240,000
  ground samples) and remade when the map is read from its cache file.
- `planet_gen`: `TENEBRIS_V11`; `GENERATOR_VERSION` 11.
- New worlds' rivers are lower where a bank dipped upstream, never higher.
  Off the rivers' valleys version 11 is version 10.
