# Proposal: the planet turns, and the whole sky turns with it

## Why

The owner: "the day night cycle is wrong, the planet is rotating, the sun
shouldn't orbit around the planet, there should also be a sun in the sky. In
order to have real day/night cycles (48 minute cycles) I should see the sun,
stars, and moon move across the sky as if the planet was rotating."

What is here today is half of that. `daylight::Clock` sweeps ONE direction -
the sun's - round a tilted axis every six minutes, and the terrain, the sky
shell, the key light and the water all read it. Nothing else in the sky moves:
the star quads in `desktop/scene.rs` are spawned once at 180 km and stay where
they were put, the moon rides its own on-rails orbit in the same fixed frame,
and there is no sun disc at all, only the scattering's brightening where the
sun direction is. So from the ground the sun crosses a sky whose stars stand
still, which is a sun going round the planet, and from orbit the terminator
sweeps a globe that never turns.

## What

Tenebris's model, which its `orbit.rs` states in one sentence: keep the active
body pinned at its render centre and rotate the whole SKY - the star, every
other body, the star field - by the body's spin. From the surface it is
indistinguishable from the planet turning and it costs one transform.

- **One rotation.** `daylight::Spin` is a quaternion about the planet's tilted
  pole, a full turn per day. Everything in the sky is a FIXED direction in the
  system frame carried into the body frame by its inverse: the sun, the star
  quads, the moon's orbit. The sun direction the terrain reads is that same
  product, so nothing downstream changes.
- **A day is 48 minutes.** `DAY_S` 360 to 2880. It is the one number, and it
  is what the owner asked for.
- **A sun in the sky.** A disc drawn by the sky shell where the view direction
  is within the sun's angular radius, with a limb and a glow, from the same
  direction the scattering already takes. Tenebris draws its disc as an
  angular-size-correct billboard at a distance inside the far plane; the shell
  is that surface here.
- **The stars and the moon turn.** The star mesh gets the spin's inverse as
  its rotation each frame; the moon's orbit position is rotated by it before
  it is placed.

## What this is NOT

- **Not the rotating body frame for flight.** A ship in orbit still lives in
  the planet-fixed frame, so it co-rotates, as Tenebris's does. At a 48-minute
  period at low orbit that is a visible error in the long run and a second
  phase: the body's spin becomes part of the frame the ship's poses are
  rebased through, per the world invariants. This change is the sky.
- **Not the moon's own phases or the sun's granulation.** A lit disc and a
  bright one.
