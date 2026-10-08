# Proposal: a compass bar, and north on the right pole

## Why

The owner, 2026-10-08: "I think I need an ingame compass or something", and
then: "not sure if the map's N/S/E/W directions are the same as the planet,
we'll have to see what it looks like when we add a compass bar like from
skyrim".

They are not the same, and the trouble is not the map alone. It is which pole
the game calls north. Measured with a throwaway probe in `pbd-core` against
`geo::north_east` and `daylight::Clock`, at four places:

| Place (lat, lon as `geo` reads them) | Facing `geo` north: screen-right . east | From space, `geo` north up: screen-right . east | The sun rises toward (east, north) | The sun's motion . east, per 10 s |
| --- | ---: | ---: | --- | ---: |
| 0, 0 | -1.000 | -1.000 | (+0.92, +0.40) | -1.77e-2 |
| 30, 45 | -1.000 | -1.000 | (+0.89, +0.46) | -1.77e-2 |
| -40, -120 | -1.000 | -1.000 | (+0.85, +0.52) | -1.77e-2 |
| 60, 170 | -1.000 | -1.000 | (+0.61, +0.80) | -1.77e-2 |

The sky is right: the sun rises on the side `geo` calls east and crosses
toward west. The atmosphere agrees with it. Its "Earth's east"
(`atmosphere::step::prograde`, `c x Y`) is `geo`'s east, and its Coriolis
term turns the wind to the left in the +Y hemisphere. So the planet turns about
-Y, and by the rule every compass on Earth follows, -Y is the north pole.

`geo` calls +Y north, so its east, north and up form a left-handed set. Two
things follow from that, and a player sees both:

- **Facing north, east is on your left**, so the sun rises on your left. On
  Earth, and in Skyrim, it rises on your right.
- **The map is a mirror image of the planet.** It is drawn with `geo` north
  up and east to the right. From space, with that north up, east is on the
  left. Every coastline on the map is the mirror of the same coast seen from
  the ship.

A compass bar built on `geo` as it stands would make this obvious at once: it
would read N, W, S, E as the player turns right.

## What

1. **North is the -Y pole.** That is the pole the planet turns
   counter-clockwise about, and the sun rises on the right of it. East stays
   the sunrise side. What changes is what the player is told: compass
   latitude is the frame's latitude negated, and longitude is unchanged.
   `pbd_core::geo` gains the compass functions (`compass_north_east`,
   `compass_heading`, `bearing`, `compass_lat_lon`). Every word, letter and
   arrow a player reads goes through them.
2. **A compass bar** across the top of the screen, as in Skyrim:
   - N, E, S and W, with the four intercardinals and a tick every 15 degrees,
     scrolling as the view turns;
   - a marker for each town or site within 3 km, at its bearing;
   - the name and distance of the place nearest the middle.
   It works walking, flying and aboard a craft, and it fades out high above
   the ground.
3. **The map is drawn compass-north up.** Its rasters are kept as built and
   drawn flipped top to bottom. That un-mirrors the map, so the map and the
   view from orbit show the same coast the same way round. The cursor readout
   gives compass latitude, and the player's arrow turns with the compass
   heading.
4. **The season reads in compass words.** The year opens on the summer of
   the +Y hemisphere, which a compass now calls south.

Nothing a world is made of moves:
- terrain, biomes, sites, towns and their layouts;
- the climate and saved records.

`geo::lat_lon`, `geo::north_east` and `geo::project` keep their frame meaning:
- towns, sites and the site placer orient by them;
- the rasters and the map's cache are keyed by them.

The harness flags (`--at`, `--map-at`) and the log lines that print them keep
frame latitude, so every capture command already written still lands where it
did.

## Impact

- `pbd-core`:
  - `geo` gains compass functions and the tests that pin them against the
    sun and the screen;
  - `daylight::Clock::season` names the compass hemispheres.
- `pbd-app`:
  - a `compass` module (the bar's pure layout and its heading);
  - the desktop bar itself;
  - `map_screen` and `map_image.wgsl`/`map_live.wgsl` draw compass-north up.
- The published world-map mockup is still mirrored. Flipping it is a
  follow-up task here; it is a page, not the game.
- Frame cost: a few dozen UI nodes repositioned per frame. It is not measured
  in this cloud session (no GPU); the owner runs `perf_suite.py`.
