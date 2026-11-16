# Design: two things were in the way, not one

## The wall, and the floor under it

The obvious blocker was one clause in the swept ground resolution: a wet
footprint was rejected exactly as a cliff is. Removing it is not enough, and
the second half is the more interesting bug.

`SurfaceContact::radius` is computed at `PLANET_RADIUS + height.max(0)`. Over a
water cap that clamp puts the contact plane at **sea level**, so the walker's
footprint reports the sheet as its support: with the rejection gone, a walker
would simply walk out onto the top of the sea. The clamp is right for its other
two callers, which is why it is there: assisted flight must clear the water,
and rain must land on it.

So the contact answers both questions now. `radius` is the surface you fly over
and rain falls on; `floor_radius` is the solid ground, the seabed under a water
cap and the same value everywhere else. The walker's footprint takes the floor.
One record, two fields, no second sampler.

## The probe, and where it lives

`Sea` is a `SystemParam` holding the terrain and the water settings, because
answering "is the walker in water" needs both and two systems need the same
answer. It also keeps the sheet radius in one place: `PLANET_RADIUS -
depth_offset_m`, the radius the water pass DRAWS at, so the physics and the
picture cannot disagree about where the sea is.

It arrived for a second reason, which this project's rules predict: adding the
probe to `drive_walker` pushed it to eight arguments and Clippy said so. The
rule is that the allow is the smell and the struct is the fix.

## Held, not tapped

A ground jump is an impulse on an edge; a swim thrust is an acceleration while
the control is down. The walker only had the edge, and `just_pressed` cannot
express holding. `WalkingState::jump_held` is the other half, and the two are
kept separate rather than one being derived from the other, because they mean
different things at the same instant: the frame you press space in the shallows
is both an edge and a hold.

## The exit, which is where we diverge

The reference gates the swim thrust on the **eyes** and gives a player floating
at the surface the **jetpack** to climb out with, unlocked the moment the eyes
clear the water. This project has no jetpack, so gated on the eyes alone a
swimmer beside a bank would bob there for ever. The thrust is gated on the
**body** while the feet are off the bottom, which covers the last metre. It is
recorded here because it is the one number that is not the reference's.

## What a scripted capture taught us about input

Two real defects, both found only because a headless capture had to press a key:

- **Scripted keys must be written where the real ones are.** Pressed in
  `Update`, they are wiped by the next frame's input clear before
  `RunFixedMainLoop` reads them. The walker stood still and nothing said why.
- **Pointer capture follows the window's focus, and a headless window never
  reports any.** So the input path dropped capture every frame and zeroed the
  movement axes. `WalkingState::scripted` is what a capture sets to say the
  keys are coming from a script.

Both are the same shape as the tests: a test that sets the movement axes
directly passes while the real input path is broken, because the axes are
rewritten from the keyboard every frame. The tests drive keys now.

## Auto-climb out of the water (owner, 2026-11-16)

The owner: *"introduce auto climb when in water and trying to climb up a ledge
of 1 or 2 blocks"*.

### What is there today, measured off `walking.rs`

- A swimmer leaves the water only by holding jump: the swim thrust
  (`water_swim_force` 20 m/s², drag 3/s) rises at about 4.2 m/s until the body
  probe clears the surface, and the swimmer then has to be moving forward at
  the instant the feet clear the ledge. In practice it is a bob against the
  bank.
- Wading (water up to `water_wade_depth`, two layers plus 0.1 m, over the
  feet) is grounded, so auto step (`step_height`, one layer, 1.05 m) already
  walks up a ONE-layer bank out of a shallow; a two-layer bank stops the wader.
- The sweep (`sweep` in `resolve_ground`) refuses a move into a rise it cannot
  step and reports the face's normal along the ground; the slide takes the part
  of the move along the face.

### The rule

While the walker's **body is in water** (the body probe, 0.5 m over the feet,
under the surface: swimming or wading), the **forward control is held**, and
the sweep **refused the move against a face** the heading points into, the
walker climbs onto the ledge beyond the face when:

| test | value | knob |
| --- | ---: | --- |
| facing the face: heading · (into the face) | at least **0.5** (within 60°) | `water_climb_facing` |
| ledge top over the water surface | at most **2.7 m** (two layers, 2 × 1.05, plus the sea sheet's 0.5 m under its layer top and a 0.1 m skin) | `water_climb_m` |
| or, wading, ledge top over the feet | at most **2.2 m** (two layers plus 0.1, the wade depth) | `water_climb_wade_m` |
| ledge top over the surface | **above it** (a step still under water is the seabed's, and swimming down past it is left alone) | - |
| ledge top over the feet | at most **4.5 m** (a body's length under the surface plus two layers; a swimmer deeper than that swims up first) | `water_climb_reach_m` |
| room on the ledge | the body stands there under any ceiling, and no town solid holds it | - |
| how long | **0.4 s**: the body rises to stand on the ledge over the first 60 % and moves **0.8 m** forward onto it over the rest, both eased | `water_climb_secs`, `water_climb_forward_m` |

Why two references: the brief's "2 layers above the water surface, or above
the feet when wading". A wader is allowed either test, so standing on the bed
is never worse than floating over it. The sea's drawn sheet sits 0.5 m under
the top of its layer (`depth_offset_m`), so a beach cap two layers over sea
level is 2.5 m over the sheet; the 0.6 m over two layers is what keeps "two
blocks" meaning two blocks at sea, in a river (whose surface is a layer top)
and in a flowing cell (whose surface sits a fraction into its layer). A cap
three layers up is 3.0 to 3.5 m over any of them and does not climb.

During the climb the motor and gravity stand down and the body is placed on the
eased path each tick (no velocity carried in, none left over); it lands
grounded. It is never a teleport: each tick moves the body at most the path's
share of 0.4 s.

What does not climb: a ledge more than the table allows (the swimmer stays in
the water or uses the swim rise, unchanged); a walker moving away from the bank
or along it (no refused move, or a heading within 60° of along the face); a
swimmer under water beside a step whose top is still under the surface; a
swimmer more than 4.5 m under the ledge top; a walker with the forward control
up.

### Shared with auto step

The climb follows the **auto step** player setting (`player-settings` 5):
on (the default) it climbs; off, it does not, and the swim rise is the way out
as before. Auto step is the setting that says "the walker goes up a rise
without a key"; a player who turned it off to have rises stop them asked for
that in water too. Recommendation taken (ask only with screenshots).

### Where it is

Water of every kind: the probe is `Sea::state`, whose surface is the column's
water (sea at the tided sheet, a river, a flowing cell, a poured pool) on the
tier and the height field's sea off it, so the climb is measured against the
surface the walker is actually swimming in.
