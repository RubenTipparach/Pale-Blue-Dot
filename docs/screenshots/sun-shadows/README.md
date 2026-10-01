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

## After: indoors and out across the day

`day.jpg` is this change alone: a column an hour (08:00, 11:00, 14:00,
17:30, 22:30), and a row a view, indoors first:
- the newel stair's room and the flight's foot, indoors;
- the house front's open door from the lane;
- the lane, and the town from 60 m, outdoors.

What it shows:
- **Indoors.** The rooms keep their soft warm light all day and go warm tan
  when the candles light at night.
- **The lane.** At 08:00 the sun is low in the frame and the house fronts
  stand in their own shade. At 11:00 and 14:00 the fronts are evenly lit.
  At 17:30 the trees' long shadows lie across the grass.
- **From 60 m.** The shadows turn with the sun. From 14:00 the walls toward
  the camera are in their own shade, and the trees shadow the fields.
- **At 22:30.** The windows glow down the lane. From above, the warm pools
  on the fields are the glowing flowers lighting their sod
  (`lamps-and-lanterns` decision 8), stepped cell by cell like every light
  in the voxel field. They are not the rooms' lights, which stay indoors.

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

  `room-sky.jpg` tries a larger share of the sky by day: 0.45 gives 127,
  90, 65 and 0.6 gives 132, 97, 73. That is a quarter of the gap at double
  the share, and the room turns greyer, not cream. The share stays at 0.3
  (the design's "Tuning across the day").
- **The flames** read pale peach rather than orange: the tonemapper takes a
  bright warm colour toward white. A flame's core is pale, so it stays.

## After: shadows on and off

`shadows-on-off.jpg` is the lane and the town from 60 m at 08:00 and 17:30,
each with the cascades and with `--no-shadows`. The lane and the view from
above have no before (see "Before"), so the change is shown against itself.
- **The lane at 17:30.** The stone house's long shadow covers the grass on
  the left, and the trees shadow the grass on the right. Without the
  cascades the whole field is lit.
- **At 08:00.** The sun is low in front of the camera, so the shadows fall
  toward it and are short in the frame. The house fronts are in their own
  shade either way: a face turned from the sun takes the sky's fill.
- **From 60 m.** Each house and tree throws its shadow away from the sun. At
  60 m the far cascade holds them (texels of about 18 cm), so they are soft.

## Not done here

- Frame cost was not measured: the container has no GPU. The owner runs
  `tools/perf_suite.py` (task 7.2).
- Crawl and shimmer as the sun moves need a real-time recording (task 7.3).
- The twilight curve for fill, fog and rim is still the old one (task 6.1).

## A fault in the views from above (found 2026-09-30)

The town from 60 m (`day.jpg`'s bottom row and `shadows-on-off.jpg`'s bottom
row) was captured without the field-lit plugin, which a still with no walker
never added (`cities-in-the-world` slice 4a, "The dark towns"). The houses in
those shots are lit by Bevy's own sun and ambient, so their faces turned from
the sun are near black. The terrain and its shadows in them are right. The
views from above are retaken on the fixed build.

A second fault, found 2026-10-01 when the retakes came back: the two
`--no-shadows` shots from above were byte for byte the shadowed ones. The
flag was read only in a walking run, inside `desktop.rs`'s block for runs that
are not photos, so a `--view column` still ignored it and drew its shadows.
`shadows-on-off.jpg`'s first bottom row compared shadows with shadows. The
flag now applies in every run, and the two views are retaken. `--open-doors`
and `--room-sky` are still read only when walking. Their photos from above
draw the doors shut, which no view from 60 m can see.

