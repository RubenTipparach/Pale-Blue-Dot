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
stuck, it turns 40 degrees away from the sea and walks on. The recording runs
with `--frame-log`, whose settings have auto step on.

## Proven before it is filmed

A test walks the same steering in 2 m steps over the height field from the
chosen start for 2 km and checks: no step rises or falls more than 2.5 m, the
sea is within 40 m on its side for most of the way, and at least one river is
crossed.
