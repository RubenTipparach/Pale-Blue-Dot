# Design: vehicles

Three craft (a VTOL tiltrotor, a sailing keelboat and a paddled canoe), the sea
they float on, the air that pushes them, and the rules that keep them in the
world after you step out. **None of it is built.** The physics exists as the JS
prototype in `docs/mockups/vehicles.html`
([published](https://claude.ai/artifact/UJebv7EdJUg1NLpvfVHi3D)), which is the
reference the Rust port will be held to.

## 1. What the engine has today, measured

### The swell cannot float a boat

`water.wgsl` lines 205–209 displace every water-cap vertex radially:

```wgsl
let t = view.camera_time.w*view.waves.z;          // PlanetClock * time_scale 0.75 * swell_speed 1.0
let scale = view.waves.y;                          // swell_frequency 2.0
let wave = (sin(t*0.9+p.x*1.4*scale+p.z*0.6*scale)*0.18
    +sin(t*1.3-p.x*0.7*scale+p.z*1.2*scale)*0.12
    +sin(t*1.7+p.x*2.3*scale-p.z*1.9*scale)*0.06)*view.waves.x;   // swell_amplitude_m 0.5
```

Worked out per component, with `p` in body-local metres:

| component | amplitude | wavelength | phase speed | deep-water speed at 25 m/s² | too slow by |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.090 m | 2.063 m | 0.222 m/s | 2.86 m/s | 12.9× |
| 2 | 0.060 m | 2.261 m | 0.351 m/s | 3.00 m/s | 8.5× |
| 3 | 0.030 m | 1.053 m | 0.214 m/s | 2.05 m/s | 9.6× |

The water cap has a vertex at each cell centre and at each corner. The corner
sits at the hexagon's circumradius, 2.833 / √3 = **1.64 m** from the centre.
A mesh with that spacing can draw only waves longer than twice the spacing,
**3.27 m**. All three components are shorter than that, so what the player
sees is the function's alias on the cell lattice, not the function.

Three consequences for vehicles:

- A CPU copy of today's function would float a hull on ripples the player
  cannot see. **The function itself has to change, not only move to the CPU.**
- At 0.18 m peak and 2 m wavelength, a 5–6 m hull averages the swell out.
  The boat would sit on a flat sea however the sea was drawn.
- The water spec deliberately forbids a second, CPU evaluation ("the
  straddling band SHALL be at least as wide as the wave amplitude, so the
  wave function is not evaluated a second time on the CPU"). Section 2
  replaces that rule with a single CPU implementation that the shader
  mirrors, and the spec delta modifies the requirement accordingly.

### The rest of what a vehicle stands on

| need | what exists | where |
| --- | --- | --- |
| wind, current, rain at a place | `Atmosphere::sample(dir)` returns `wind`, `upper`, `current` and `rain_rate`, in body-frame m/s and kg/m²/s | `pbd-core/src/atmosphere` |
| gravity | 25 m/s² at the surface, full to 1.4R, fading to 0 at 1.8R | `pbd-core` gravity, `Scene::gravity_at` |
| physics tick | Avian at 60 Hz with 4 substeps (240 Hz), `PhysicsSchedule` in `FixedPostUpdate`, Avian gravity off | `pbd-app` |
| ground contact | no terrain colliders; custom contact against `PlanetContact::{sample, stand}` | walker, `protect_terrain_clearance` |
| a craft | one Avian sphere, no mesh, a first-person camera, F to swap in and out | `lib.rs::spawn_ship`, `walking.rs::switch_mode` |
| persistence | `WorldSave{edits, carried, kit, pose, world_seconds, weather}`; walker pose on a 5 s autosave | `desktop.rs`, `save` |
| cameras | walking and flight cameras; systems find "the camera" through `Camera::is_active` | `walking.rs::set_active_mode` |
| free keys | C, H, I, J, K, L, N, O, T, U, V, Y, Z, Tab | `controls.rs::BINDINGS` |

## 2. The sea: one function, one table, drawn and floated on

**A fixed table.** The table holds 30 components: 5 wavelengths (7, 14, 28, 56
and 112 m) × 6 headings 60° apart, plus one long swell (70 m). Each component's
wavenumber `k` and its deep-water frequency `ω = √(g k)` are **constants of the
planet**. A component's phase is `k·x − ω t + φ₀`, with `t` the world clock
the save carries.

**The wind sets heights, nothing else.** Amplitudes come from a
Pierson–Moskowitz spectrum at the local sea-state wind `U` with cos²
spreading about its heading. They are then scaled so the table's significant
height is the spectrum's, `Hs = 0.21 U² / g`. Each component is capped at a
steepness `a k ≤ 0.1`.

This is the decision the prototype forced. A table whose wavelengths or
headings follow the wind has to change `k` and `ω` when the wind changes,
and `k·x − ω t` then jumps everywhere at once, the further from the origin
the more. With fixed components, a change of wind only fades amplitudes.

**The sea state lags the wind.** `U` eases toward the atmosphere's wind with
a time constant. The mockup uses 25 s. The engine should carry `U` as a slow
per-cell field in the atmosphere step, saved with the weather, so a sea
builds over minutes and outlasts the squall that raised it.

**Heights at g = 25 m/s²** are 2.5× lower than Earth's for the same wind, and
peak wavelengths 2.5× shorter:

| wind | Hs (g = 25) | peak λ | Hs (Earth) | peak λ |
| ---: | ---: | ---: | ---: | ---: |
| 6 m/s | 0.30 m | 12 m | 0.77 m | 30 m |
| 11 m/s | 1.02 m | 40 m | 2.59 m | 101 m |
| 16 m/s | 2.15 m | 84 m | 5.48 m | 213 m |
| 24 m/s | 4.84 m | 188 m | 12.3 m | 480 m |

The surface band means 0–3.5 m/s (`examples/climate.rs`), so most of the
planet's sea sits under half a metre and storms are what raise it. A fetch
limit (`max_hs_m` in `sea.ron`) is a knob, not a rule. Its default is off.

**Shoaling.** Where the water is shallow, each amplitude is multiplied by
`smoothstep(0, λ/2, depth)` and the total is capped at `0.6 × depth`. Both
sides know the depth: the CPU reads it from `PlanetContact`'s floor radius,
and the cap has it per cell. Without this, a 2 m storm sea would stand as a
wall against the beach cells.

**The renderer filters, not aliases.** Every vertex drops the components
shorter than 2.5× its own spacing, fading them in over 1.6× that. On the cap,
the spacing is the hexagon's 1.64 m, so the 7 m band is the shortest the cap
ever draws and the physics floats on nothing the player cannot see. The
mockup's sea mesh has variable spacing and applies the same rule per ring.

**Where it lives.**
- `pbd_core::sea::{SeaTable, SeaState, height_and_velocity(x, t, depth)}`.
- The velocity is linear-theory orbital motion decaying as `e^{kz}` below the
  surface, plus the atmosphere's current.
- The WaterView uniform carries the table: 31 × (a, k, dx, dz) and
  (ω, φ₀, λ, 0).
- `water.wgsl` loops over it. The three literal sines go.

**Holding the two together.** CLAUDE.md asks for the actual artifact to be
validated, not two hand-written copies. The shader's height function is a WGSL
function over the same uniform. A test runs it as a compute shader on a
headless adapter (lavapipe in CI, skipped where there is no adapter) at a
fixed set of points and times. It compares the result to `pbd_core::sea` to
1 mm, and a layout test pins the uniform's offsets.

**Per place.** The CPU samples the sea state at the hull. The shader samples
it from a sea-state channel on the weather maps, `U` and heading per texel,
and derives amplitudes with the same spectrum function. The spectrum function
is the one formula written twice, and the compute test covers it too.

## 3. The air at a point

```text
wind(p, t) = mean(p) * shear(h) + gust(p, t) - downdraft(rain)
```

- `mean` is `Atmosphere::sample(dir).wind`, tangent to the body. Above the
  cloud deck it blends toward `upper`.
- `shear(h) = ln(h/z₀) / ln(10/z₀)`, clamped to 1.4. It is 1 at 10 m, the
  height the atmosphere's wind stands for. `z₀` is 0.0005 m over sea and
  0.03 m over land.
- `gust` is frozen turbulence carried downwind. It is three incommensurate
  sines per axis, evaluated at `t − ξ/U`, where `ξ` is the along-wind
  coordinate. It has no state, so every client and every replay computes the
  same gust at the same place. Its strength is `G · U`. `G` is a knob plus a
  storm term from the rain rate. The vertical part fades toward the surface.
- `downdraft` is `0.3 √rain` m/s, with rain in mm/h: 2.8 m/s at 90 mm/h.
  This is the microburst a hovering craft has to fight in a storm.

## 4. The physics, shared by all three craft

**One rigid body per craft** (Avian). Forces are computed in `pbd-core` as
pure functions of the craft's state and the samples above, returning
point forces. The app applies them in `PhysicsSchedule` before the solver,
where the skiff's thrust is applied today. Mass and principal inertia come
from the craft's parts in `vehicles.ron`. The prototype steps at the engine's
240 Hz. Its stiffest mode is the empty canoe's roll, at ω·dt = 0.19, well
inside the stable range of semi-implicit Euler.

**Hulls are cells.** A hull's envelope is one shape function: half-beam and
bottom along the length, with fineness exponents. It is voxelised at a
per-craft size (0.25 m for the Tern, which gives 389 cells, and 0.14 m for
the Loon, which gives 293). Each cell carries

```text
buoyancy = ρ_w g s³ · clamp((η − h + s/2)/s, 0, 1)   along local up
damping  = −c_v · submerged volume · (v_cell − v_water)·up
```

with c_v = 17,000 N·s/m⁴. The same shape function draws the loft, so what
displaces water is what is drawn.

**One foil function** serves the wing panels, tail, fin, sail, keel,
rudder and the canoe's lateral plane:
- lift slope `2π·AR/(AR+2)`;
- a stall that blends from `clamp(CLα·α, ±CLmax)` into flat-plate
  `1.05 sin 2α` over 0.18 rad;
- drag `cd₀ + CL²/(π·0.8·AR)` blending into `1.25 sin²α`;
- control deflection shifts α.

Flow is taken relative to the foil's own point, so roll damping,
weathercocking and a gust arriving on one wing first all fall out without
extra terms.

**Hull resistance.**
- Friction and wave-making resistance act along the keel line.
  `Cw = 0.0015 + 0.05·smoothstep(0.28, 0.46, Fr) + 0.12·max(0, Fr − 0.46)`,
  so a displacement hull meets a wall near hull speed, `√(gL/2π)`.
- There is a lateral drag.
- Hull speed at 25 m/s² is 1.6× Earth's: 9.2 kt for the Tern's 5.6 m
  waterline, against 5.7 kt.

**Contact** is springs and dampers against `PlanetContact`: gear wheels with
rolling and braking friction, and keel and hull bottom points against the
floor radius. There are no terrain colliders, as for the walker.

**Water aboard is a load.**
- Water comes in from rain over the open area (Loon 3.6 m², Tern cockpit
  1.6 m²) and from green water over the rim, a weir law
  `Q = 1.7 b H^1.5` at each sheer point under the local sea.
- The Tern's scuppers drain 0.6 kg/s. Bailing removes 6 kg/s.
- Water adds mass, and it sloshes toward the low side as a heeling torque.
  That positive feedback is what swamps a canoe.

## 5. The three craft

| | Kestrel | Tern | Loon |
| --- | --- | --- | --- |
| kind | tiltrotor VTOL | cat-rigged keelboat | canoe |
| mass | 1,200 kg | 640 kg + 80 kg crew | 30 kg + 80 kg paddler |
| size | 10.8 m span, 8 m long | 6.2 × 2.3 m, 1.5 m draft | 5.0 × 0.92 m |
| lifting surfaces | wing 2 × 8 m² (AR 6.25, 3° incidence, 4° dihedral), tail 3.4 m², fin 2.2 m² | sail 9.4 m² (AR 3.4), keel 1.0 m², rudder 0.3 m² | lateral plane 1.1 m², skeg 0.06 m², blade 0.11 m² |
| drive | 2 rotors × 24 kN (T/W 1.6), tilting 90°→0° at 15°/s | apparent wind on the sail | strokes: 1.3 m in 0.55 s, then 0.5 s recovery |
| at 25 m/s² | stall 46 m/s; hands-off trim 111 m/s | hull speed 4.7 m/s | hull speed 4.4 m/s |

**Kestrel.**
- Each rotor's thrust falls with inflow along its axis (zero at 150 m/s).
  It rises up to 20% in ground effect, `1/(1 − (R/4z)²)`, and each rotor
  drags sideways in a crosswind.
- In the hover, roll is differential collective and pitch and yaw are
  cyclic and differential tilt. All three are scaled by how vertical the
  nacelles are and by power. Wing, tail and fin surfaces take over as
  dynamic pressure grows.
- Rain wets the wing: CLmax falls by up to 15% and cd₀ rises by up to 60%
  at 150 mm/h.
- **Assist** (X, the flight model's dampeners):
  - rate command;
  - in the hover, attitude hold and vertical-speed hold (Space and Ctrl
    command ±7 m/s);
  - with the stick centred, tilt against drift over the ground.

  With assist off the stick is raw, and the hover is unstable, as it is in
  the real thing.

**Tern.**
- The boom swings to align with the apparent wind until the sheet stops it
  (5° to 85°). The angle between the stopped sail and the flow is the sail's
  angle of attack.
- So the sail luffs when it is eased too far, stalls when it is sheeted too
  hard, and a gybe swings the boom across at a rate that grows with the wind.
- Heeling moment is balanced by the hull's form stability, a 220 kg ballast
  bulb and the crew's hiking torque (Q/E).
- Upwind ability comes from the keel's lift at a few degrees of leeway.
  Inside about 32° of the apparent wind the sail cannot drive: "in irons".

**Loon.**
- A blade's velocity is the boat's point velocity plus its stroke velocity
  relative to the boat. The force is `−½ρ C_d A |v_rel| v_rel` against the
  local water velocity, applied at the blade.
- Forward strokes alternate sides (W). A turns left by paddling on the
  right, D the mirror. S back-paddles, and Q/E hold the blade at the stern
  as a rudder.
- The paddler's windage is 1.3 m² side-on, so strong wind beats paddling.

## 6. Boarding, occupancy, persistence

**One interaction key.** The game design says "one interaction key boards,
uses, or opens the highlighted target". F is taken (walk/fly), so the key is
**G**:
- on foot, it boards the nearest craft whose boarding point is within reach
  (the cockpit door, the cockpit, the canoe's centre thwart);
- aboard, it leaves.

**T** moors to a bollard within 14 m, drops anchor where the water is at
most 35 m deep (rode 3 × depth + 2 m), or casts off. The mockup has no
walker, so there 1, 2 and 3 stand in for walking up to a craft, and G steps
out.

**The walker is a passenger, not a second copy.** While aboard, the walker
body is parked in the craft's frame at its seat, and its collider is off.
Leaving puts it at the craft's exit point, resolved against `PlanetContact`
(deck, pier, ground or water, where "Leaving the ship over water lands in the
water" already holds).

**A vehicle record**, stable ID first, holds:
- kind and craft data version;
- frame ID, pose and velocity in that frame;
- water aboard;
- mooring (bollard ID or anchor point and rode);
- occupancy;
- an owner field;
- empty inventory and damage slots, reserved for the changes that give
  them meaning.

**When it is written.** The record enters the durable transaction path when
the craft is spawned, boarded, left, moored, anchored or cast off, and when
it comes to rest (sleeps). A drifting unattended boat is also written on the
walker pose snapshot's cadence. That is honest about what a timer can
promise: the craft's existence and occupancy are never lost, and its drift
between snapshots is re-simulated on load from the last written pose. The
rule "acknowledge commitment only after the storage backend succeeds" applies
unchanged.

**The unattended policy**, as the game design requires. It is written down
because it will be visible:
- **Kestrel:** nacelles return to 90°, assist forces on, it holds ground
  velocity at zero, commands −3 m/s and cuts power on touchdown. It lands
  where it is, and if that is the sea, it floats (fuselage buoyancy of
  1.73 m³ holds up to 44 kN against its 30 kN weight).
- **Boats:** sheet eased fully, tiller centred, crew mass removed. They
  drift and weathercock unless moored or anchored.
- **Outside the active region** a craft that is at rest sleeps where it is.
  One that is still moving is advanced coarsely: position only, along the
  local current and 3% of the wind for a boat. It is snapped back onto the
  sea when the region returns.

## 7. Camera, controls, HUD

- **`VehicleCamera`** is the third camera marker, switched by a generalised
  `set_active_mode`.
  - Seat view: the craft's pose times the mouse-look offsets, with raw
    displacement and no easing, as CLAUDE.md requires.
  - Chase view: orbit from raw mouse input about a point above the craft,
    with only the heading follow eased.
  - Water state, weather, LOD and digging all key off the active camera, so
    they follow for free.
- **Bindings** go into `controls.rs::BINDINGS` in three new groups, KESTREL,
  TERN and LOON. The reader test's `READERS` gains the vehicle input file.

  | action | Kestrel | Tern | Loon |
  | --- | --- | --- | --- |
  | W/S | pitch down/up | sheet in/ease | forward strokes / back-paddle |
  | A/D | roll | tiller left/right | paddle right/left (turn) |
  | Q/E | yaw | crew port/starboard | stern rudder left/right |
  | Space/Ctrl | climb/sink (collective) | | |
  | Z/C | nacelles forward/up | | |
  | X | assist | | |
  | B | wheel brake | bail | bail |
  | G / T | leave / (n/a) | leave / moor, anchor | leave / moor, anchor |

  The mockup puts sink on Shift, because Ctrl+W closes a browser tab. The
  native build keeps Ctrl, as the FLYING group does. Two departures from the
  FLYING group are deliberate. A winged craft has no strafe, so A/D roll it.
  Q/E are the rudder, not roll.
- **HUD.** The mockup's instrument strip is the brief: a compass rose (true
  wind, course, apparent wind), per-craft readouts, and state chips (HOVER /
  CONVERSION / WINGBORNE, STALL, DRIVING / LUFFING / STALLED, In irons,
  Moored, Bail!, Swamped). The engine draws it in `bevy_ui` next to the
  existing hotbar HUD.

## 8. What the prototype measured

Measured in the JS prototype under headless Chromium with SwiftShader (no
GPU), at the engine's 240 Hz. These are the prototype's numbers, not the
engine's.

| check | result |
| --- | --- |
| Kestrel hover takeoff, Space held 3 s | climbs at the 7 m/s command, holds attitude within 0.3° |
| Kestrel left in a 16 m/s squall | without the drift hold it blew downwind at 11–20 m/s. With the hold (now in the design) it moved at 5.5 m/s in all, 3 m/s of it the commanded descent, and set down at rest (0.02 m/s) at pad height |
| Tern, 11 m/s wind, close reach, sheet 55% | 3.2–3.9 kt, heel 6–13°, leeway 0.4–9°, VMG to windward 0.8–1.6 kt |
| Tern steered into the wind | "in irons" at 29° apparent, 1.5 kt and falling, as it should |
| Loon, empty, moored, 6 m/s sea (Hs 0.45 m) | rolls 3–20°, stays dry |
| Loon paddled into a 16 m/s squall | 1.0 m/s, heel 29°, 2 cm freeboard: the edge of swamping |
| Physics cost, all three craft | 67–82 ms of JS per simulated second on the headless run's shared CPU. This is a JS number, not a Rust one |

**Three bugs the mockup caught, each a rule for the port:**

1. **Flat coordinates on a sphere.** Both boats were placed at y = 0 about
   220 m from the pole and started **5 m in the air**, because the sea there
   is `d²/2R` = 5.0 m below the tangent plane. They fell in and the canoe
   swamped. Every authored pose is placed as a direction and a height on the
   body (`homeAt(X, Z, h)` in the mockup, `PlanetContact` in the engine),
   never as flat XYZ.
2. **A mooring line has to scale with what it holds.** One 4 kN/m line
   yanked a 30 kg canoe's bow onto the pier and flooded it. The line
   stiffness is now `min(4000, 40 m)` N/m, with damping at 80% of critical.
3. **Attitude hold is not a dampener.** An unattended hover that held
   attitude alone drifted downwind at the wind's speed. Ground-velocity hold
   is what "dampener" means here, as it does for the skiff.

**One visible mistake:** both meshes were wound facing down, so the sea and
the island were culled from above. It was caught by the one screenshot, and
there is nothing in it for the port.

## 9. Configuration

- **`assets/config/vehicles.ron`:** per craft, the parts (mass and position),
  the hull shape and cell size, foils (position, chord, normal, area, AR,
  CLmax, cd₀, control gain), rotors, stroke timing, contact points,
  boarding and exit points, and limits. All in SI units, validated at load,
  with a code-defaults test that loads the shipped file.
- **`water.ron`** gains the sea spectrum knobs: wavelengths, headings, the
  swell, the steepness cap, `sea_tau_s`, the shoaling and `max_hs_m`, and
  the filter factor (2.5).
- **`weather.ron`** gains the gust knobs: base strength, the storm term,
  `z₀` for sea and land, and the downdraft factor.

## 10. Open questions for the owner

1. **The feel.** Is the Kestrel's transition too easy or too hard? Its stall
   at 46 m/s is 1.6× an Earth equivalent's, because of the 25 m/s²
   gravity. Is the Tern too tender, or too stiff?
2. **Storm seas.** At 24 m/s the fully developed sea is 4.8 m. The planet's
   storms are local, so fetch-limiting (`max_hs_m`) may be wanted. Its
   default is off until someone has sailed one.
3. **Keys.** G for board and leave, and T for moor, are proposals.
