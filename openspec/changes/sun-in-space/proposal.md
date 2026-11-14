# Proposal: the sun is drawn from space

## Why

The owner (2026-11-14), a screenshot from orbit: "why did the sun dissapear?
when I go to space". With the sun just above the planet's limb, only the lower
half of its disc shows, cut off flat along the atmosphere's outer edge; above
that edge there is black space and stars where the rest of the sun should be.

## What causes it

The sun's disc and its glow are not an object. They are a term of the
atmosphere dome's fragment shader, `assets/shaders/sky_atmosphere.wgsl`
(`disc`, `limb`, `glow`, `sun_light`, `opacity = max(opacity, disc * shadowed)`).
The dome is a sphere mesh of radius `ATMOSPHERE_RADIUS` = 1.2 R, 960 m above the
sea (`crates/pbd-app/src/sky.rs`), and the comment there says so: "the shell is
that surface here".

- From inside the shell its back faces surround the eye: every direction has a
  dome fragment, so the sun always has a pixel to be drawn in.
- From outside, only the directions whose ray meets the shell have a fragment.
  The shell's disc, seen from 6 km up, reaches 57.8 degrees below the horizontal;
  the planet's limb is at 63.6. A sun between the two lies behind the shell and
  is drawn. A sun higher than the shell's rim has no fragment at all, and is
  not drawn. A sun across the rim is cut along it, flat: the owner's picture.

Measured in `docs/screenshots/sun-in-space/` ("before"): from 6 km at the
equator, the sun at 60.6 degrees under the horizontal (behind the air) is
whole; at 57.8 (on the rim) it is cut in half along the shell's edge; at 55.7
(above the rim) it is gone.

The moon is not affected: it is a lit sphere mesh (`scene.rs`), drawn wherever
it is. The stars are their own mesh at 180 km. Only the sun is drawn by the
dome.

## What changes

- The sun's disc and its near glow are drawn from outside the atmosphere by a
  sun of their own: a quad facing the camera, 150 km out along the sun's
  direction, sized in angle, with the same disc, limb darkening and glow the
  dome draws. It is depth tested like the stars, so the planet (and the moon)
  hide it.
- Seen through the air from outside, it is dimmed and reddened by the
  transmittance along its ray through the shell, the dome's own extinction
  (`beta_r + beta_m` times the optical depth), faded in over the same 200 m
  past the shell's top that `high-haze` uses, so nothing changes in the frame
  the eye crosses the shell.
- From inside the shell the dome draws the sun exactly as today; the new sun
  draws nothing there. From outside, the dome stops drawing the sun, and the
  new sun is the only one: no doubled glow along the shell's rim.
- Clouds still hide it (the clouds pass draws after both); the frame's
  exposure and bloom see the same radiance as before (`SUN_COLOUR * 3.2`).

## Impact

- `assets/shaders/sky_atmosphere.wgsl`: the sun's term moves into a function
  both draws call; the dome gates it to the inside; a `SUN_DISC` variant of the
  file (vertex and fragment) for the new sun. One file, so the air's density,
  the extinction and the disc have one implementation.
- `crates/pbd-app/src/sky.rs`: a `SunDiscMaterial` (the dome's parameters,
  copied from the dome's material each frame), its quad entity, sorted to draw
  just after the dome. Always visible, so its pipeline is built while the
  loading screen is up (no `LATE_PIPELINE`).
- Saves: none. Generation: none.
- Frame time: not measured (one 12-degree quad, discarded from inside the
  shell by a collapsed vertex position). The perf suite is the desktop's.
