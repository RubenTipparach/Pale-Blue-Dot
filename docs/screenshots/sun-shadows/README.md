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
front at 08:00, 11:00, 14:00 and 17:30.
- **The rooms.** A wall that faces the sun is as bright indoors as out,
  whatever the hour (the newel's white plaster), and a face turned from the
  sun is near black (the flight's walls).
- **Not taken.** The flight and the front at 17:30: the build was replaced
  under the run.
- **Not a before.** The newel at 14:00 is a capture flake: the walker was
  not standing in the town when the frame was taken.

The shots after this change, and the tuning across the day, follow.
