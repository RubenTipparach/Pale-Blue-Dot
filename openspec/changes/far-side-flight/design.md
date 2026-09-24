# Design: the far-side route and a steady frame

## The route

One guidance function in `flight_view/route.rs`, in the style of `tour.rs`: a
commanded acceleration and attitude from the ship's position, velocity and the
route's state. The CPU physics stays authoritative; this only commands it.

The phases, measured along the great circle from the start `s` to the
destination `d` (the antipode, snapped to the nearest dry column):

| Phase | Commanded | Ends when |
| --- | --- | --- |
| Hold | still on the ground, 1 s | time |
| Climb | flight-path angle 60 deg above the local horizon, speed easing up to the climb speed | height reaches cruise |
| Cruise | the great circle at cruise height and speed, with tour.rs's centripetal feed-forward | the remaining arc equals the descent's arc |
| Descend | a 60 deg glide, mirrored, speed easing down | clearance under the flare height |
| Flare and land | vertical speed to touchdown speed, horizontal to zero | on the ground, then 2 s |

Speeds are blended between phases with smoothstep over a fixed time, so the
acceleration is continuous. The descent starts at the arc that the climb took,
so the flight is symmetric and the far-side landing is where it was predicted.

The settings come from `FlightViewConfig` with units: `route_climb_angle_deg`
(60), `route_cruise_height_m`, `route_climb_speed_mps`, `route_cruise_speed_mps`,
`route_touchdown_mps` and `route_blend_s`. The cruise height is chosen so the
planet's disc shows whole, with the atmosphere's rim below the camera. It is
measured by eye in a capture and written down, not guessed.

## The camera

The ship flies the route, and the camera is a separate rig on it, aimed by a
look target and a roll that change with the phase. It eases toward both,
critically damped with no overshoot, because it is scripted, not the player's
mouse:

| Phase | Looks at | Roll and motion |
| --- | --- | --- |
| Hold | Along the runway, the horizon a little above centre | Still |
| Climb | Pitches from the horizon up the climb, then over the shoulder back down at the ground falling away | Banks into a slow turn off the start |
| Cruise | The planet: the disc's limb across the lower third, the terminator and the clouds, leading the direction of travel | A slow roll through the long curve, as a player drifts the view |
| Descend | Swings forward to the landing site and keeps it framed | Banks into the final turn, levels out at the flare |
| Land | The ground ahead, then a settle | Level |

The rig's settings are data with units: the look lead in seconds of travel, the
bank per turn rate, and the ease time. The route's headless check reports the
camera's peak angular speed and acceleration, so "dynamic" never becomes
nauseating. The accessibility rule on reduced camera motion (`game-design.md`)
applies: it is a setting that damps the roll and the swings.

## Frame pacing

- **The fine-set gate, in every flight mode.** Rebuilds are skipped while the
  camera's clearance is above the height where the finest band's 2.8 m tile
  falls under a pixel. That height is computed from the projection, not written
  as a literal. The resident set stays drawn; it is simply not replaced. It
  applies to manual flight as much as to the route: the rebuilds that stuttered
  the tour happen to any player flying fast.
- **The descent's lead.** When the route predicts touchdown within
  `rebuild time x 2`, it asks for the fine set at the touchdown point. The
  anchor is the predicted landing, not the ship's current position.
- **Cores.** The fine set's thread pool leaves one core free. Measured before
  and after with `--frame-log`: if generation is not the cause, this does not
  ship.
- **Landing.** Timed alone with `--frame-log`. If a landing frame is over
  budget, the upload is split into bands over several frames, publishing only
  complete bands per the rendering rules.

## Instruments

- **`--frame-log <path>`**: a CSV with frame index, wall ms, the phase, the
  clearance, and the fine-set events (requested, landed, took). It is written
  only when asked for.
- **`--verify-route far-side`**: headless, like `--verify-flight`. It reports
  completion, the touchdown distance from the destination, the minimum
  clearance, the peak acceleration and the peak jerk.
- **The cadence check** (the `obs-record` skill): the share of frames per
  second that differ from the previous one. A threshold relative to the scene's
  own noise avoids night-side false repeats.

## Recording

The game runs at VSync on a 60 Hz or faster display, and OBS records at 60 fps.
The skill's cadence check must report 60/60 over the flight before the video is
uploaded.
