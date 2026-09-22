# Proposal: a storm that looks like one

## Why

**The owner's words: "can you improve the cloud sim? right now it looks pretty
faint, we are also missing rain, and I'd like to improve the wet rain shaders on
the ground as well, right now it looks like a well lit warehouse with rain drops
on the ground."**

Captured on the shipped binary at the spawn, 09:00, `--fixed-dt`, frame 150
(`docs/screenshots/weather-before-*.png`), and measured off those frames:

| Frame | Measure | Value |
| --- | --- | ---: |
| Ground, clear | mean luminance | 101.9 |
| Ground, `--rain 1` | mean luminance | 73.9 |
| Ground, rain over clear | ratio | **0.725** |
| Ground, rain over clear | blue channel ratio | **0.37** |
| Sky, clear | share of the sky that is grey (cloud) | **0.0%** |
| Sky, `--rain 1` | share of the sky that is grey | 94.1% |

Each of the owner's three complaints is one of those rows.

**The warehouse is a full sun under a grey sky.** The ground in rain is 0.725
of the ground in sun, and 0.72 is `rain_wet_darken`: the whole difference is
the wet surface's own darkening. The LIGHT never changed. `planet_surface.wgsl`
computes the sun term as `max(dot(n, sun), 0) * daylight` and the fill without
any reference to cloud, so a storm is lit exactly like noon: hard shadows,
bright faces, under a sky that is 94% grey. Tenebris does not do this. Its
`weather.yaml` dims the sun by `overcast_sun_dim` (0.72) and the ambient by
`overcast_amb_dim` (0.48) at full overcast, and greys the sky dome to match,
"because without them the ground went stormy while the sky stayed clear blue".
Our port took the rain knobs and left that block behind.

**The raindrops on the ground only ever darken.** The wet sheen is
`ambient * (clamp(dot(wet_n, up)) - clamp(dot(n, up)))`. On an upward face `n`
is `up`, so the second term is 1 and the first is at most 1: the difference is
never positive, and every ring the ripple normal draws SUBTRACTS a blue sky
colour from the ground. That is the blue channel falling to 0.37 while red and
green fall to 0.73, and it is why the rings read as black lines printed on a
floor rather than as water. They are also on every upward face alike, grass
included, so a meadow in rain is a tiled floor of identical rings. A direct-sun
glint adds on top at full strength, because nothing tells it the sun is behind
a cloud.

**The clouds are there and almost never visible.** The field puts 0.18 cover
over an average place. The slab's threshold is `mix(0.72, 0.20, cover)`, so an
average sky thresholds a three-octave value noise at 0.63, which a noise with a
mean of 0.5, multiplied by a vertical profile that is under one everywhere but
the middle third, almost never reaches. Measured: no pixel of the clear sky at
the spawn is grey.

**Rain stops eighteen metres away.** The shower is 1,700 streaks on an 18 m disc
around the camera and nothing beyond it. Tenebris draws every raining cell in
view, and past `rain_detail_range_m` (150 m) one translucent grey curtain per
cell, so a storm is something you can see coming and fly out of. Ours cannot
be seen from anywhere but inside it. And the lens drops are not gated by
altitude: `weather-before-coast-rain.png` is 420 m up with rain running down
the glass, where Tenebris stops at `rain_lod_alt_m` (200 m).

## What

Four parts, in the order of what they buy.

1. **Overcast light.** Tenebris's overcast block, ported with its shipped
   values: the sun and the fill on the ground dimmed by the cover over the
   player, the sky dome greyed and darkened to match, the distance haze
   thickened in cloud and in rain, the water sheet's sun with them. One
   number drives all of it, the `cover` the field already computes and the
   clouds already read.
2. **Wet ground that is water.** The sheen becomes a Fresnel reflection of the
   sky along the rippled normal, so a ring is the reflection bending, bright
   on one side and dark on the other, and a puddle at a grazing angle mirrors
   the grey sky. Rings and the mirror only where water stands: flat faces,
   under a puddle mask, and not on grass, which darkens and takes a light sheen.
   No sun glint under cloud.
3. **Clouds that read.** A lower base threshold so fair weather has clouds, a
   darker base under a storm, and the cloud numbers out of Rust constants into
   `weather.ron`. Measured against a target share of the sky at four covers.
4. **Rain that can be seen.** The near streaks out to the detail range at the
   reference's density, far curtains over every raining cell in view, and the
   lens drops stopped above 200 m.

## Measured before anything moves

The table above is the before. Every part carries its own after, taken the same
way off the same frames: the ground's light ratio and blue ratio in rain, the
sky's cloud share at 0, 0.5 and 1 forcing, and the rain visible from 400 m
outside a storm.
