# Proposal: the rain volume is in the air, not in the sea (rain-under-water)

## Why

The owner (2026-11-14), underwater at night looking up: "snells window
underwater needs some work". Then, on a second shot: "I think snels might
fine, its the rain shadow thats not respecting order/transparency", and on a
third: "you can see underwater is fine if rain shadow turned off".

"Rain shadow" is the heavy volumetric rain (CLAUDE.md: the rain volume, the
full-screen rain pass, its fog and streaks; `rain`, `rain_half`,
`rain_compose` in `water.wgsl`, `rain_needed` in `planet_water.rs`).

Measured on the Mac (`docs/screenshots/rain-under-water/`, before): with the
walker 5.4 m under the sea at `--at 12.748 9.455`, in rain (`Weather here`
rain 0.54 at night, 0.91 by day), the same frame with `--rain-shadows on` is a
flat grey fog over the whole view and with `--rain-shadows off` is the dark
underwater scene: the sheet overhead, total internal reflection outside
Snell's window, the murk. Looking up at 20 degrees the shore's trees stand
in the fog with the rain on, where with it off the surface beyond the window
is a dark mirror and hides them.

The cause is in `water.wgsl`. The rain pass runs after the water's compose
and cap (`planet_water.rs`: "Water compose and cap", the clouds, then "Rain
volume, gathered small"), over the finished underwater frame, and its ray
(`rain_far`, `rain`) starts at the eye: it stops at the scene's depth, the
cloud base, the range or the mean sea sphere's NEAR hit, which from inside
the sphere is behind the eye and so ignored. Every ray from an eye under the
sea is marched through the water as if it were air, and the rain's light and
darkening are laid over the murk, the sheet and the seabed alike. The clouds
pass and the lens already return early under water (`fx.y > 0.75`); the rain
does not.

## What changes

- From under the sea, a ray's rain starts where the ray leaves the water: at
  its exit through the sea sphere, on the refracted direction, and only
  inside Snell's window. A ray past the critical angle (total internal
  reflection), a ray that never rises, or a ray that meets something in the
  water first gathers no rain.
- What it gathers is laid over the frame weighed by what reaches the eye
  through the surface: the Fresnel transmission at the exit times the water's
  absorption over the path from the eye to the surface. The rain seen through
  the window is faint and blue-shifted, never a fog over the water.
- Above the sea nothing changes: the rain already stops at the sea sphere and
  at the scene's depth.

## Not in this change

- Snell's window and the underwater look: with the rain volume off they are
  what the owner calls fine. Seen at the window's edge the sheet's rain rings
  tilt the normal across the window's smoothstep, which prints the rings as
  dark lines over what is seen through the surface (a hull above it in the
  shots); that is the sheet's own term, there with the volume off, and is
  reported, not fixed here.
- The straddling state (the eye within 0.8 m of the surface) keeps today's
  rain.
- A launch with `--rain 1` by day panics (`min > max, or either was NaN` in
  an f64 clamp) at this place; found while capturing, not touched here.
