# Design: a walk along the coast

## Steering (`pbd_app::coast_walk::steer`)

Every 0.2 s, from the walker's direction and heading, each candidate heading
from 75 degrees left to 75 right in 15 degree steps is looked along at 4, 8,
14 and 22 m (on the sphere), reading `surface_height`:

- a cliff: any stretch between reads rising or falling more than 3 m is
  ruled out; one rising more than 1.5 m (more than auto step's one layer and
  a hop) costs 10 a metre over, so it is taken only where nothing flatter is;
- water (height under sea level) is a river if land comes back within 30 m
  along the heading, crossed; else it is the sea, ruled out;
- of what is left, the score is how far the point 8 m out sits from 2.5 m
  above the sea (the shore's band), plus a cost per degree of turn, plus a
  reward when the point 15 m to the sea's side of it is water: the sea stays
  on that side.

The view turns toward the chosen heading at no more than 50 degrees a second,
so the camera reads as a walker looking where it goes. Stalled on a step the
height field cannot see (an edit, a tree), it hops, as the stroll does; 1.5 s
stuck, it turns 40 degrees away from the sea and walks on; trapped in one
spot (three turns there) it holds jump, and at four it backs out, walking
backwards with a hop for 1.5 s, then turns a quarter. It never digs. The
recording runs with `--frame-log`, whose settings have auto step on.

## Proven before it is filmed

A test walks the same steering in 2 m steps over the height field from the
chosen start for 2 km and checks: no step rises or falls more than 2.5 m, the
sea is within 40 m on its side for most of the way, and at least one river is
crossed.

## The take

`--clock-runs` (an instrument) lets `--time` set the hour the clock starts
from without stopping it. A planet day is 2,880 s, so a game hour is 2 real
minutes and the walk (about 6 minutes) spans 3 game hours: from `--time 8` at
the start (sun 12 degrees up) it walks through sunset (about `--time 9`) into
the night, so one take shows the low sun, the dusk sky, the stars and the
water at each.

`pbd-app --world <new> --walk --at 30 177.5 --time 8 --clock-runs --coast-walk 2000 --no-vsync --frame-log <csv> --frame-graph`

## COAST_TURN's latitude and the compass test (found 2026-11-14, not changed)

`desktop::tests::the_screen_reads_the_compass_not_the_frame` fails on main:
`COAST_TURN` (1.2, `f07556d`) logs where a stall happened with
`pbd_core::geo::lat_lon(up)`, the FRAME's latitude, and the test refuses the
frame's functions anywhere in `desktop.rs` (`compass-bar` decision 2: the
frame's north is the compass's south, so a readout in it mirrors the planet).

The line is not on the screen. It is a log line for whoever reruns the walk,
and it is in the frame on purpose, whether or not that was meant: `--at`
takes the frame's latitude (`spawn_direction` passes it straight to
`geo::direction`), and the take is launched `--at 30 177.5`. A stall logged
at frame latitude 30.0123 is reproduced by `--at 30.0123 ...`. Routing it
through `geo::compass_lat_lon` would print -30.0123, and a rerun pasted from
the log would start on the mirror of the spot. That is a behaviour change to
the instrument, so it was written up here instead of made
(`stone-and-wood-ui`'s close, which met it).

Ways out, for the owner or the next coast-walk step:
1. **Recommended:** print the spot as the flag that reproduces it, through
   one helper beside `spawn_direction` (the inverse of `--at`, named for
   it), and let the test allow that helper by name: the frame is `--at`'s
   contract, and the screen still never reads it.
2. Print the compass latitude and make `--at` take the compass's, so the log,
   the map's readout and the flag agree. That moves every `--at` written
   down so far (this file's take, the town captures in `cities-in-the-world`)
   to its mirror unless each is negated in the same commit.
3. Move the coast walk's script out of `desktop.rs` into `coast_walk.rs`,
   which the test does not read. It passes the test without answering it.
