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

## The scenic route

The owner's next recording: "flying through the clouds and showcasing some cool
and interesting land features". The far-side route punches straight up through
the cloud layer, 300-750 m above sea level, to cruise at 3 km, so it shows
neither. A second route, `--route scenic`, flies low instead.

- **Chosen, not guessed.** Waypoints are picked by scoring the terrain
  generator around the spawn, within about 6 km:
  - relief, the height range within 300 m;
  - coast, land and sea within 300 m of each other;
  - how rare the biome is among the candidates.

  A greedy pick takes the highest scores at least 1.5 km apart and orders
  them into a path from the spawn. The picks are logged with their scores, so a
  run says why it went where it went.
- **Through the clouds.** The first leg crosses the cloud layer at mid-slab
  (about 500 m above sea level), where the weather map has cover. The pick
  prefers the direction with the most cover and says so when there is none.
- **Over the land.** Past the clouds the route descends to a terrain-following
  height, about 90 m above the ground ahead. It is read a few seconds ahead
  along the path and smoothed, so it rises over a ridge before reaching it.
- **Speed:** 150 m/s along the path, slow enough for detail near the ground to
  keep up and for the land to read.
- **The camera** looks along the path, turns toward each feature as the ship
  passes it (the look target leads to the feature's point, then releases), and
  keeps the route rig's ease, rate cap, sway and reduced-motion setting.
- **It lifts off from the ground and lands at the last feature,** with the
  far-side route's lift-off, touchdown and complete lines, so the same recording
  recipe applies. `--verify-route scenic` checks it headless, as the far-side
  route is checked.

## Frame pacing

- **Bands measured from the player's true position, height included.** The
  owner: "the player cant even see that! detailed sets should also account for
  height radius". Today a band is a great-circle radius on the ground (`BAND_M`,
  `band_cos` in `planet_lod.rs` and `planet_visibility.wgsl`), so from 1 km up
  the 300 m finest band is still built and drawn under the ship, although the
  nearest ground is 1 km away.
  - **The rule:** a band of slant radius `B`, seen from height `h` above the
    ground, covers the ground out to `sqrt(B^2 - h^2)`, and is empty once
    `h >= B`.
  - **One table:** the CPU computes the four band cosines each frame from the
    player's height and publishes them in the view uniform. The GPU's level
    choice and the fine set's builder both read that table; neither keeps its
    own copy of `BAND_M`.
  - **What it gives:** at 300 m up level 11 is gone; at 2.4 km up the whole fine
    set is empty and nothing is built. The height used is the player's (the
    ship's), never the free camera's, so looking around still changes no
    level.
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
