# Proposal: the sea reads as a bright flat sheet, and the knobs are not the cause

## Why

The owner's report is "water is way too shiny", for the second time. The first
time it was answered by tuning: the reflection colours, the Fresnel horizon
floor and the specular intensity were all authored downward in `water.ron`, and
`detail_fade` was added so the sheet stops sparkling with distance. That is the
wrong lever to reach for again, and the measurement says so.

**Every shine knob this project has is already at or below the Tenebris value
it was ported from**, and two of them are at half:

| knob | Tenebris `water.yaml` | ours `water.ron` |
| --- | ---: | ---: |
| `specular_intensity` | 0.30 | **0.12** |
| `specular_power` | 140 | 140 |
| `sun_tint` | 1.35, 1.25, 1.10 | same |
| `sky_horizon_strength` | 0.50 | **0.35** |
| `sky_horizon_color` | 0.85, 0.92, 0.98 | **0.46, 0.60, 0.74** |
| `sky_zenith_color` | 0.35, 0.55, 0.85 | **0.18, 0.34, 0.62** |
| `foam_intensity` | 0.10 | 0.10 |
| `wave_steepness` | 0.65 | 0.65 |
| `slope_max` | 1.6 | 1.6 |
| `absorption` | 0.60, 0.20, 0.10 | same |
| `deep_color` | 0.02, 0.10, 0.22 | same |

So the sheet is drawn with a reflection colour a little over **half** the
brightness of the reference's, a grazing Fresnel floor **30% lower**, and a
specular **60% lower**, and it still reads as shinier than the reference. A
knob that has already been halved without fixing the complaint is not the
knob. Turning it down again lands somewhere with no water in it at all.

Two things differ structurally, and neither is a tunable.

## The two real differences

### 1. Tenebris does not tone-map. We do.

There is no tone mapping, exposure or gamma step anywhere in the Tenebris
renderer: its composite ends on a bare `frag_color = vec4(color, 1.0)` and the
framebuffer saturates at one. Its water values are authored **for that**, which
is why its sky horizon reflection is near-white (0.85, 0.92, 0.98) and its
specular peak is 0.405 without looking blown out: the clip is the look.

Every camera here carries `Tonemapping::TonyMcMapface`, a filmic curve that
lifts the mid-tones and desaturates toward white as a value rises. A sheet at
roughly 0.39 linear, which is what the current values produce at a grazing
view, lands as a pale desaturated grey-blue rather than as a saturated one, and
the sea's own colour is what the curve takes out first.

Measured on the shipped `shore` capture: the sea is **brighter than the sky it
reflects** and much less saturated.

| region | mean sRGB | saturation |
| --- | --- | ---: |
| sky | 107, 144, 147 | 41 |
| sea, far | 124, 149, 140 | 25 |
| sea, near | 130, 151, 137 | 20 |

A sea whose red channel reads **17 higher** than the sky's is not reflecting
that sky. Nothing is clipping: the brightest pixel in the sheet is 174 of 255
and not one pixel of it passes 200. This is not a highlight problem at all.
It is a sheet sitting at a flat, desaturated mid-value over its whole area,
which is what "shiny" describes when there is no bright spot to point at.

### 2. The rescale left a one-metre sand shelf out to the horizon.

The transmitted half of the mix is the seabed seen through the water, and after
the rescale there is almost no water in front of it. Measured along the
`shore` capture's own walk east from 72 N:

| distance from the waterline | seabed |
| ---: | ---: |
| 3 m | -1 m |
| 45 m | -1 m |
| 91 m | -2 m |
| 363 m | -3 m |
| 725 m | -7 m |

From an eye 1.6 m up on a 4,800 m body the sea horizon is **124 m** away, so
**every pixel of sea in that frame is one or two metres deep**, over sand. One
metre of water removes 45% of the red and 10% of the blue, so what the sheet
transmits is very nearly the sand itself, and the pale warm grey that comes out
is a correct rendering of a shallow sand flat. It is not what the owner means
by water, and it is a consequence of the ocean relief being compressed by 0.12
in the rescale while the land was compressed by 0.17.

This is the same defect as the `dive` preset photographing the inside of a
rock, one commit earlier: a number that silently depended on the elevation
step. The ocean depth had no reason to be cut as hard as the land relief. The
land was cut so a walker could climb a mountain; nobody walks up the sea floor.

## What changes

1. **Deepen the sea near the shore** so that the water a player looks across is
   water. `OCEAN_RELIEF` rises from 0.12 toward the land's own 0.17 or above,
   with the shelf profile measured rather than assumed, and the relief test
   re-pinned to the new floor.
2. **Author the water for the tone mapper this project actually has**, rather
   than for one it does not: state in the design that the values divergE from
   Tenebris's on purpose and why, so the next reader does not "restore parity"
   by copying the reference numbers back in.
3. **Do not lower the shine knobs further.** They are already below the
   reference. If anything they come back up once the sea has depth under it.

## What this does not change

The wave shape, the foam, the refraction, the absorption coefficients and the
underwater view are all faithful ports and are not in question. The complaint
is about the colour and flatness of the sheet seen from above.
