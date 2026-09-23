# Design: atmospheric circulation

Status: proposed. Nothing here is built. The numbers labelled *measured* were
measured, and the ones labelled *arithmetic* are estimates to check once built.

## 1. What exists, measured

- `pbd_core::weather::cloud_cell(field, terrain, direction, seconds)` is the
  **one seam**. Every consumer calls it: the player's `Weather`, the rain
  lattice, `precipitation_map` and the lightning places. So a new source goes in
  behind it, and the consumers are unchanged.
- The field is `moisture_noise * (floor + warmth_noise)`. The warmth is
  translated along `(1, 0.3, -0.5) * solar_drift`. The instrument in the
  proposal measured:
  - no latitude structure;
  - no ocean/land contrast;
  - a single rigid slide, which stops at two opposite points (15 deg N and S).
- **Distribution** *(measured)*, 40,000 directions at 5 times: median cover
  0.09, p90 0.49, p99 0.84; 19% of the planet clear.
- **Correlation length** *(measured)*: cover differs by 0.04 at 168 m apart,
  0.13 at 838 m, and 0.23 at 3.3 km or more. So weather "systems" are about
  1-2 km across, on a planet 30 km round.
- **Rendering.** The cloud shader takes the player's cover as its threshold for
  every pixel on the planet (`sky.rs`, `apply_weather`). Its drift is a constant
  translation of the noise (`slab.z` seconds times a fixed vector).
- **Captured** (`docs/screenshots/circulation-before-orbit-*.png`):
  - Under a natural sky, the globe is scattered with puffs of one size,
    spaced evenly over land and sea, with no bands.
  - With the slider at half, the cover over the player (0.59) becomes every
    pixel's threshold. Cloud covers **98-100%** of each fifth of the disk, top
    to bottom, *(measured)*: one player's weather is the whole planet's.
- **The sun.** `daylight::SUN_FIXED` has y = 0.398, so the sub-solar latitude
  is 23.45 deg N for ever. The spin is about +Y, and the day is 2,880 s:
  Omega = 2.18e-3 rad/s.

## 2. The grid

The Goldberg dual that `planet_topology::dual_sphere` builds, moved into
`pbd-core` (glam only), with the app calling the core's. The atmosphere runs at
**level 5 by default** (knob `atmosphere_level`, 4..6):

| Level | Cells | Mean spacing |
| ---: | ---: | ---: |
| 4 | 2,562 | 363 m |
| **5** | **10,242** | **181 m** |
| 6 | 40,962 | 91 m |

A weather system about 1.5 km across is 8 cells at level 5. The cloud noise
still draws detail finer than a cell; the simulation decides how much cloud
stands where, and the noise decides its shape.

Precomputed once, per cell: the centre, the dual area, the neighbour list (5
or 6, in the stable order the dual builds them), each edge's length and the
distance between centres, and a tangent basis. The operators are standard
finite volume on the dual:
- gradient: the sum over edges of the difference times edge length over area;
- divergence: the sum of the flux through each edge over the area.

They are written for **any** per-cell field, which is what an ocean layer would
reuse.

## 3. State

Per cell, `f32`, about 40 B. That is 410 KB at level 5.

| Field | Unit | Meaning |
| --- | --- | --- |
| `phi` | m^2/s^2 | pressure anomaly, as geopotential |
| `wind` | m/s | surface wind, a 3-vector kept tangent |
| `air_k` | K | air temperature anomaly |
| `ground_k` | K | surface temperature anomaly (the sea-surface temperature over ocean) |
| `vapour` | kg/m^2 | water vapour in the column |
| `cloud` | kg/m^2 | condensed water in the column |
| `charge` | unitless | electrification |
| `rain_rate` | kg/m^2/s | output only, not carried |

Per-cell constants come from the terrain generator, which stays pure:
- ocean or land;
- the land's wetness (biome moisture: swamp high, desert low);
- the surface heat capacity (ocean large, land small, ice middling);
- the surface albedo (snow and ice high).

## 4. One step

The step is fixed at `atmosphere_dt_s` = 1 s of world time. It runs these
stages in this order, over cells in index order:

1. **Sun.**
   - Insolation is `solar_constant * max(0, n . sun) * (1 - cloud_albedo *
     cover)`, with `sun` from the world clock.
   - The ground warms by that times `(1 - albedo)`, divided by its heat capacity.
   - The ground cools by radiation toward a reference, exchanges sensible heat
     with the air, and loses the latent heat that evaporation takes.
2. **Air heat.**
   - The air takes sensible heat from the ground.
   - It radiates toward its reference, which is tied to latitude through the
     sun: the equilibrium is not written down by hand.
   - It gains latent heat from condensation (stage 6).
3. **Pressure** (the Gill model).
   - `phi` falls where air is warmer than the global mean, which makes a
     thermal low.
   - It rises where air converges: `d phi/dt = -c^2 div(wind) - gamma
     (air_k - mean) - phi / tau_phi`.
   - That is what "equalizes": a pressure bump radiates as gravity waves at
     `c` and damps out.
4. **Wind.**
   - `d wind/dt = -grad phi - wind / tau_drag`. Drag is stronger over land
     than over sea.
   - Then the **Coriolis force**, `f = 2 * coriolis_scale * Omega * (n . Y)`,
     applied as an exact rotation of the wind about the local vertical by
     `-f dt`. It is written as a Cayley rotation, so it needs no sine or cosine
     and keeps speed exactly.
5. **Carry.** The wind, `air_k`, `vapour`, `cloud` and `charge` are moved
   semi-Lagrangian:
   - trace each cell centre back along `-wind dt`;
   - interpolate barycentrically in the dual triangle it lands in;
   - parallel-transport vectors to the arrival cell's tangent plane.

   A global mass fixer then rescales `vapour` and `cloud`, so the transport
   neither creates nor loses water. Stage 6 is the only place water changes.
6. **Water.**
   - **Evaporation** is `k_e * max(0, q_sat(ground_k) - vapour) * (1 +
     |wind| / wind_ref) * wetness`. Wetness is 1 over sea, the biome's moisture
     over land, and low over ice.
   - **Condensation** is `max(0, vapour - q_sat(air_k - lapse * lift)) /
     tau_condense`. `lift = max(0, -div(wind)) * H` is converging air rising.
     Latent heat goes into `air_k` (stage 2), and the water goes into `cloud`.
   - **Re-evaporation**: cloud in dry, sinking air returns to vapour.
   - **Rain** is `max(0, cloud - cloud_rain_min) / tau_rain`. It falls as snow
     where `ground_k` is below `freeze_k`.
   - `q_sat` is `q0 * exp(k * t)` (Clausius-Clapeyron), evaluated as a fixed
     polynomial in the core rather than `f32::exp`. That way the answer is the
     same on every platform (section 7).
7. **Charge and lightning.**
   - `charge += k_charge * condensation * lift`, and it decays on `tau_charge`.
   - A cell over `strike_charge` strikes when a hash of its index and the step
     passes `strike_chance`. The strike:
     - discharges it;
     - rains out a share of its cloud at once;
     - drops a **cold pool**: `phi` up by `pool_phi`, `air_k` down by
       `pool_k`.

     The cold pool is high pressure, so the next steps push air out of the cell.
     Where that outflow meets the air around it, it converges and lifts, and
     the neighbours can start storms of their own. That is the physical reason
     lightning drives weather: the strike itself carries little energy, but it
     marks the downdraft that spreads the storm.
   - Strikes are recorded as (cell, step, strength) for the renderer. The flash
     and bolt the player sees come from these.

**The cloud-level wind** is diagnosed, not carried. It is the surface wind plus
the thermal wind, `(thermal_wind / f) * n x grad(air_k)`, capped at
`jet_max`. `thermal_wind` (m^2/s^2/K) stands for `g H / T0`. The
thermal wind is largest where the temperature changes fastest toward the pole.
That band is the **jet stream**, and it moves as the temperature field does.
Near the equator `f` goes to zero, so `f` is floored at `f_min`.

**Stable step.**
- Semi-Lagrangian transport is stable at any wind speed.
- The explicit gravity waves need `c dt < spacing`. At `c` = 30 m/s and 181 m
  cells, that means dt < 6 s, and dt is 1 s.
- The Coriolis rotation is exact.
- Every stage guards against non-finite values and clamps its fields to their
  physical ranges; a non-finite value is logged and the cell reset to its
  climatology.

## 5. Why the Coriolis force is scaled *(arithmetic)*

Held and Hou (1980) put the edge of the Hadley cell at `phi_H = sqrt(5/3 * g H
Delta / (Omega a)^2)`. For Earth that gives 35 deg, and Earth has three cells a
hemisphere. Here, `a` = 4,800 m, `Omega` = 2.18e-3, and `g H` becomes the
model's `c^2`:

| c (m/s) | Scale 1 | 2 | 4 | 8 |
| ---: | ---: | ---: | ---: | ---: |
| 20 | 82 deg | 41 deg | 20 deg | 10 deg |
| 30 | 90 deg | 61 deg | 31 deg | 15 deg |
| 50 | 90 deg | 90 deg | 51 deg | 26 deg |

At the real spin, the Hadley cell reaches the pole: one cell per hemisphere, no
westerlies, no jet. The planet is small and its day is short in absolute terms,
but its rotation is slow compared with how far air crosses it in a day. To
land near Earth's 35 deg, `coriolis_scale` starts at **4** with `c` 30 m/s. The
climatology test (section 9) decides the final value, and the knob says why it
is not 1.

The deformation radius `c / f` agrees: at scale 4 it is 2.4 km (0.5 radii),
which leaves room for several storm systems round a latitude circle.

## 6. Coupling to everything else

- **The seam.** `cloud_cell` takes an `&Atmosphere` and a direction. It
  interpolates the cell state with the same barycentric function the transport
  uses, and returns cover (a smoothstep of `cloud`), alpha, raining and what
  falls. `rain_at`, `raining_cells`, `precipitation_map` and the player's
  `Weather` are unchanged except for the argument.
- **The old field stays only as seed noise.** `solar` and `weather_moisture`
  seed the first state's perturbations (section 7) and nothing else. There are
  no longer two answers to "is it raining here".
- **Rain-trailing goes.** `rain_min_s` sampled the field in the past so that
  rain would trail its cloud. A simulation's cloud rains out over `tau_rain`
  and trails by itself.
- **Lightning.** `weather::Lightning::strike`, the clock hash, is replaced by
  the strike list. `strike_lightning` picks the brightest strike within range
  of the camera this frame. The flicker and the bolt are unchanged.
- **The slider.** `StormForcing` becomes a forcing term. While it is above
  zero, cells within `forcing_radius_m` of the player are nudged toward a storm
  column on `forcing_tau_s` (5 s): vapour and cloud up, air warmed. The storm
  forms at the player within seconds. When the forcing is released, the storm
  lives on and drifts with the wind. NATURAL means no forcing.
- **Time.**
  - The simulation advances with the world clock in whole steps, carrying the
    remainder.
  - At most `max_steps_per_frame` steps run per frame, so a slow frame cannot
    spiral.
  - It does not advance while paused.
  - It runs on the main schedule (a few ms at most, section 9). If it measures
    over budget, it moves to a task and publishes a completed state, never a
    partial one.

## 7. Authority, determinism and saves

This replaces the requirement that weather is stateless.

- **Deterministic.** Given the same seed and the same number of steps, a build
  produces a bit-identical state. The ingredients:
  - a fixed step;
  - cells in index order;
  - no hash map anywhere in the step;
  - strike randomness from a hash of the cell and the step;
  - only `+ - * /` and `sqrt` in the step (IEEE-exact everywhere), with `exp`
    and the rotation done in core arithmetic rather than `libm`, which varies
    by platform.
- **Starting.** A new world starts from the zonal climatology: temperature at
  radiative equilibrium for its latitude, wind at rest, vapour at half
  saturation. The old field's noise perturbs it. It is then spun up for
  `spinup_s` (default one day, 2,880 steps) at load, so the first frame already
  has weather. That cost is measured (section 9).
- **Saving.** The state goes into the world save through the existing durable
  backend. It is written on a cadence (`snapshot_s`, 60 s) and when the world
  is left. The format is versioned, with the simulation version in the world ID
  as topology versions already are.
  - This is not the "every accepted mutation, immediately" path. That rule is
    for what a player *did*. A lost weather snapshot costs a minute of sky and
    nothing anyone made, and the simulation continues from any valid state.
  - **Owner's call**: if weather should be as durable as an edit, it goes on the
    transaction path. At 410 KB a write, a whole-state write every step is not
    reasonable, but a per-minute one is.
- **Multiplayer, later.** The server steps and sends snapshots quantized to 8-16
  bits a field, about 80 KB each, every ~10 s. Clients keep stepping between
  snapshots and blend toward each one. Nothing here depends on the clients
  agreeing bit for bit.

## 8. Rendering

- **The weather map.** Two cube textures, each face 64 x 64, RGBA16F (24,576
  texels, about 2.4 per cell):
  - **A**: cover, cloud-top height (0..1 of the tallest tower), precipitation
    (negative for snow), and `tau_up` (the column's optical depth, which
    `cloud-lighting`'s ambient reads in one fetch).
  - **B**: the cloud-level wind, xyz in the body frame (m/s).

  The CPU resamples the cells into the map with the same barycentric function
  as `cloud_cell`, at `map_hz` (2 Hz). It keeps the previous map as well, so the
  shader blends the two by time and the cover never pops.
  - Upload: 2 x 393 KB per refresh, about 1.6 MB/s at 2 Hz.
  - Linear filtering is right for this data texture. The nearest-point rule is
    for terrain art.
- **The march.**
  - `cloud_density` remaps the noise by the map's cover at the sample
    (`remap(noise, 1 - cover, 1)`, as in Horizon Zero Dawn) instead of by one
    global threshold.
  - The height profile is scaled to the map's cloud top: a storm column
    towers, a stratus deck stays flat.
  - The slab's outer shell is raised to the tallest tower `cloud_top_max_m`,
    which must stay under the atmosphere shell (960 m above the surface); that
    is validated.
  - The empty-space early-out keeps the added height cheap.
- **Wind.** The noise is advected by the map's wind with a two-phase flow map
  (the technique `water-flow` names for rivers). The displacement is the wind
  times the phase time, converted to noise units by `22 / a`. Two phases half a
  period apart are cross-faded, so the stretch never accumulates. The jet shows
  as streaming bands, and a low as a turning spiral.
- **What goes.** The global cover lane (`slab.y`) and the constant drift
  (`slab.z` times a fixed vector) are replaced by the map. The overcast light
  on the ground still reads the player's cover, which is now the simulation's
  cover at the player.

## 9. Measured as

There is a climate report instrument: a core example, `cargo run --release -p
pbd-core --example climate`. It spins up, runs a day, and prints zonal means
of cover, rain, surface wind, cloud-level wind and temperature per 10 deg
band, plus the ocean/land split, the diurnal phase of land rain, and step
timing. Its claims become tests where they are robust.

**Circulation** (relative to the sub-solar latitude, since the sun sits at
23.45 deg N):
- the maximum zonal-mean rain is within 20 deg of the sub-solar latitude (a
  tropical rain belt);
- zonal-mean cover has a minimum in each hemisphere 15-40 deg from that belt,
  at most half the belt's cover (desert belts);
- surface zonal wind is easterly in the belt's tropics (trades) and westerly
  in mid-latitudes in at least one hemisphere;
- the cloud-level wind's zonal-mean maximum lies poleward of a subtropical
  minimum (a jet).

**Water and sun:**
- cover over ocean is higher than over land, averaged over a day;
- land rain peaks between local noon and 18:00.

**Structure** (this is the "not so uniform" claim):
- at any time at least 10% of the planet's area is clear and at least 5% is
  under full cover;
- the standard deviation of zonal-mean cover across bands is at least 0.1
  (now: 0.03, measured from the proposal's table).

**Dynamics** (unit tests):
- with forcing off, a single pressure bump spreads and damps, its variance
  falls every step, and mean `phi` is conserved to 1e-5;
- with the spin on, flow round a low turns anticlockwise in the north and
  clockwise in the south (the sign of the relative vorticity);
- the total water (vapour, cloud and what has rained out) balances what
  evaporated to 1e-4 over a day;
- nothing becomes non-finite over ten simulated days at the knobs' extremes.

**Lightning** (unit tests):
- no strikes after the forcing is removed and the storms decay;
- a strike discharges its cell, and the next step has outward divergence
  round it.

**Determinism** (unit tests):
- two runs from one seed agree bit for bit;
- saving, loading and stepping gives the same state as stepping straight on.

**Cost** (release, this container, hardware reported):
- ms per step at levels 4, 5 and 6;
- the spin-up time;
- map upload bytes per second;
- memory.

*Arithmetic*: about 500 flops a cell a step, which is 5 Mflop, which is
1-3 ms a step on one core at level 5. The target is under 2 ms.

**Pictures**, measured the way the cloud bands were measured in
`overcast-and-rain`:
- from orbit at three hours of the day: the bands and storm systems;
- a time-lapse of 60 frames of one low turning;
- from the ground, a storm forming under the slider.

**Risk.** A single layer may not produce three clean cells even with the
scaled spin. The fallback is a weak relaxation of the zonal-mean wind toward a
three-cell profile (`climate_nudge_tau_s`, as Held and Suarez's benchmark
does). It is off by default and turned on only if the circulation claims fail,
and the design will say so if it is.

## 10. What this does not settle

- Seasons: the permanent solstice, flagged in the proposal.
- Ocean currents: they would reuse section 2's operators later.
- Whether weather state should ride the per-edit durability path (section 7).
