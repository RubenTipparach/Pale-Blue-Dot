# Design: the field in the core, the slab in the sky

## `pbd_core::weather`, and why it goes there

Pure functions of seed, direction and time, depending on nothing but
`planet_gen` and `std`:

```rust
pub fn solar(seed: u64, direction: Vec3, seconds: f32) -> f32
pub fn cloud_density(cfg: &WeatherField, terrain: &TerrainConfig, d: Vec3, t: f32) -> f32
pub fn cloud_cell(cfg, terrain, d, t) -> CloudCell   // cover, alpha, raining, precip
pub fn precip_kind(terrain: &TerrainConfig, d: Vec3) -> Precip
```

It belongs in the core for the reason the architecture rules give: a future
multiplayer has to agree about the weather, and a field that is a pure function
of position and time is agreed on without a single byte being sent. It is also
the only way `raining` can trail a cloud without storing anything - the trail is
a second sample of the same field at `t - rain_min_s`.

**The noise is this project's own gradient noise, not the reference's simplex,
and that is deliberate.** `pbd_core::planet_gen` carries `gnoise3d_seed` pinned
bit for bit against Tenebris, and the terrain is built on it. Adding a simplex
implementation beside it would be a second noise basis in one crate for one
caller. What the solar field needs is a low-frequency drifting blob field, which
both bases give; the exact values differ and could never match anyway, since the
moisture they multiply is sampled on a body sixteen times the reference's
radius. The reference's own comment says the field's SHAPE is the point.

**`moisture` is planet-scale and stays that way.** The terrain change split
every field into planet-scale (a unit-sphere frequency, so a world has the same
handful of continents whatever its radius) and land-scale (a size in metres).
Moisture is land-scale at 188 m, which is right for deciding whether a hillside
is jungle or fields. A weather system is not 188 m across. So the field samples
moisture at its own coarse scale for weather - one more call to the same fBm at
a planet-scale frequency, named `weather_moisture_scale` - and the biome's own
moisture keeps deciding biomes. Two questions, one function, two frequencies.

## What the app does with it

`pbd-app`'s `weather.rs` keeps its two numbers and gains a source:

```rust
let here = surface_direction_of(camera);
let cell = weather::cloud_cell(&field, &TERRAIN, here, clock);
weather.rain = if cell.raining { cell.cover } else { 0.0 };
weather.cover = cell.cover;
```

`wetness` still chases `rain` on its e-fold, so the ground still dries slowly
after a front passes - which now happens because the front MOVED, not because a
key was pressed. Every other consumer is untouched.

**P keeps working and becomes the reference's own knob.** `moisture_boost` in
the reference lerps every cell toward saturation, which is how its weather menu
forces a storm without a second code path. Here P cycles that boost rather than
`rain` directly, so forcing a storm goes through the field: the same one path
decides the weather whether or not anybody pressed anything.

## The cloud slab

`sky_atmosphere.wgsl` gains a second march. The clouds are a shell between
`cloud_inner` and `cloud_outer` rather than a surface at `CLOUD_RADIUS`:

1. Intersect both spheres, clip the segment to the view ray's own span.
2. March `CLOUD_STEPS` samples through it. At each, three octaves of the
   existing value noise over the drifting position give a density; below a
   coverage threshold it is nothing.
3. Accumulate transmittance with Beer's law, and light each step by a short
   march toward the sun through the same field, so the top of a mass is bright
   and its underside is dark. That difference IS the thickness: it is what a
   single-sample decal cannot have.
4. The threshold is driven by the app's `cover`, so an arriving front thickens
   and lowers the sky.

**A grazing ray crosses more slab than a vertical one by construction**, which
is why the horizon builds up and the zenith stays open, and why no extra term is
needed to make clouds look like they have a bottom.

**Steps cost the whole sky, so the count is the one number to be careful with.**
The atmosphere already marches 16 view samples with 4 sun samples each; the
clouds add their own budget on top, on every pixel of sky. The task list has the
measurement rather than a guess.

## What is NOT built

- **Puff geometry.** The reference's per-cell placement, lump clusters, slab LOD
  and far texture shell. On this architecture that is a fifth indirect draw; it
  is its own change.
- **The field in WGSL.** Distant clouds are the shader's noise, not the field's
  moisture. Making them the same means porting the fBm into WGSL and pinning the
  two implementations against each other on the GPU, the way the visibility
  regression already pins a rule.
- **Wind.** `wind01` and `wind_dir` drive the reference's grass sway, and this
  project's grass does not sway yet. The two land together or not at all.
- **Snow as a different particle.** `precip_kind` answers snow over tundra and
  mountains and the field carries it, but the shower draws rain streaks either
  way until there is a snow particle to draw.
