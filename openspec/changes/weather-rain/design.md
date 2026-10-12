# Design: one number, four consumers

## The field

`Weather { rain: f32, wetness: f32 }`, a main-world resource extracted to the
render world. `rain` is what the sky is doing now; `wetness` is what the ground
remembers, `wetness += (rain - wetness) * (1 - exp(-dt / wet_fade_tau_s))`, so
puddles outlast a shower. The launch argument sets `rain`; the key cycles it
through 0, 0.5, 1.

## The cap: rain ripples

`rain_ripple_grad` from `water.fs.glsl` lines 170-201, a 3x3 kernel of
expanding rings hashed per cell, at `WATER_RAIN_RIPPLE_SCALE` 3 cells/m and
strength 6, added into the fbm gradient before the slope cap. The slope cap
matters: Tenebris's comment records that the kernel spikes the gradient to
eight times the waves and, uncapped, flipped the whole sheet into a mirror.
The cap is already in the port.

## The terrain: the wetness block

`hex.fs.glsl`'s `wet_amt` branch, ported term for term into
`planet_surface.wgsl`, with the thirteen `rain_*` knobs in a `Rain` uniform
block (four vec4). Gates: `wet_amt = wetness * skylight * above_water`, where
`above_water` is one for a fragment at or above the sea radius, so the seabed
gets no rings. Up-faces get the two-octave wet sheet plus impact rings on a
triplanar UV; side faces and trunks get the rivulet kernel in the face's own
texture UV; both darken by `rain_wet_darken`, add a sky sheen on the tilt of
the wet normal toward up, and a sun glint to `rain_glint_power`.

This is the first time `planet_surface.wgsl` reads a second uniform block. The
existing 112-byte `Params` grows to carry the weather vector (wetness, rain,
spare, spare) and the pinned size test moves with it.

## Precipitation

`weather_fx.rs`'s near shower, on the CPU: up to `MAX_MARKS` 1,700 streaks at
full intensity on a disk of `DISK_R_M` 18 m around the camera, each hashed to
an angle, radius and phase, falling at `rain_fall_mps` through `COLUMN_H_M`
46 m and landing on `terrain_radius` (which is already "surface or sea,
whichever is higher"). Built as one `Mesh` of camera-facing quads with vertex
colours, unlit and alpha blended, positions in the render frame (the streaks
are near the camera, so f32 is fine). Skipped when the camera is under water or
below the terrain under it.

The distant storm shafts are not built: they iterate the cloud cells, and
there is no cloud field.

## What is Tenebris's and what is not

The kernels (Zavie's rings, the rivulets, Heartfelt's drops) keep their inner
constants inline, as Tenebris does: those numbers are the effect's identity.
Everything a designer would turn is in `weather.ron`.
