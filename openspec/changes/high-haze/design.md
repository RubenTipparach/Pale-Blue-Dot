# Design: high-haze

## 1. Two hazes, one handed to the other

Over the ground there are two models of the same air.

- **The ground's haze**, `finish` (`planet_surface.wgsl`) and `distance_fog`
  (`water.wgsl`): `fog = (1 - exp(-d * 0.00036)) * air * 0.55` toward the sky
  colour `ground_sky`, with `air = exp(-h / 1050)` for the eye's height `h`
  above the sea. It thins as the eye climbs.
- **The dome's column**, `sky_atmosphere.wgsl`: drawn over the ground only from
  outside the shell (proposal, "What causes it").

At the shell (`top` = 960 m) the ground's haze still has `air` = 0.40 of its
strength, and the dome's column switched in at full strength on top of it.
The dome over the ground is weighted by

```text
w = 1 - exp(-(h - top) / 1050)
```

zero at the shell, 0.38 at 1.5 km, 0.87 at 3 km, 1 from orbit: the dome takes
up the air the ground's haze drops, at the same scale height. Above the
horizon, `w` is not applied: that is the sky.

## 2. The column's light over the ground

The dome's scattering is
`integrated * (beta_r * 0.75 (1 + cos^2) + beta_m * mie_phase) * sun.w`. The
Mie phase carries its 1 / 4 pi (the `12.566` in its denominator), the Rayleigh
phase does not: the sky's brightness was tuned with it so. The colour a haze
of opacity `a` tends to is `S / a`; with the unnormalised phase that colour is
several times the horizon sky's, so a column of opacity 0.07 lit the ground
like a sheet of sky.

Over the ground the Rayleigh term is divided by `4 pi` (the normalised phase,
`3 / 16 pi (1 + cos^2)`), so the haze tends to the sky's colour. Toward the
limb the ground's path through the air is long and the opacity nears its 0.94
cap; there the ground's haze has to meet the sky just above the limb, which is
the unnormalised light. The gain blends between the two with the opacity:

```text
g = mix(1 / (4 pi), 1, smoothstep(0.3, 0.94, opacity))
```

so near the nadir the ground keeps its colours under a thin sky-coloured haze,
and at the limb the ground fades into the same blue the sky above it is. The
exact smoothstep range is chosen from stills (tasks 3), from orbit as well as
at 1-3 km.

## 3. What does not change

The shell's radius, the scale height, the scattering constants and the sky
above the horizon from every height; the ground's own haze; the clouds and
their air (`cloud_air`); the day veil and the stars.

## Verification

- The ladder at the owner's spot, 600-2000 m above the ground plus 3000 m and
  9000 m (orbit), before and after, as a contact sheet in
  `docs/screenshots/high-haze/`, with the ground patch's mean and spread per
  rung: no step between consecutive rungs larger than the steps below the
  shell.
- `PBD_NO_SKY` at 1000 m as the reference for the ground's own colour.
- The shader validated by the build (Bevy compiles the material at load; the
  capture logs no shader error).
