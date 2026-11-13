# Design: where the camera is

## The numbers

From the active `Camera3d`'s translation less `PlanetRenderFrame.center`
(subtracted in f64, CLAUDE.md), a body-local point `p`:

- `--at` latitude and longitude: `geo::lat_lon(p)` in degrees, to three
  places. `--at <lat> <lon>` with these puts the walker back on the spot.
- The map's reading: `geo::compass_lat_lon(p)`, printed as `20.000S 40.000E`.
- Elevation: `|p| - PLANET_RADIUS`, metres above sea level.
- Above ground: `weather::height_above_ground(p)`, the eye over the generated
  surface (sea level where that is below the sea). It does not see digs,
  buildings or cave roofs: it is the number the rain's altitude gate already
  uses, and it is labelled "above ground".

## Where it shows

- HUD: a second line under the near-field line, refreshed when it changes by
  a displayed digit: `at 20.000 40.000 (map 20.000S 40.000E) | 1203.6 m
  elevation | 1.6 m above ground`.
- Log: `PLACE spawn ...` once the walker or flight camera first exists,
  `PLACE ready ...` with the `READY` line, and `PLACE capture <png> ...` when a
  capture is saved. The time of day and the weather are on their own lines
  already (`sun from the spawn at --time`, `Weather here`).
