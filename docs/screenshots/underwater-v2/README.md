# underwater-v2: light under water (`ocean-environments` group 4)

Fixed-step captures on the Mac (M4 Max, Metal, `--profile fast`), each in a
world made fresh for the shot. The walker sinks from the sheet to the seabed
and stands on it, so every depth is the eye's depth over a real floor. Frame
time is NOT measured here: these are looks, not performance.

`pbd-app --world <new> --walk --at <lat> <lon> --pitch <-10|40> --time <hour> --frames <400 + 25 (d + 4)> --capture <png>`

| Depth (eye) | `--at` | `--time` | sun at the spot |
| --- | --- | --- | --- |
| 10 m (floor 10.9 m) | -4.125 -11.125 | 11 | 30 deg up |
| 40 m | -19.0 -11.75 | 11 | 21 deg up |
| 90 m | -12.375 12.25 | 11 | 43 deg up |
| 123 m | 10.5 -88.0 | 16 | 35 deg up |

**140 m is not on this seed.** The deepest floor the generator makes is
125 m (10.5, -88.0), so the fourth depth is 123 m (design B1).

- `before-*.jpg`: `claude/phase-1` at `6632239`, one absorption for everything.
- `after-*.jpg`: downwelling for the light arriving, turbidity, caustics,
  shafts and marine snow. `-p-10` looks level, `-p40` up 40 degrees.
- `after-10-sun.jpg`, `after-40-sun.jpg`: facing the sun, 15 degrees up
  (`--yaw -150 --pitch 15`): the shafts.
- `compare-sheet.jpg`, `before-sheet.jpg`: side by side.
- `spray-real-wind.jpg` and `spray-test-from-1mps.jpg`: the fishing shore
  (`--fish --pitch -12 --time 11`). No gale was found to film, so the second
  is a TEST with `spray_from_mps` lowered to 1 m/s in a scratch copy of
  `water.ron`, to show what the streaked foam looks like at full strength;
  the shipped value is 17 m/s.

What changed, measured on the level frames (mean of the floor band, sRGB
0-255): the 90 m floor went from a lit blue (about 0, 1, 13) to near black,
the 123 m frame is black but for the snow, and 40 m sits at the twilight's
top edge with its shafts and snow.

## Water life (`ocean-environments` group 7, first cut)

`--life <id>` is a capture instrument: five seconds after the walker is up it
puts one group of that species 3 to 14 m ahead of the eye.

- `life-shore_crab.jpg`, `life-giant_clam.jpg`, `life-lobster.jpg`,
  `life-reef_turtle.jpg`: on a flat 4.8 m seabed (`--walk --at 3.5 -18.15
  --pitch -35 --time 11 --frames 560`). The turtle has already swum out of
  frame; the clams nearest the eye are shut.
- `life-reed_dabbler.jpg`, `life-sea_otter.jpg`: from the fishing shore
  (`--fish --life <id> --time 11 --frames 560`); the ducks are at the
  bottom of the frame.
- `life-zoomed.jpg`: the crab, clam and duck enlarged.

They are drawn as the fish are, small lit cones tinted by the species'
colour, not yet the pixel cards of Q2; the owner judges whether to go on to
cards. Over a 25 s fishing run the spawner put lobsters, a pond diver, a
reef turtle, reed dabblers and shore crabs in the water beside the minnows,
rays, eels and perch.
