# Proposal: a flight from the ground, through space, to the ground on the far side, at a steady 60 fps

## Why

The owner, on the first recording: "I want you to record a flight from one side
of the planet to the other in real time", and then: "the way I imagine this fly
through happening is that you go into fly mode, fly up at a 60 degree angle, then
orbit the planet to the destination, and fly down to get to the destination", and
of the result: "why is the video all choppy? I wanted a smooth transition from
ground to space back to ground again in perfect 60 fps".

### The route does not exist

`--tour` starts the ship at cruise, 1 km above sea level at 600 m/s, and flies
great circles until one lap is done (`flight_view.rs`, `tour.rs`). It never
leaves the ground, never goes above the atmosphere (the shell tops out 960 m
above sea level), and never lands. The recorded video is therefore one height
from start to end.

### The frame rate is not steady

Measured on the owner's desktop (RTX 3070, i7-9700F, release build, 1440 x 900):

- **The tour's own frame times** (`--capture ... --tour --frames 2400`, no VSync):
  p50 2.66 ms, p95 6.23 ms, **p99 37.56 ms**. One frame in a hundred misses a
  60 Hz deadline by more than twice over.
- **The recording** (OBS at 60 fps, the game at VSync): counting the frames that
  differ from the one before, most seconds carry only **20-45 new frames of
  60**, with freezes of **9-19 frames (150-300 ms)**. Some of the night-side
  seconds may be undercounted, since a dark frame changes little.
- **The cause, by the log:** in the recorded 47 s the fine set (the L8-L11
  detail around the player, `planet_lod.rs`) was rebuilt **35 times**, one
  landing every ~1.3 s. It rebuilds whenever the player is `REGEN_DISTANCE_M`
  (40 m) from its anchor, which at 600 m/s is continuous. The generation runs
  on every core ("the fine set in a second"), competing with the main thread,
  and each landing swaps in a new set of records on the frame it lands. From
  1 km up the finest bands cannot be seen at all.

## What

1. **A far-side route.** A new automated flight mode, `--route far-side`:
   - takes off from the walker's spawn, standing on dry land;
   - climbs out at 60 degrees above the local horizon to a cruise height above
     the atmosphere;
   - cruises the great circle, then descends on a matching glide to land at the
     antipode (the nearest dry land to it, so it lands rather than splashes).
   Every phase change is continuous in velocity and acceleration, so the camera
   never jerks. The speeds and heights are validated data with units, not
   literals.
2. **No ground detail rebuilt while it cannot be seen.** The owner: "after a
   certain height you should not be rebuilding detailed terrain obviously".
   - Above a clearance where the finest band falls under a pixel, the fine set
     is not rebuilt, whatever the flight mode, manual flight included.
   - On the way down it is rebuilt ahead of the ship, at the predicted touchdown,
     so it has landed by the time the ground is close.
   - Generation leaves at least one core to the main thread.
   - A landing is measured, and spread over frames if it is what hitches.
3. **A camera that flies like a player.** The owner: "I also want the camera
   angle to focus on the planet, be more dynamic, like how a player would fly
   around". The route's camera banks into turns, pitches toward the planet
   through the climb, holds the planet's disc and limb in frame through the
   cruise, and swings to the landing site on the way down. It is scripted, not
   the player's mouse: `CLAUDE.md`'s no-easing rule is for raw mouse look, and
   this camera eases because no hand is on it.
4. **A frame-time instrument**, `--frame-log <path>`: every frame's wall time,
   with the fine-set events on the same clock, so a hitch can be attributed and
   not guessed at.
5. **A cadence check** in the `obs-record` skill: the share of frames in each
   second that differ from the one before, so a recording is judged "60 fps"
   by measurement and not by eye.

## Success

On the owner's machine at 1440 x 900, the far-side route:

- completes from takeoff to touchdown, with the clearance never below the
  protection minimum;
- holds **p99.9 frame time under 16.7 ms** by `--frame-log`;
- records at **60 of 60 new frames per second** by the cadence check, from
  takeoff to touchdown.

## Not in this change

- Manual flight's own handling, and the Kestrel (a craft on this route is a
  later step).
- Cloud or terrain look.
