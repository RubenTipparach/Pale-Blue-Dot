# sun-shadows

Screenshots for `openspec/changes/sun-shadows`: the sun's cascaded shadows,
and the rooms' own light (design decision 7).

Every shot is an in-game capture by `tools/capture_sun_shadows.sh`:
- taken in a cloud container on lavapipe, with no GPU;
- at 1440 x 900, from the fast build, under `xvfb-run`;
- each in a new memory-only world, 150 frames in;
- `--rain 0 --weather-at 7200 --open-doors`, at the hour its name gives.

The views are the slice 2b spots in Holbrook:
- `newel`: the newel stair's room;
- `flight`: the foot of the straight flight;
- `front`: the Fieldstone house's door from the lane;
- `lane`: down the lane;
- `above`: the town from 60 m.

The shots show what things look like, not how smoothly they run. Frame cost
was not measured: the container has no GPU.

## Before

`before-2026-09-30.jpg` is the build before this change: newel, flight and
front at 08:00, 11:00, 14:00 and 17:30. The same shots are the top row of
each sheet below.
- **The rooms.** A wall that faces the sun is as bright indoors as out,
  whatever the hour (the newel's white plaster), and a face turned from the
  sun is near black (the flight's walls).
- **Not taken.** The flight and the front at 17:30, and every view at 22:30:
  the build was replaced under the run.
- **Not a before.** All three views at 14:00 and the newel at 17:30 are
  capture flakes, labelled "(flake)" in the sheets: the walker was not
  standing in the town when the frame was taken, and the shot is of sky or
  sea. The before that stands is 08:00 and 11:00.

## After: the rooms

`newel.jpg`, `flight.jpg` and `front.jpg`, one sheet a view:
- a row each for the build before this change, this change, and the towns
  mockup's same spot (`tools/mockup_village_2b_shots.js`: the newel from
  below, the flight's foot, a door open);
- a column an hour: 08:00, 11:00, 14:00, 17:30 and 22:30.

The 22:30 shots take `--weather-at 3600`: at 7200 Holbrook is under a storm
at that hour (`towns::tests::print_the_rain_over_holbrook`).

- **Shadowed and warm at every hour.** A room takes 0.3 of the sky with a
  door open and no direct sun. Its warmth is its own hearth and the stairs'
  sconces, which burn all day, as the mockup's do.
- **By day a room does not follow the sun.** 08:00 to 17:30 read the same
  indoors, as they do in the mockup. The sun reaches in only through the
  door.
- **At night the candles come on.** The rooms at 22:30 are the mockup's warm
  tan, and the house front's window glows.
- **Against the mockup.** By day the game's rooms are darker and more
  orange than the mockup's cream plaster. On a wall patch in sRGB:

  | Patch | Game | Mockup |
  | --- | --- | --- |
  | Newel, 08:00 to 17:30 | 123, 83, 55 | 160, 119, 79 |
  | Flight, 08:00 to 17:30 | 116, 88, 60 | 134, 111, 81 |
  | Newel, 22:30 | 166, 107, 55 | 176, 121, 68 |

  A larger share of the sky by day (0.45 and 0.6, `--room-sky`) is being
  captured against it.
- **The flames** read pale peach rather than orange: the tonemapper takes a
  bright warm colour toward white. A flame's core is pale, so it stays.
