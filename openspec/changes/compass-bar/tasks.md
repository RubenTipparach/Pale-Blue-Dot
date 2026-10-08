# Tasks

## 1. Measure

- [x] 1.1 Probe facing-north handedness, the view from space, and where the
      sun rises at four places (numbers in the proposal). The atmosphere's
      `prograde` and Coriolis sign read off `atmosphere::step`.

## 2. The compass in `geo`

- [x] 2.1 `COMPASS_NORTH`, `compass_north_east`, `compass_heading`, `bearing`,
      `compass_lat_lon`; the frame functions' docs say their north is +Y.
- [x] 2.2 Tests: facing compass north, east is screen-right everywhere; the
      sun rises toward compass east; from space with compass north up, east
      is on the right; `bearing` to a place a little east is 90 degrees.
- [x] 2.3 `Clock::season` in compass words; its test.

## 3. The bar

- [x] 3.1 `pbd_app::compass`: the view heading (decision 4) and `marks`
      (decision 3, 5), with tests: turning right reads N, NE, E, ...; pitching
      to the feet keeps the heading; a site at a bearing lands at its offset;
      the nearest within 7 degrees is named.
- [x] 3.2 The desktop bar: nodes, letters, ticks, diamonds, name line; hidden
      off `Playing`; faded with altitude.
- [x] 3.3 A test that no player-facing desktop module calls a frame function
      for what it shows.

## 4. The map

- [x] 4.1 `MapView` runs v upward; the base, the layer copies and the tiles
      are placed and drawn flipped; `map_live.wgsl` flips its v; a drag pans
      the right way.
- [x] 4.2 The cursor readout gives compass latitude; the arrow turns by the
      compass heading; the map's tests updated and a test that a place a
      little compass-north is drawn above and a little east to the right.
- [ ] 4.4 The eight-wind calibration (decision 7a): the test facing each wind
      at six places through the real systems; `geo::map_heading` and the
      arrow turned by it; the test passing within a degree; map shots facing
      N, E and NE.
- [ ] 4.3 The published world-map mockup redrawn compass-north up (a page,
      not the game; follow-up).

## 5. Show it

- [x] 5.1 Captures, in `docs/screenshots/compass-bar/` (`tools/capture_compass.sh`):
      - the bar at sunrise facing east, with the sun just right of E at
        bearing 113°;
      - the bar facing north at noon, naming Tahal 2.2 km;
      - the map beside the planet seen from orbit, before and after the
        flip. The before map is the 2026-10-01 shot taken with the same
        flags, because an older build reads this tree's assets and cannot
        run here.
- [ ] 5.2 Video for the batch (CLAUDE.md: screenshots stand in for now).

The requirements stay in this change's delta until the owner's gate. Each is
pinned by a passing test:
- `geo::tests::the_compass_has_east_on_the_right_and_the_sun_rising_there`
  and `compass_latitude_and_bearing`;
- `daylight::tests::the_season_follows_the_sun_north_and_south`;
- `pbd-app` `compass::tests` (four of them);
- the desktop's `the_screen_reads_the_compass_not_the_frame` and
  `map_screen::tests::north_is_up_and_east_is_right_on_the_map`.

`--yaw 0` faces compass west, because the walker's default heading is
`Y x up`, which is frame west. The comment beside it in `desktop.rs` said
"facing east", which was wrong under either name. The flag's meaning is
unchanged, so recorded capture commands keep working.
