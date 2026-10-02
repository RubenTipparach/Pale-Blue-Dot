# Design: town-flyover

## The tour

| # | Town | Kind | lat, lon (shipped seed) | Leg from previous |
| ---: | --- | --- | --- | ---: |
| 1 | Coringport | harbour | 14.07, 72.28 | (start) |
| 2 | Mirhan | desert town | 4.33, 61.82 | 1,188 m |
| 3 | Dunmouth | village | 9.626, 56.571 | 622 m |
| 4 | Ashenstead | walled town | 19.644, 59.178 | 865 m |
| 5 | Ulvevik | tundra camp | 78.22, -9.39 | 5,568 m |

Coordinates from `docs/mockups/world-map/sites.json` (generator 6, seed
1592598566); the game picks them from the world's own stored sites
(`crates/pbd-app/src/sites.rs`), so another world gets its own tour. Every
tundra camp lies near a pole, so the last leg is the long one whichever set
is chosen: the shortest tour ends there rather than visiting it in the
middle and flying back.

## The camera path

One cubic spline in time through keyframes, position and look-at target
separately, with the speed set by the time between keyframes (centripetal
Catmull-Rom, so it never overshoots a keyframe).

At each town (its template is a 50 x 34 hex grid, about 142 x 83 m):

- **Arrive** at 160 m from the centre, 90 m up, on the bearing the leg
  arrives on.
- **Sweep** half a circle round the centre at 130 m, from that bearing round
  to the bearing of the next leg (on whichever side is shorter), descending
  to 60 m at the middle of the arc and back to 90 m. The look-at target is the
  town's centre, raised 10 m, so the camera is pitched about 20 degrees down.
  At 25 m/s the arc takes about 16 s.
- **Leave** along the next leg.

Between towns, the path climbs to a cruise height of 8% of the leg's length
(within 150 and 600 m) and flies the great circle at up to 250 m/s, easing in
and out. The look-at target leads the camera by two seconds of travel, so in
transit it looks ahead along the flight and swings onto the next town as the
camera arrives.

The camera is the only thing moved: the pilot ship is placed on the path each
frame (kinematic, no physics), so the ground detail, the towns' standing range
(1,200 m) and the clouds all stream round the camera exactly as they do in
flight.

## How long

| Part | Estimate |
| --- | ---: |
| 5 town arcs, about 16 s each | 80 s |
| Legs of 1,188, 622 and 865 m (about 130 m/s on average) | 21 s |
| The 5,568 m leg (cruise 250 m/s, eased) | 28 s |
| Lead-in and hold at the end | 6 s |
| **Total** | **about 2 min 15 s** |

At 60 fps that is about 8,100 frames; at the recording settings used so far
(H.264 at 12 Mbit/s, 1440 x 900) about 200 MB. The sun turns 17 degrees in
that time (a day is 2,880 s).

## Time of day

The five towns span 82 degrees of longitude, about 5.5 hours of local time.
`--time` is set so the first four (longitudes 56 to 72) are in the early
afternoon and Ulvevik (longitude -9) in the morning; on day 0 the sun stands
23.5 degrees north, so Ulvevik, at 78 degrees north, has the midnight sun.
The exact hour is chosen from overview captures of all five.

## Measurement

1. **Each town's cost from the air** (before building anything): a windowed
   run over each town with the existing flags (`--view column --at lat lon
   --height 90 --pitch -40 --no-vsync --frame-log`), 30 s each, release
   build. This says whether any town is too heavy for the shot.
2. **The shot itself**, once built: `--route towns --no-vsync --frame-log`,
   windowed, real time, two runs, nothing else running; its length, frame-time
   percentiles, the slowest frames and what was streaming then.
3. Overview captures of each town at the chosen hour, for the owner.

## Measured so far (task 1)

Release build, RTX 3070, 1440 x 900, uncapped, a still camera 90 m over each
town pitched 40 degrees down, `--time 10.5`, 2,400 frames, the first 10 s
thrown away. Capture mode: the camera does not move, so these are real
per-frame drawing costs, not the shot's pacing.

| Town | Kind | Frame p50 | p95 | p99 | GPU p50 | Sun |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Coringport | harbour | 10.6 ms | 11.5 | 12.4 | 8.6 | 68 deg |
| Mirhan | desert | 9.4 | 12.9 | 17.2 | 9.2 | 68 |
| Dunmouth | village | 10.4 | 16.2 | 20.0 | 10.4 | 75 |
| Ashenstead | walled | 11.4 | 12.3 | 13.4 | 9.6 | 81 |
| Ulvevik | tundra | 10.2 | 17.4 | 19.6 | 9.9 | 29 |
| (no town: open sea at 0, 30) | | 8.4 | 14.3 | | 8.1 | |

- **About 90 to 105 fps over the towns**, GPU-bound; the 95th percentile is
  within 60 fps everywhere, the 99th dips under it at Mirhan, Dunmouth and
  Ulvevik (17 to 20 ms).
- **The towns are not the cost.** The ground at this height and angle is:
  open sea alone is 8.1 ms of GPU, a town adds 0.5 to 2.3 ms, the clouds 0.1
  to 0.2 ms at this hour (`PBD_NO_CLOUDS` changed Dunmouth by 0.2 ms). The
  2026-09-25 flights spent 1.3 to 3 ms of GPU from their own heights; whether
  the ground has grown dearer since is worth a perf-suite run of its own.
- **The look:** at `--time 10.5` with the default weather every town lies in
  a grey haze (`docs/screenshots/town-flyover-towns-from-90m.png`, the sixth
  frame is the sea). The town shots used `--rain 0 --weather-at 7200`; the
  flyover takes the same. The still camera faces east from straight over each
  town, so it frames them poorly; the flyover's arcs are what frame them.

## The owner's notes on the first cut (2026-10-02)

The owner, on the first flyover: "There are gaps in the terrain on the sides
of hex faces. The fly over should fly through the city to get a more
detailed look. The fly over should also fly up in to the sky and to the next
city instead of gimbal locking the camera and teleport.. I don't like how
you're doing transitions we had good transitions before".

The first cut was built in the desktop session and not pushed. Each note
changes the design above; the path, the legs and the camera are redone as
follows before anything is recorded.

### 1. Through the town, not round it

The 130 m half-circle at 60 to 90 m shows a town as a model on a table. The
owner wants a detailed look, so the shot goes through it.

- **The pass.** Each town is crossed on a straight line through its centre,
  along its long axis (the template's columns, 142 m), entering on the side
  the leg arrives from where that is within 60 degrees of the axis, else
  turning onto the axis in the approach.
- **Height.** 6 m over the town's highest roof or wall on the line (the
  template's buildings and masonry give it, so a walled town's towers and
  the jungle's canopy are cleared), and never under 12 m over the terrace.
  That is rooftop height: the lanes, the doors and the dressing are seen.
- **Speed and look.** 12 m/s through the town, about 12 s for 142 m. The
  look leads the camera by 1.5 s, pitched 15 degrees under the line, so it
  looks into the streets ahead, not at a fixed centre.
- **In and out.** The descent into the pass and the climb out of it are
  the legs' own glide and climb (below), so the pass is part of the one
  line, not a separate move.

### 2. Up into the sky and on to the next town: the far-side route's flight

The far-side route already flies the transition the owner liked:
`flight_view/route.rs`, "one rule shapes the whole flight: the target
height at a point is the lowest of the cruise height, the arc flown times
the climb slope, and the arc remaining times that slope". Each leg between
towns is that rule, from the end of one town's pass to the start of the
next one's.

- **Climb, cruise, glide.** It climbs at the far-side route's angle to a
  cruise height that grows with the leg: 25% of its length, within 300 and
  1,500 m. The 5.5 km leg to the tundra camp cruises 1.4 km up, high
  enough to see the planet round off, and then glides down into the camp's
  pass. There is no cut and no jump anywhere: every leg is flown.
- **Speed.** Eased from the pass's 12 m/s to a cruise of up to 250 m/s and
  back, so the shortest legs are a hop and the long one a flight.
- **The camera rig.** The far-side route's: the look leads the path, drops
  to the ground as the horizon falls away, and swings to the next town on
  the way down; it eases without overshoot.

### 3. No gimbal lock

A look-at with the local up as its up fails when the look runs along the up:
straight down over a town, or a steep climb. There the camera's roll is
undefined, and it spins or snaps. That was the "gimbal locking".

- The camera's orientation is a quaternion, carried from frame to frame:
  each frame it turns toward the wanted look by the shortest rotation,
  rate-limited and eased, as `tour.rs`'s `attitude_acceleration` turns the
  ship. Its roll is held level to the local horizon by a second, small
  correction, not rebuilt from the look each frame.
- The wanted look never comes within 20 degrees of straight down or up,
  so the level-horizon correction always has a horizon to level to.

### 4. Gaps in the terrain on the sides of hex faces

From the air, the cliff faces between hex columns of different heights show
gaps. Not yet measured: the first cut's footage is on the desktop.

- **Measure first.** Column-view captures over each town at the pass's
  height and the legs' heights (60, 300 and 1,000 m), looking at stepped
  ground, to find where the side faces are missing. Then see whether the gaps
  fall on a boundary between detail tiers or terrain tiles (`hexagon-lod`
  says neighbouring tiles must agree, or the boundary cracks) or inside one
  tier.
- The fix is written up once it is found, in the change that owns that
  ground (`hexagon-lod` or the column tier's), not in this one.

### The jungle

The jungle village is built (`cities-in-the-world` slice 4i), so the tour
takes six kinds: a jungle village joins the harbour, desert town, village,
walled town and tundra camp, by the same shortest-tour choice.
