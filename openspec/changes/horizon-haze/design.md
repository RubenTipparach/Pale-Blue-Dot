# Design: horizon-haze

## 1. The sky's air

`sky_atmosphere.wgsl`'s `air_density` is
`exp(-altitude / H) * (1 - smoothstep(0.72, 1.0, altitude))`, with `altitude`
as a share of the shell (960 m) and `H` = `SkyParameters.atmosphere.y`, set at
the spawn in `sky.rs` (0.22). The change is that one number, to the value the
owner picks from the stills (0.35, 0.5 or 0.7). The shell, the taper and the
day veil's fade (288-960 m) are unchanged, so the air ends where it does
today and the stars come out at the same height.

## 2. Holding the zenith

The air straight up from the sea, in shell units, is
`D(H) = integral over 0..1 of exp(-x / H) * taper(x) dx`:

| H | m | D(H) | x today | from 421 m, x today |
| ---: | ---: | ---: | ---: | ---: |
| 0.22 | 211 | 0.215 | 1.00 | 1.00 |
| 0.35 | 336 | 0.320 | 1.48 | 2.74 |
| 0.5 | 480 | 0.410 | 1.90 | 4.64 |
| 0.7 | 672 | 0.494 | 2.29 | 6.64 |

`CLEAR_RAYLEIGH` and `CLEAR_MIE` are divided by `D(H) / D(0.22)`. That ratio
is written beside the scale height in `sky.rs`, with a test that computes it,
so the two cannot drift apart. The overcast's fractions (`overcast_scatter`)
apply to the new clear values as before. From the ground, straight up, the
optical depth, and so the colour, is today's. Toward the horizon, and from
inside the layer and above it, the air is deeper than today's by the rest of
the ratio.

## 3. Clouds

Unchanged. `cloud_air` fades a cloud into what is behind it, and what is
behind a far deck from inside the layer is now the brighter sky.

## Verification

- Stills at the owner's ring frame (`--route clouds --frames 1300`, clouds
  on and off), the ground at noon, 1500 m (`--view column --height 1500
  --pitch -32`), orbit, and the deck at dusk (`--time 20.5`), for each of the
  three depths with the zenith held. The ground's near-zenith pixel within a
  few levels of today's.
- `cloud_haze` 1, 2 and 3 on the chosen depth.
- A unit test that the scattering's divisor is `D(H) / D(0.22)` from the
  shader's own density.
- The perf suite is not needed for a changed constant; the sky pass does the
  same work.
