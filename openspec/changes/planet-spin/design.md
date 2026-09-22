# Design: one spin, everything in the sky derived from it

## The frame

The planet's pole is the tilted axis `daylight::TILT` already tilts the sun's
arc about. `Spin::at(clock)` is the quaternion that turns about that pole by
`fraction * TAU`; at fraction 0.5 (noon) the body's noon meridian faces the
sun. The system frame is what the sky is fixed in: the sun at `SUN_FIXED` - the old
`SUN_DIRECTION`'s azimuth at the tilt's latitude, because the old constant's
own latitude was 45 degrees, a sun that never sets north of it - the star
field where `scene.rs` spawned it, the moon on its orbit.

Body frame = `spin.inverse() * system frame`. `Clock::sun()` becomes exactly
that product applied to `SUN_FIXED`, which is what it already computes by
hand; the test that it is a unit vector at every hour stays, and a new one
holds `Clock::sun()` equal to `Spin::at(clock).inverse() * SUN_FIXED` so the
two can never disagree.

## What turns, and where

| thing | today | with the spin |
| --- | --- | --- |
| sun direction | `Clock::sun()` | unchanged in value, derived from `Spin` |
| key light, terrain, water, sky scatter | read `Sun::direction()` | unchanged |
| star quads | fixed transform | `Transform::rotation = spin.inverse()` each frame |
| moon | `orbit.sample(t)` in the fixed frame | the same position rotated by `spin.inverse()` |
| sun disc | none | drawn by the sky shell along `Sun::direction()` |

The star and moon updates are one system beside `follow_sun`, reading the
same `Sun` resource, so the sky cannot come apart: a frame in which the sun
turned and the stars did not is not expressible.

## The disc

In `sky_atmosphere.wgsl`, after the scattering: `let cosine = dot(direction,
sun)`; inside `cos(SUN_ANGULAR_RADIUS)` the disc, a limb-darkened white-gold,
over a `pow` glow that reaches a few radii. It is added before the day veil
and after the clouds so the clouds occlude it, and it is multiplied by the
same `sun_visibility` the scattering uses so the planet's own shadow hides it
at night. The angular radius is the Sun's from Earth, 0.27 degrees, widened
to 0.6 for legibility at the shell's resolution, as a named constant beside
the other art-directed numbers, with the `per-body-rendering` change the one
that lifts them into data.

## The number

`DAY_S = 2880.0`. Twenty-four hours in forty-eight minutes: a minute of play
is half an hour of world. `START_HOUR` stays at nine. The capture harness's
`--time` still pins the clock, which is what keeps every screenshot the same
hour.

`--yaw <degrees>` turns the walker's starting heading to the right of the
default, beside `--pitch`, and with `--time` the launch log says where the
sun stands from the spawn as the yaw and pitch that would centre it. That is
Tenebris's own lesson about photographing the sky: solve the aim off the
thing being photographed rather than guess it. At the default spawn the sun
rises at about yaw -160 and sets in front of the default heading, passing
nearly overhead at fourteen hours, so one fixed aim cannot hold both ends of
the day and the captures are two fixed aims from the one spot: the sunset
aim (`--yaw 0 --pitch 40`) holds sixteen to twenty hours as the disc walks
down the frame, and the sunrise aim (`--yaw -165 --pitch 30`) holds eight to
ten as it climbs on the other side.

## Held: the rotating body frame

For the walker nothing changes: the body frame IS the walker's frame. For a
ship, the honest model is that the planet's spin is part of the frame the
ship's Newtonian state is rebased through at a tick boundary, so that a ship
hanging over one spot sees the ground turn under it. That is the world
invariant on rotating body-following frames, listed in
`docs/source-migration.md` as not yet ported, and it is a change to the
flight and physics frames rather than to the sky. Tenebris does not do it
either; its comment says a body's spin is only observable from its surface.
