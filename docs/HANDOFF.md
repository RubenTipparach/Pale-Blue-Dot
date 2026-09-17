# Pale Blue Dot: session handoff

Written to be read with no access to this repository. It carries what was
measured, what was decided, what is still open, and the order to take it in.

Everything numeric below was measured off the two source trees (`Pale-Blue-Dot`
and `tenebris-rs`) rather than derived or remembered. Where a figure came out
differently than first reported, the corrected value is the one here.

## Implementation update: 2026-09-17

The user has now requested implementation; the historical "keep planning"
answer below no longer describes the current authorization. This pass covers:

- Shared gameplay gravity: 25 m/s^2 at 1 g, full through 1.4 R, tapering to
  zero at 1.8 R; walking, flight and hover assistance query the same core rule.
- A body-local surface camera and clip transform, with the atmosphere shell
  following the same frame and a translated-scene capture instrument.
- GPU frustum culling and separate terrain/nearby-foliage indirect draws.
- The live surface shader's 0.25 night-side rim floor.

- The five-system water port (cap pass, composite, wetness, precipitation,
  flow hook), with `water.ron` and `weather.ron` as the tunables.
- **Hexagon LOD and the rescale, built.** The body is **4,800 m** with a
  level-7 base for the whole globe and levels 8 to 11 resident as bands of
  2,400, 1,200, 600 and 300 m around the player, so the tile underfoot is the
  gold-standard **2.833 m** and the step is **1 m**. The relief is cut to
  ~150 m summits and a ~60 m ocean floor, the sky shell keeps its 1.2 R ratio,
  clouds sit at +300 m, the walker steps one cell, and trees scatter at
  Tenebris's per-biome rates. `planet::lattice` addresses any level's dual by
  `(face, level, i, j)`, `planet::lod` generates the resident set off the CPU
  height function and republishes it whole when the player has walked 40 m,
  the GPU visibility pass partitions by the coarser level's cells, and the
  surface shader closes the band boundary with fine floors, split midpoint
  cells and a cut wall. Design and decisions:
  `openspec/changes/hexagon-lod/design.md`; captures, three findings fixed
  off them (a mis-framed seam preset, trees ending at the band edge, a dark
  water horizon from the sky's solid sphere sitting above the sheet) and the
  remaining seam
  judgement, which is the owner walking a band edge in the running game:
  [tenebris-comparison.md](tenebris-comparison.md), "After the rescale and
  hexagon LOD".

RON body assets and the volumetric engine remain planned. The measurements and
"not yet acted on" findings below describe the earlier 4,000 m / level-8
baseline and are kept as the record of why; current implementation and
validation status is tracked in the OpenSpec task files and
[validation results](handoff-validation.md).

---

## 1. The project in one paragraph

Pale Blue Dot is a Rust/Bevy voxel planet game descended from **Tenebris**, an
earlier project by the same author. A planet's surface is the dual of a
subdivided icosahedron: hexagonal tiles with exactly twelve pentagons. The
current build is a **preview**: one height per surface column, a whole globe
built eagerly and drawn by GPU vertex pulling from one storage buffer. It is not
the editable volumetric voxel engine the design targets.

Two crates: `pbd-core` (engine-independent, `glam` only) and `pbd-app` (Bevy
0.18.1 + Avian 0.6.1). That dependency direction never reverses.

---

## 2. The complaint that started this

> "the camera feels really short compared to the hexagons"

Correct, and it is not the camera. The avatar was taken from Tenebris at 1:1
while the world around it was built about six and a half times larger.

### Measured

| Quantity | Tenebris | Pale Blue Dot | Ratio |
| --- | ---: | ---: | ---: |
| Sea-level radius | 300 m | 4,000 m | 13.3x |
| Subdivision level | 7 | 8 | +1 |
| Surface cells | 163,842 | 655,362 | 4.0x |
| **Tile width, mean** | **2.833 m** | **18.883 m** | **6.67x** |
| Tile width, range | 2.595 - 3.101 m | 17.298 - 20.669 m | 6.67x |
| **Vertical quantum** | **1.000 m** | **6.000 m** | **6.0x** |
| Eye height | 1.60 m | 1.60 m | 1.0x |
| Walk / sprint | 8 / 14 m/s | 8 / 14 m/s | 1.0x |
| Jump velocity | 12 m/s | 12 m/s | 1.0x |
| **Surface gravity at 1 g** | **25.0 m/s^2** | **9.0 m/s^2** | 0.36x |
| Jump apex | 2.88 m | 8.00 m | 2.8x |
| ...in that world's cell heights | 2.9 | 1.3 | |
| **Fall through one cell height** | **0.28 s** | **1.15 s** | **4.1x** |
| Atlas tile across a cap | 2.83 m | 28 m | 9.9x |
| Atlas tile down a wall | 1.00 m | 18 m | 18x |
| Texel at the surface | 8.8 cm | 87.5 cm | 9.9x |
| Tree height | about 6 m | 28 - 45 m | about 5.5x |

Terrain relief, measured rather than estimated (400,000 sampled directions for
Pale Blue Dot; constants for Tenebris):

| | Tenebris | Pale Blue Dot |
| --- | ---: | ---: |
| Highest peak above sea | 40 m (`MAX_LAND_HEIGHT`) | **+432 m** |
| Deepest ocean | 24 m (`MAX_OCEAN_DEPTH`) | **-516 m** |
| Total relief | 64 m | **948 m** (10.8% of radius) |
| Column height | **128 layers**, sea at index 64 | n/a - one height per column |

### Read in units of the player

| In eye heights (1.6 m) | Tenebris | Pale Blue Dot |
| --- | ---: | ---: |
| Width of one hexagon | 1.8 | **11.8** |
| Height of one terrain step | 0.63 | **3.75** |
| Height of one tree | 3.8 | 21 |
| Size of one art pixel | 0.055 | 0.55 |

A hexagon is under two people wide in Tenebris and nearly twelve here. A terrain
step is something you walk up there and a wall over twice your height here. The
same 12 m/s jump clears 2.9 one-metre blocks there and 1.3 six-metre steps here,
and the fall takes four times as long, so the walker also floats.

**The floatiness is a second, independent defect.** Fixing the tile size alone
leaves it, because 9.0 m/s^2 against a 6 m step is slow whatever the tiles do.

---

## 3. The arithmetic everything rests on

### Tile width

Measured mean tile width on a body of radius `R` at subdivision level `L`:

```text
width = 1.2087 * R / 2^L
```

The constant was measured on the unit sphere. Tenebris's own
`subdivisions_for_radius` writes it as **1.05**, which is about 15% low; its
clamp at level 7 is what actually decides its tile sizes, so the error never
surfaced there. **Do not port that function as the rule.**

### The nesting property

Subdivision **appends** midpoint vertices after the existing ones, so a vertex
keeps its index forever. Measured:

```text
L2->L3: first     162 of     642 vertices identical, at the same index
L3->L4: first     642 of   2,562 vertices identical, at the same index
L4->L5: first   2,562 of  10,242 vertices identical, at the same index
L5->L6: first  10,242 of  40,962 vertices identical, at the same index
L6->L7: first  40,962 of 163,842 vertices identical, at the same index
```

`tenebris-rs`'s `goldberg.rs` states this as a contract it must never break,
because those indices appear in saves and wire data. Consequences:

- **A coarse LOD tile is a PREFIX of the fine cell array.** Nothing to merge,
  nothing to remesh, no second mesh to keep in step.
- **A coarse tile's height is free**: its centre IS a fine cell's centre, so its
  height is `heights[i]` from the array that already exists.
- Only the **corner rings** are per level, and summed over all levels that is
  `4/3` of the finest level alone.

### Why a uniform level cannot serve a big planet

Holding the tile at the standard locks radius to level:

```text
R = 300 m * 2^(L - 7)   ->   300, 600, 1200, 2400, 4800, 9600 m
```

| level | radius | diameter | cells | topology at 128 B/cell |
| ---: | ---: | ---: | ---: | ---: |
| 8 | 600 m | 1.2 km | 655,362 | 80 MiB |
| 9 | 1,200 m | 2.4 km | 2,621,442 | 320 MiB |
| 10 | 2,400 m | 4.8 km | 10,485,762 | 1.3 GiB |
| 11 | 4,800 m | 9.6 km | 41,943,042 | 5.0 GiB |
| 12 | 9,600 m | 19.2 km | 167,772,162 | 20.0 GiB |

---

## 4. Decisions made this session

All seven are settled; four further answers follow in 4.8. They are recorded in `CLAUDE.md` and in the change
proposals; this is the summary.

### 4.1 One hex size on every body

**Standing rule.** A cell is the same size on every planet. Digging a hex of dirt
on one world and finding different-sized hexes on the next is the bug this
prevents: a cell is a unit of material, and a unit that changes size between
worlds is not a unit.

The definitive spec is `tenebris-rs`, and the gold standard is its **main
Tenebris planet** (radius 300 m at level 7). Its other bodies are prototype
stage and Sequoia is still under development; neither is a reference.

| | |
| --- | ---: |
| Tile width, mean | **2.833 m** |
| Tile width across the sphere | 2.595 - 3.101 m |
| Cell height | **1.000 m** |

The +/-9% spread is geodesic distortion, the same shape on every body, and is
not a tolerance to spend.

### 4.2 Gravity: `tenebris-rs` is definitive

| | |
| --- | ---: |
| Surface gravity at 1 g | **25.0 m/s^2** |
| Full pull out to | **1.4 R** |
| Linear taper to zero by | **1.8 R** |
| Fields | **two**, sharing one surface constant |

Taken as-is rather than retuned for a larger body: gravity is felt at human
scale, not planet scale, and a walker who steps off a ledge should fall the same
way on every body for the same reason a hex is the same size on every body. A
body that wants to feel different says so with `gravity_g`.

### 4.3 Radius: 4,800 m at level 11

Keep a ~4 km-class body rather than shrink to fit a uniform level. **4,000 m is
not on the ladder** and cannot hold the standard at any level:

| radius | level | tile width | verdict |
| ---: | ---: | ---: | --- |
| 4,000 m | 10 | 4.721 m | too coarse |
| 4,000 m | 11 | 2.361 m | 17% under, outside the spread |
| **4,800 m** | **11** | **2.833 m** | **exact** |

4,800 m is *larger* than 4,000, lands the standard to the digit, and its 9.6 km
diameter sits inside the authored catalog's 5 - 12 km range.

**This makes hexagon LOD load-bearing, not an optimisation.** A uniform level 11
on that body is 42 million cells and 5 GiB of topology.

### 4.4 Hexagons at every distance

The far tier is hexagons, never a triangle-mesh impostor. The JS ancestor did
this (`createDistantLODMeshes` builds three more Goldberg polyhedra at
`subdivisions - 2/-3/-4`); `tenebris-c` and `-rs` gave it up for an icosphere
impostor, and the seam between a hexagon world and a smooth ball is what the
owner does not want.

**One-way to the GPU.** Topology and heights upload once; the level per tile,
the cull and the draw arguments are computed in a compute pass and consumed by
an indirect draw. Nothing about a tile comes back to the CPU, and no per-tile
visibility or level state is maintained per frame on the CPU.

### 4.5 LOD level comes from spherical distance to the player

A tile's level is quantised from its **great-circle distance to the player**,
with band thresholds stored as cosines so no trig runs per tile:

```text
cos_angle = dot(tile_direction, normalize(player_pos - body_centre))
T         = the band cos_angle falls into
```

One dot product and a few compares per tile. Two properties make it work:

- **Neighbours agree by construction.** `T` is continuous in the tile's own
  direction, so adjacent tiles land in the same band unless they straddle a
  threshold - and a threshold is a **circle of known radius**, not an arbitrary
  boundary.
- **The player, not the camera.** Camera-anchored bands would re-shuffle whenever
  the player merely looks around. Player-anchored bands move only when the player
  moves.

Still open: what closes the ring (skirts are the leading candidate, reusing the
surface pass's existing terrain-step trick), the thresholds themselves,
hysteresis as the player walks across a boundary, and the twelve pentagons.

### 4.6 Write it up before touching code

Investigate, measure, put the finding and the plan in `openspec/`, stop. Editing
source is a separate step on a separate request. The one exception is a
measurement instrument - code whose only purpose is to produce a number the
write-up needs - and that is still a code change.

### 4.7 Process: OpenSpec, and one rules file

- `CLAUDE.md` is the only copy of the engineering rules. `AGENTS.md` is a pointer
  to it and must stay one.
- `openspec/specs/<capability>/` is **what the engine does**, every requirement
  pinned by a passing test.
- `openspec/changes/<name>/` is **what is designed but not built**.
- Move a requirement from a change into `specs/` in the same commit that makes
  it true and adds the test proving it. Never ahead of one.
- `openspec validate --all` passes before a push, alongside `cargo fmt --check`,
  `cargo clippy -D warnings` and `cargo test --workspace`.

---

### 4.8 Four further answers

| Question | Answer |
| --- | --- |
| Finest LOD tier extent | **~300 m** great-circle from the player |
| Terrain relief after the rescale | **~100-150 m peaks** |
| Per-body config format | **RON assets** (serde + a Bevy asset loader) |
| What to build first | **Nothing yet - keep planning** |

**The 300 m fine tier is cheap and is this project's own idea.** Tenebris has no
radial render distance to port: it meshes the entire planet as one chunked mesh
whenever the body is active and culls only on a horizon test, with its single
distance number (`MAX_DETAIL_DIST_M = 5000.0`) applying to the whole body.
Measured cell counts on a 4,800 m body at 2.833 m tiles:

| fine tier extent | cells | topology at 128 B |
| ---: | ---: | ---: |
| 124 m (standing horizon) | 6,950 | 0.8 MiB |
| **300 m** | **40,670** | **5.0 MiB** |
| 600 m | 162,520 | 19.8 MiB |
| 1,200 m | 647,543 | 79.0 MiB |

1,200 m of full detail costs the same 79 MiB the preview currently spends on the
whole globe at 18.9 m tiles. 300 m clears the standing horizon (124 m) with 2.4x
margin, which matters because a band boundary sitting exactly at the visible
horizon is the worst place for a seam.

**The relief target wants a flag.** ~100-150 m peaks sit between Tenebris's
absolute 40 m and the ~640 m that scaling its proportions to a 4,800 m body would
give. It is a defensible middle, and it implies a column roughly twice
Tenebris's 128 layers. If matching Tenebris matters more than the figure, the
number changes and nothing downstream is built yet.

## 5. Findings not yet acted on

### 5.1 There are three shader families, and the faithful one is not running

| Job | Tenebris GLSL 410 | PBD standalone port | PBD live |
| --- | --- | --- | --- |
| Terrain | `hex.vs`+`hex.fs`, 76+351 lines | `hex_terrain.wgsl` 117 | `planet_surface.wgsl` 194 |
| Water | `water.vs`+`water.fs`, 28+311 | `water.wgsl` 162 | a 12-line branch in `planet_surface.wgsl` |
| Bound to a pipeline | yes | **no** | yes |

`hex_terrain.wgsl` is a faithful port with every knob a uniform. The live
`planet_surface.wgsl` reimplements a subset with every knob a literal, and drops
the night-side rim floor, the ambient floor, torch light, underwater absorption
and the Bayer cutout. Four differences show in a still frame:

1. **The night limb goes black.** Tenebris keeps `distant_rim_floor: 0.25` in
   `lod.yaml`, which is the `0.25 + 0.75 * day` in `hex.fs`. The live rim is
   multiplied by `daylight` outright.
2. **Lighting is flat across a whole 19 m tile.** Tenebris interpolates sky light
   per vertex; ours is `@interpolate(flat)` per column. At 2.8 m tiles that is
   nearly free; at 18.9 m it is the main reason the ground reads as faceted
   plates.
3. **There is no second planet.** Every colour, threshold and falloff is inline.
4. **The ocean is two shaders.** `water.wgsl` has refraction, path-length
   absorption, foam and an underwater path; the one that renders is a Fresnel
   and two sines.

   Measured against the whole Tenebris water system and captured at 1.6, 10,
   50, 200 and 1,000 m above the polar shore (`--view shore --height N`; see
   `tenebris-comparison.md`, "Water: five systems in Tenebris, one branch
   here"). Tenebris's water is **five systems**: the cap pass, the composite
   pass (underwater fog with a dry/straddling/submerged tri-state, screen
   distortion, rain-on-glass lens droplets, emerge drips), terrain wetness in
   `hex.fs` (wet sheet, impact rings, rivulets, sheen, glint), world-space
   precipitation (`weather_fx.rs`), and the CPU flow simulation
   (`world_water.rs`, which feeds the cap's flow UVs). `water.wgsl` ports the
   cap pass only and drops five of its terms: rain ripples, flow advection,
   the waterfall scroll, the two foam weights and the specular sun tint. Pale
   Blue Dot has none of the other four systems in any form; there is no
   post-process pass at all. So "bind the port" is the first water step, not
   the whole of it, and the rest is a systems list in dependency order:
   composite, then a weather field with a rain intensity, then flow. Two more
   things the captures show: the hexagon mosaic in the water is the terrain's
   flat-per-cell depth, not a water bug; and above 800 m the sky goes black
   because `ATMOSPHERE_RADIUS` is `R + 800`, 1.20 R against Tenebris's 1.24 R,
   which moves with the rescale.

   **Built since, on the same branch, on the owner's "implement all of
   that":** the cap pass is bound (`planet_water.rs`, cap pulled from the
   `Cell` record, the two omissions fixed), the composite node runs compose,
   cap and lens with a CPU submersion tri-state, the terrain draws its seabed
   and carries the `hex.fs` wetness block, a `Weather` resource with `--rain`
   and the P key drives ripples, wetness, lens droplets and a near-shower
   streak mesh, and the flow hook reads a zero buffer. Knobs are
   `assets/config/water.ron` and `weather.ron`. Three look findings for the
   owner's eye were in `tenebris-comparison.md` under "Status after
   implementation"; the owner called the shine and the self-overlap, and both
   are fixed there with a private sheet depth buffer, a `detail_fade` on the
   wave normal and re-authored reflection values. One real bug came out of
   the shower not drawing: the globe was queued in the transparent phase at
   `f32::MAX`, which Bevy sorts LAST, so it painted over every transparent
   mesh in front of it. It is `f32::MIN` now. The mosaic in the water is gone
   with the inline branch.

### 5.2 A confirmed latent bug: the camera is not planet-local

`planet.rs` sends the **world** camera to the surface shader, and both sides then
treat the world origin as the planet centre:

```rust
camera_position.length() - PLANET_RADIUS          // foliage cutoff
```
```wgsl
let altitude = max(length(params.camera.xyz) - params.settings.x, 0.);
```

Correct today only because the body sits at the origin. Offset it and `altitude`
becomes distance-from-world-origin, so the fog gate `exp(-altitude/1050)` and the
rim gate `1 - air` both saturate: terrain renders with its haze and limb stuck at
their far values everywhere.

Tenebris documents its own camera uniform as PLANET-LOCAL for exactly this
reason, and its notes record what the same mistake did to a second planet's
water: fog distance reading ~20 km per fragment, and Fresnel, specular and
Snell's-window angles all resolving against a garbage view vector. It was
mis-diagnosed and "fixed" by turning the atmosphere off before the real cause was
found.

**This is cheap now and a bug hunt later.** It is invisible until the day a
second body exists, which is the worst possible day to find it.

### 5.3 Per-body look is data there and constants here

Tenebris drives a body's whole appearance from YAML, one section per body:
`atmosphere.yaml` alone carries an on/off toggle, both shell radii, Rayleigh and
Mie scales, sun intensity, scale height, Mie asymmetry, the **wavelength ratios
that are the sky hue**, four sunset vectors and a fog tint. Its second planet is
a brown-sky world made by `rayleigh_scale: 2.1` and `wavelengths: [18, 10, 4.5]`
against the first's blue `[5.6, 9.5, 19.6]`, **with no shader edit between them**.

PBD's sky is five `vec4`s of Rust constants for one planet.

One caveat: Tenebris's "inherit" sentinel is a **zero value**, and this
project's own rules reject that - a body that genuinely wants a rim intensity of
zero must be able to say so. Take the per-body override with a global default;
make the mechanism an explicit optional.

### 5.4 Smaller, noted

- Tenebris routes sky scattering, distance fog, clouds **and** precipitation
  through one `body_has_atmosphere` predicate so they cannot drift. PBD states
  the rule and names no predicate, so it is four places that happen to agree.
- `atmosphere.yaml`'s `enabled: 0` is a **diagnostic**: it strips sky and fog and
  leaves everything else, which isolates whether the atmosphere is tinting the
  water.
- `walking.rs` writes the gravity falloff curve a second time inline, so the
  walker and the ship each carry their own copy.
- **`planet_visibility.wgsl` binds `clip_from_world` and never reads it.** The
  only test is the sphere-horizon one, so there is no frustum, far-plane or
  distance culling anywhere: standing on the ground submits the whole visible
  hemisphere, about 327,000 cells.
- **The foliage cutoff wastes vertices.** `FOLIAGE_DRAW_CUTOFF_ALTITUDE` is one
  global altitude switch, so below 3,200 m every visible cell is submitted at 162
  vertices and the tree branch discards beyond 2,300 m *after* submission, as
  degenerate triangles - about 108 wasted tree vertices per cell over hundreds of
  thousands of cells. The per-cell distance test exists; it runs too late.
- **The project has no runtime config of any kind.** No `assets/config`, no
  serde, no custom asset loader; every tunable is a Rust `const`. This is in open
  tension with `CLAUDE.md`'s own rule about tunable values living in validated
  data.
- 92 of the 128 bytes per cell are pure topology (direction, six corner rays),
  identical for every body at a level, and each corner ray is shared by three
  cells. Deduplicating gets a cell to roughly 50 bytes and buys about one level
  on the radius ladder.

---

## 6. The work, and the order to take it in

Five change proposals, all documentation, none started. Each is a directory
under `openspec/changes/` with `proposal.md`, `design.md`, `tasks.md` and spec
deltas.

| # | Change | What it settles |
| --- | --- | --- |
| 1 | **`hexagon-lod`** | hexagons at every distance, level and cull decided on the GPU |
| 2 | **`preview-scale-and-shader-parity`** | the rescale to 4,800 m and 1 m steps, the two shader families, the night rim, the literals |
| 3 | **`gravity-model`** | two fields, 25 m/s^2, the `1.4 R`/`1.8 R` bands, the floaty walker |
| 4 | **`per-body-rendering`** | the planet-local camera bug, per-body look as data, one atmosphere predicate |
| 5 | **`voxel-engine-foundation`** | the volumetric engine: chunks, streaming, durable edits, caves |

### Recommended order

1. **Finish the LOD seam.** The level-selection half is decided (4.5). What is
   left is what closes the ring, the band thresholds, hysteresis and the
   pentagons - see `openspec/changes/hexagon-lod/SEAM-BRIEF.md`, which states
   the remainder on its own and is written to be handed over cold. A grazing-angle
   still frame of two adjacent bands settles it; prose will not.
2. **The planet-local camera fix** can go any time, independently. It is small,
   it is a real bug, and it gets harder to find the longer it waits.
   `sky_atmosphere.wgsl` already does it correctly - line 77 subtracts the planet
   centre - so this is extending an existing pattern, not inventing one.
3. **The frustum cull and the per-cell foliage distance**, which are independent
   of LOD and already paid for: the matrix is bound and the distance test
   exists.
4. **Hexagon LOD**, once the seam is settled.
5. **The rescale** to 4,800 m and 1 m steps, which drags with it the atmosphere
   shell, the cloud layer, the foliage range, the draw-budget switch, the terrain
   amplitude, the tree geometry and the atlas UV divisors.
6. **The gravity model**, and collapse `walking.rs`'s duplicate falloff onto the
   core well while in there.
7. **Shader and water parity.** The night rim floor is one line; lifting the
   literals into the params uniform is a refactor that should keep the shipped
   values byte-identical on the first pass. Then bind the faithful `water.wgsl`
   port as the single water system above and below the surface, with its scene
   colour/depth inputs, and remove the live inline approximation. Fixed-camera,
   fixed-time captures on both sides of the surface settle parity.
8. **The voxel engine**, which is the whole remaining game.

### What a visual change needs

Green tests do not settle a look. A rendering change wants a before-and-after
capture the owner has seen. The repository has a headless capture harness and a
benchmark script; raw run output belongs where it was produced, not committed.
Two selected proof plates are committed as the explicit baselines requested for
this handoff: `docs/screenshots/water-parity-baseline.png` pairs the native
Tenebris and live PBD water captures, and
`docs/screenshots/shader-parity-baseline.png` fixes the current orbit/night
views. They are labelled as baselines, not presented as completed parity.

---

## 7. Corrections made during the session

Recorded because a reader may have seen the earlier numbers.

- **Tenebris's surface gravity is 25 m/s^2, not 9.81.** `gravity_g: 1.0` on its
  main planet multiplies `SURFACE_GRAVITY_MPS2_PER_G = 25.0`, arcade-scaled to
  its ~300 m planets. The jump apex is **2.88 m**, not the 7.34 m first reported,
  and the gravity ratio is 0.36x, not 0.92x.
- **OpenSpec 1.13.0 does not overwrite `AGENTS.md` or `CLAUDE.md`.** An earlier
  concern that `openspec init` would fight the pointer layout was wrong; it
  writes `.claude/` and `.agents/` skill directories instead.
- **4,000 m cannot hold the hex standard at any level**, which is why the radius
  decision landed on 4,800 m rather than simply keeping what was there.
