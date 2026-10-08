# Tasks

## 1. Measure

- [x] 1.1 Probe facing-north handedness, the view from space, and where the
      sun rises at four places (numbers in the proposal). The atmosphere's
      `prograde` and Coriolis sign read off `atmosphere::step`.

## 2. The compass in `geo`

- [ ] 2.1 `COMPASS_NORTH`, `compass_north_east`, `compass_heading`, `bearing`,
      `compass_lat_lon`; the frame functions' docs say their north is +Y.
- [ ] 2.2 Tests: facing compass north, east is screen-right everywhere; the
      sun rises toward compass east; from space with compass north up, east
      is on the right; `bearing` to a place a little east is 90 degrees.
- [ ] 2.3 `Clock::season` in compass words; its test.

## 3. The bar

- [ ] 3.1 `pbd_app::compass`: the view heading (decision 4) and `marks`
      (decision 3, 5), with tests: turning right reads N, NE, E, ...; pitching
      to the feet keeps the heading; a site at a bearing lands at its offset;
      the nearest within 7 degrees is named.
- [ ] 3.2 The desktop bar: nodes, letters, ticks, diamonds, name line; hidden
      off `Playing`; faded with altitude.
- [ ] 3.3 A test that no player-facing desktop module calls a frame function
      for what it shows.

## 4. The map

- [ ] 4.1 `MapView` runs v upward; the base, the layer copies and the tiles
      are placed and drawn flipped; `map_live.wgsl` flips its v; a drag pans
      the right way.
- [ ] 4.2 The cursor readout gives compass latitude; the arrow turns by the
      compass heading; the map's tests updated and a test that a place a
      little compass-north is drawn above and a little east to the right.
- [ ] 4.3 The published world-map mockup redrawn compass-north up (a page,
      not the game; follow-up).

## 5. Show it

- [ ] 5.1 Captures: the bar at sunrise facing east (the sun under E); the bar
      facing a town with its name; the map beside the planet seen from orbit,
      before and after the flip. In `docs/screenshots/compass-bar/`.
- [ ] 5.2 Video for the batch (CLAUDE.md: screenshots stand in for now).
