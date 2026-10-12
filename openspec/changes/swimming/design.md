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
