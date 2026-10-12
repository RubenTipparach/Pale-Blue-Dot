# Proposal: a weather FIELD, and clouds with a thickness

## Why

**The owner asked for thicker clouds and a rain system like Tenebris's.** Both
are real gaps, and each has a different cause.

**The clouds have no thickness because they are one sphere.**
`sky_atmosphere.wgsl` takes a single ray-sphere hit at `CLOUD_RADIUS`, samples
two octaves of value noise at the hit point and blends a flat colour over the
sky. One sample, one depth, no interior: whatever the noise says, the result is
a stencil painted on a shell. That is why they read as wisps in every capture -
there is nothing in them to be lit from one side, nothing to occlude anything,
and no silhouette from below.

**The rain is one global number, and the file says so.** `weather.rs`'s own
module comment: *"Rain is global until a cloud field shared with the sky shader
exists; see `openspec/changes/weather-rain`."* `Weather.rain` is cycled by the P
key and applies to the entire planet at once. You cannot walk out of a storm,
because there is no storm - there is a switch.

## What Tenebris has

`tenebris-core/src/weather.rs` and `assets/config/weather.yaml`. Rain there is
not a state anything keeps. It is a **deterministic field of (seed, surface
direction, time)**:

- **`solar`** - a drifting simplex field of warm pockets over the sphere, two
  octaves, scrolled in time.
- **`moisture_source`** - the biome moisture map: deserts low, wet biomes high.
- **`cloud_density = (moisture * heat) ^ arid_gamma`**, where `heat` is
  `solar_floor + (1 - solar_floor) * solar`. The aridity gamma clears the dry
  end harder, so deserts rarely cloud and rarely rain.
- **`cloud_cell`** turns that into what the renderer needs: `cover` (a
  smoothstep between `cloud_min` and `cloud_full`), `alpha` (small clouds
  transparent, big ones opaque), and `raining`.
- **Rain needs a solid cloud overhead**, `cover >= rain_cover_min`, and it
  **trails** the cloud: a column that was that cloudy `rain_min_s` ago still
  counts as raining, sampled from the same field at `t - rain_min_s`, so it
  stays stateless and every client agrees without syncing anything.
- **`precip_kind`** is snow over tundra and mountains, rain everywhere else.

The whole lifecycle a player sees - a puff forming, growing, darkening, raining,
drifting away - is that field drifting over the world. Nothing is spawned and
nothing is stored.

**And its clouds are geometry**: puffs on a shell 60 m up, placed one per 30 m
cell by a density hash, each a cluster of lumps, LODing to a slab past 230 m and
to a texture shell past 5 km, gated by a weather-system patch field.

## What this change does, and what it deliberately does not

**The field ports, and it is mostly already here.** `pbd_core::planet_gen`
already carries `moisture` and `biome` from the terrain port, which is the
expensive half of the reference's field. What is missing is the drifting warmth
and the ramps over it. `pbd_core::weather` is that: `solar`, `cloud_density`,
`cloud_cell`, `precip_kind`, pure functions of seed, direction and time, in
`pbd-core` because they are engine-independent rules and a future multiplayer
must agree about the weather without sending it.

**Every consumer of rain is untouched, which is the point.** The cap's ripples,
the terrain's wet sheet and rivulets, the lens droplets, the shower and the
wetness lag all read `Weather.rain` already. This change replaces where that
number COMES FROM - a sample of the field under the player instead of a global
switch - and none of them learns anything new. That is the one-code-path rule
paying a dividend it was owed.

**The clouds get a thickness, and it is a slab rather than geometry.** The sky
shader already marches the atmosphere; clouds become a second march between an
inner and an outer cloud radius, accumulating density from three octaves and
lighting each step toward the sun. A ray that enters the slab at a grazing angle
crosses more of it than one going straight up, which is the whole of what makes
a cloud look like a mass rather than a decal, and it is free here because the
march already exists.

Porting the reference's PUFF GEOMETRY is deliberately not done. It is a
per-puff placement grid, a lump cluster, two LOD tiers and a texture shell -
its own change, and on this architecture it would be a fifth indirect draw
rather than a CPU mesh. The slab is what this renderer is shaped for.

**The clouds and the rain agree overhead, and not yet at distance.** The sky
shader cannot evaluate `planet_gen::moisture`, which is fBm on the CPU. So the
app samples the field at the player and hands the shader its `cover`: the sky
thickens and darkens as a front arrives over you, and it is raining when it is
overcast, which is the coupling a player can actually perceive. A cloud fifty
kilometres away is still the shader's own noise rather than the field's. Making
that exact means porting the moisture fBm into WGSL and pinning the two against
each other; it is named in the tasks rather than smuggled in.

## Success

Standing on the ground, the sky is sometimes clear and sometimes a thick
overcast with a lit top and a dark underside, and it rains when and only when it
is overcast over you. Walking far enough leaves the rain behind. The same
direction and time give the same weather on every run, with nothing saved.
