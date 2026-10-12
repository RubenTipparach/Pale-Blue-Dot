# Design: smooth-weather

## Context

`crate::atmosphere::Air` publishes one whole state at a time. Every reader
uses it for the whole frame (sky, sea, ground, rain), which is why they never
disagree. The proposal measures the cost of that: one-second ticks, lurches
after a held step, and rain that switches between nothing and full cover in
bursts about 25 s long. These decisions keep the one-state agreement and take
the jumps out.

## Decisions

### 1. The rain seen is a per-cell follower in the atmosphere, derived and unsaved

The atmosphere gains `rain_seen: Vec<f32>`, one entry per cell. After each
step, each cell's entry follows a target:
- the target is the cell's cover where its rain rate is at or over
  `raining_rate`, and nothing where it is under;
- it climbs toward that target with `rain_rise_s` and falls with
  `rain_fall_s`:

```text
tau  = if target > seen { rain_rise_s } else { rain_fall_s }
seen = seen + (target - seen) * (1 - exp(-dt / tau))
```

`Atmosphere::sample` interpolates it like any field, and returns it as
`Sample::rain_seen`.

Why in the atmosphere, and not in each consumer:
- **One number, one place.** The player's rain, the rain shafts and the rain
  volume all read the seam (`pbd_core::weather`). With the follower in one
  place, the three agree at every point without each keeping its own memory.
  That is the "one rain intensity drives every rain effect" requirement.
- **Shafts need it per cell.** A shaft cell that comes into view must show
  the rain it has had, not start from nothing or from full. A memory per
  cell does that.

Why it is safe there:
- **The physics never reads it.** It is not in `scalars()`, so `to_bytes`,
  the saved weather, the shipped settled climates and every digest of them
  are byte for byte what they were.
- **It starts from the rain as it stands.** `restore` and `new` set it to its
  target, so a loaded world shows the rain as it is rather than fading it in.

The constants, from the instrument (proposal):

| Rise, fall | Crossings of 0.05 a raining place, 10 min | Largest move in 1 s | Mean rain seen |
| --- | ---: | ---: | ---: |
| switched (today) | 8.4, all full jumps | 1.00 | 0.236 |
| 4 s, 15 s | 2.9 | 0.221 | 0.266 |
| **5 s, 20 s** | **2.3** | **0.181** | **0.273** |
| 8 s, 8 s | 4.9 | 0.118 | 0.236 |

The pick is 5 s and 20 s. Rain builds in over about ten seconds and dies away
over about a minute, so most dry gaps between bursts (median 23 s) become a
lull rather than an off switch. The cost is 15% more rain on screen on
average, the tails of showers. Both constants are `atmosphere.ron` settings
(`rain_rise_s`, `rain_fall_s`), validated positive. Adding them changes
`AtmosphereSettings`, which the shipped settled climates are checked against
(`the_shipped_settled_climates_are_made_with_the_running_settings`). Their
`.ron` gains the two fields at these values, and their bytes are unchanged.

`cloud_cell`, `rain_at`, `raining_cells` and `precipitation_map` read
`rain_seen`:
- a cell draws rain when `rain_seen` is over 0.01;
- what falls is snow or rain by the sample's ground temperature, as now;
- `CloudCell::raining` keeps the simulation's own threshold answer, for the
  readers that want the physics (the lattice test, the overlays).

### 2. What is shown runs between two published states

`Air` gains `shown: Shown`:

```rust
pub struct Shown {
    pub from: Arc<Atmosphere>, pub from_maps: Arc<WeatherMaps>, from_at: f64,
    pub to: Arc<Atmosphere>,   pub to_maps: Arc<WeatherMaps>,   to_at: f64,
    /// How far from `from` to `to`, 0..1.
    pub t: f32,
    span_s: f32,
    pending: Option<(Arc<Atmosphere>, Arc<WeatherMaps>, f64)>,
    /// Bumped when the pair changes, so the GPU uploads it once.
    pub pair: u64,
}
```

- **A publish** puts the new state in `pending` (a newer one replaces an
  older one that is still waiting).
- **Each frame**, `blend_air` advances `t` by the world clock's step over
  `span_s`. When `t` reaches 1 and a state is waiting, the pair moves on:
  - `from` becomes `to`, and `to` becomes the waiting state;
  - `span_s` becomes the weather time between them, held between `dt_s` and
    `MAX_BLEND_S` (4 s);
  - whatever `t` ran past 1 carries over.
- **Nothing waiting**: `t` holds at 1.

Why this shape:
- **No re-base, so no jump.** A new state never interrupts a blend, so what
  is shown is continuous whatever the cadence.
- **The ordinary step plays at its own pace.** One step is 1 s of weather
  over 1 s.
- **A held step plays at its own pace up to 4 s.** A three-step catch-up
  plays over 3 s. A thirty-step one plays over 4 s, which is fast, but in
  240 frames rather than one.
- **The lag stays small.** Behind the newest state it is at most one span
  plus a wait, under 5 s. It is not noticed against weather that changes over
  minutes.

What snaps to the newest state at once (`from = to = newest`, `t = 1`):
- **The first state.**
- **A capture** (`in_place`): its picture must be a function of its flags.
- **A clock that jumps**: back, or by more than `MAX_GAP_S`, which is a load,
  the time slider or `--day`. The sun jumps with it, so the weather jumping
  too is the same event.

`Air::now` is unchanged. It is the newest state, the one stepping, saving,
lightning, the map screen and the overlays read.

### 3. The CPU readers mix the pair

- **`weather::sample_field`**:
  - `cover` and `rain` are the two states' values at the camera, mixed by `t`;
  - `snowing` comes from whichever state `t` is nearer.
- **`weather::fill_rain_map`**:
  - fills a map from each state of the pair when the pair moves, or when the
    anchor strays;
  - `values` is their mix, made each frame (4,096 lerps).
- **`weather::cell_shafts`**: each lattice cell in the detail range takes its
  `rain_seen` mixed by `t` from both states, about twenty cells. The share of
  its streaks, and their alpha, scale with it. A shaft thins in and out
  instead of popping.
- **The lens, the ripples, the wet sheet and the sky dome** read
  `Weather::rain` and `cover`, and need nothing of their own.

### 4. The GPU mixes the maps, in a compute pass

Today `planet_weather` writes the cloud and wind cubes whole at each publish.
Instead:
- **Uploads**: the render world keeps `from` and `to` textures for each map,
  uploaded only when `pair` moves.
- **The pass**: `weather_blend.wgsl` writes `mix(from, to, t)` into the cubes
  the bind group already holds. It is one dispatch of 8 x 8 x 6 workgroups
  per map, every frame, submitted from a render-world system before the
  frame's graph runs.
- **What does not change**: the bind group layout and every shader that
  samples it.

The cubes gain `STORAGE_BINDING`. They are bound for the pass as
`texture_storage_2d_array<rgba16float, write>`, which every adapter supports
for that format.

Until the pipeline has compiled, and on any adapter where it fails, `to` is
written straight into the cubes when the pair moves. That is today's
behaviour, so the weather is never blank. A GPU test runs the pass on an
adapter and checks the mix against the CPU's.

## Risks

- **15% more rain on screen.** A shower's tail now lingers. The wetness
  already lags rain by `wet_fade_tau_s`. The owner judges both together on
  the before-and-after shots.
- **The overlays and the map screen still step.** They read `Air::now` once
  a second, which is right for a data layer. They are not what the owner
  flagged.
