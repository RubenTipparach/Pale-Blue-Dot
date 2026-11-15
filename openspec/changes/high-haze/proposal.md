# Proposal: no milky veil when the eye leaves the atmosphere shell

## Why

The owner (2026-11-14), two screenshots from a daytime climb in flight: "800-900
m transition is rough" and "not sure why theres like super fog on the planet".
At 784.7 m above the ground the land below is crisp: hex biomes, the coast, a
ship. At 992.1 m the whole planet is under a pale blue-grey veil and the land
barely reads; only the deep sea at the limb stays strongly blue. This is
`CLAUDE.md` priority 5 ("the ground-to-space transition must not be jarring").

## What was measured

Fixed-step stills, `--view column --at 16.86 57.63 --pitch -42 --time 11
--rain 0 --weather-at 3600` (clear: the log says `cover 0.00, rain 0.00`), the
ground there 76 m above the sea, fast build, the M4 Max Mac, 2880 x 1800.
A ground patch (the lower middle of the frame, clear of the HUD) measured
in sRGB:

| Above ground | Above the sea | Mean R, G, B | Blue's spread |
| ---: | ---: | --- | ---: |
| 600 m | 676 m | 92, 114, 90 | 33 |
| 800 m | 876 m | 109, 129, 101 | 32 |
| 850 m | 926 m | 107, 130, 101 | 32 |
| 900 m | 976 m | 111, 147, 164 | 11 |
| 1000 m | 1076 m | 105, 145, 161 | 13 |
| 1000 m, `PBD_NO_SKY` | 1076 m | 98, 129, 97 | 31 |
| 1000 m, `PBD_NO_CLOUDS` | 1076 m | 105, 145, 161 | 13 |

Between 850 and 900 m the ground's blue jumps by 63 levels and its texture
(the spread) falls to a third. With the atmosphere shell left unspawned the
veil is gone; with the clouds off it is unchanged. The clouds, the rain and
the ground's own haze are not it.

## What causes it

The sky is a mesh, a sphere of `ATMOSPHERE_RADIUS` = 1.2 R, which is **960 m
above the sea** on this 4800 m planet (`sky.rs`). `sky_atmosphere.wgsl` draws
its back faces while the eye is inside it and its front faces once the eye is
outside, with the depth test `GreaterEqual` and no depth write. From inside,
the back face over the ground lies beyond the ground and fails the depth test,
so over the ground the dome draws nothing and the ground's own haze (`finish`
in `planet_surface.wgsl`, `distance_fog` in `water.wgsl`) is all there is. From
outside, the front face lies in front of everything, so the dome lays its whole
column of in-scattered light, premultiplied, over the ground.

That column is the veil. Its opacity is small (about 0.07 straight down), but
its in-scattered light is the sky's: the Rayleigh phase is
`0.75 (1 + cos^2)`, not normalised by 4 pi, and the sun's radiance is 3.2,
both tuned for how bright the dome looks as sky. Laid over sunlit ground it adds
several times the light a sky-coloured haze of that opacity would, a flat
blue-white wash. It appears in the one frame the eye crosses 960 m above the
sea, 884 m above the owner's ground, which is the owner's "800-900 m".

## What changes

- Over the ground, seen from outside the shell, the dome's in-scatter is
  taken at the haze it physically is: the Rayleigh phase normalised, so the
  colour it hazes toward is the sky's, not several times it, meeting the sky's
  own brightness only toward the limb where the path is long.
- The dome takes over the ground's air as the ground's haze gives it up:
  weighted by `1 - exp(-(h - top) / 1050 m)`, zero at the shell and rising
  with the same scale height the ground's haze (`air = exp(-h / 1050 m)`)
  falls with. Nothing switches on in one frame.
- The sky above the horizon, from anywhere, is unchanged; so are the clouds,
  the ground's own haze and the shell's radius.

## Impact

- `assets/shaders/sky_atmosphere.wgsl` only. No uniform, binding or pipeline
  change; the sky pass does the same sixteen steps.
- Saves: none. Generation: none.
- Frame time: not measured (a few ALU per sky fragment, the same work). The
  perf suite is the desktop's; this is a look change.
