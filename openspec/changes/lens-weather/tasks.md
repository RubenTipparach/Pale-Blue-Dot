# Tasks

- [x] 1. `rain_lens_scale` (config, validation, `.ron`), the drop space scaled and the refraction divided by it
- [x] 2. The mist probe pass and its 1x1 pair; `dt` and airspeed to the GPU; the lens window
- [x] 3. The mist's look in `lens()`: mask, beads, film, wiped by the drops
- [x] 4. Stills of the drops at scale 1 and 2 (half the size, four times as many); a cloud-hop recording (the lens mists in the cloud and clears over about a second after, edges first); tests; the perf suite. The first build's target (the density itself) barely misted: it now saturates at a thin cloud's density
