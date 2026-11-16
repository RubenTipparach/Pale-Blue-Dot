# Tasks: the rain on the map is the rain you stand in

## 1. Measure

- [x] 1.1 `atmosphere::tests::map_rain_against_local`: where the player stands
  in rain, what the map's clouds layer and RAIN overlay draw there (design,
  "Measured").

## 2. One field

- [x] 2.1 `pbd_core::weather::shown_precipitation`: the rain seen, negative
  for snow, nothing under `RAIN_SEEN_MIN`; `cloud_cell` reads it.
- [x] 2.2 `WeatherMaps::rain_seen`, made with the other maps from the same
  state; the map's `cloud_bytes` draws it.
- [x] 2.3 The RAIN overlay reads it, in percent of a full rain (-100..100).
- [x] 2.4 Test: at sampled places and times, the map's layer, the overlay and
  `Weather.rain` agree within a byte.

## 3. Show it

- [x] 3.1 Before and after captures of the map and the world at the same
  place and moment, `docs/screenshots/rain-matches-map/`.
- [ ] 3.2 Video for the batch (CLAUDE.md: screenshots stand in for now).
