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

## 4. Half the rain, none on the desert (owner, 2026-11-16)

- [x] 4.1 `examples/rain_budget.rs`: share raining, mean rate, rain seen and
  cover over 30 days, planet, desert and tundra.
- [x] 4.2 `evaporation` 3.0e-4 -> 0.75e-4, `rain_threshold_kg` 4 -> 8:
  raining 0.48x, rate 0.47x, rain seen 0.51x.
- [x] 4.3 Virga: `virga_dry_wetness`, `virga_wet_wetness`, `rain_lands`;
  test `no_rain_falls_on_the_desert` (fails without the rule).
- [x] 4.4 Remake the shipped settled climates on the new settings.
- [x] 4.5 Map before and after (RAIN overlay), `docs/screenshots/rain-matches-map/`.

## 5. Snow in freezing air (owner, 2026-11-16)

- [x] 5.1 `Sample::air_c` decides snow; test `what_falls_in_freezing_air_is_snow`.
- [x] 5.2 The puddle rings, sheen and rain fog read `Weather::liquid`.
- [x] 5.3 `snow_switch_s`: the water eases out where rain turns to snow;
  test `rain_gives_way_to_snow_over_the_switch_time`.
- [ ] 5.4 Tundra in summer still rains (design 10): a question for the owner
  with screenshots, if "cold regions" means the white ground.
