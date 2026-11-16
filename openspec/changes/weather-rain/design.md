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

## Splashes where rain lands (owner, 2026-11-15)

The owner: "quick note on rain, one of the things were missing are puddle
splashes on the ground, can you mock thatn up for me". Then, on how: "particle
splashing for rain, the more gpu native rain we can have the better".

Proposed, not built. The mockup is `docs/mockups/rain-splashes.html` (M32);
nothing below enters the game until the owner approves it, and then only with
the perf A/B in tasks group 6.

### What is missing today

Rain in the game today is four things, all of them flat on a surface or a
line in the air:

- the ground darkens and saturates by `wetness * skylight` (the wetness block
  above);
- puddles stand on flat floor-like faces under the puddle mask
  (`rain_puddle_scale_m` 0.4, `rain_puddle_share` 0.35), with the impact ring
  kernel in them, at half strength on grass (`rain_grass_rings` 0.5), only
  within `rain_detail_reach_m` 20 m (`wet-near-detail`);
- the near shower: up to 1,700 CPU streaks on an 18 m disk
  (`weather.rs`, `near_shower`), rebuilt into a mesh every frame;
- drops on the lens.

Nothing leaves the ground. A drop that lands draws a ring and nothing else, so
a storm on a puddle reads as a pattern on a texture rather than water being
hit. Real rain throws a small crown of droplets up from standing water and a
fainter spray off grass and wet soil, for a third of a second, everywhere at
once around the viewer. That is the missing piece.

### The proposed look

- **On a puddle, a crown.** Six droplets thrown up and out in a ring
  (0.55-0.9 m/s out, 1.3-1.8 m/s up, so 9-16 cm high), and a late central jet
  (one droplet, about 0.07 s after the impact, straight up). Gone in about
  0.35 s. The ring kernel already in the puddle reads as the crown's footprint.
- **On grass, a softer spray.** Three or four smaller, fainter droplets, low
  (3-5 cm) and short. Grass takes about half the splashes a puddle does, as it
  takes half the rings.
- **On wet bare ground** (dirt, path, boards, outside the puddle mask),
  between the two: five droplets, a little lower than a crown.
- **Small and pixel-like.** A droplet is a square of a few pixels at arm's
  length and one pixel at the reach, never an outlined sprite or a soft blob
  (CLAUDE.md, "No outlines in the world; one art style"). It is coloured and
  lit as the water it came from: the wet ground's own colour mixed toward the
  sky it mirrors, lit by the same sun, sky and lamps as the ground.
- **Lit by the lamps at night.** A droplet near a lamp catches it, as the
  puddle's own lamp highlight does; away from the lamps, under a dim moon, the
  splashes all but vanish, as they do in the real dark.
- **More with more rain.** The splash rate follows `rain` (light, heavy,
  storm); the crown's size follows how much water stands there (`wetness` and
  the puddle mask).

### The design principle: rain belongs on the GPU

The owner's direction, 2026-11-15: "particle splashing for rain, the more gpu
native rain we can have the better". The splashes are GPU particles:
spawned, moved and aged on the GPU, from a hashed seed and the time, with no
per-particle work on the CPU and no per-frame upload. Nothing about a splash
is stored; every droplet's whole life is a pure function of its lane index and
`time`, as the ring kernel's drops already are.

### How the game draws it

One instanced draw, after the opaque terrain, alpha blended, depth-tested
against the scene and not writing depth.

- **Lanes on the columns, not on a world grid.** The near list is the hex
  columns within the reach (22 m: the detail reach plus its fade): their
  centre in the render frame, the height of their topmost up-face, its
  material class (grass, floor-like, other) and its sky light. About 220
  columns at 2.833 m cells; 16 bytes each, about 3.5 KB. It is rebuilt and
  uploaded only when the camera crosses a cell or a column within reach is
  edited, never per frame. It is read from the data the near shower already
  samples (`PlanetContact`), so the splash lands where the streak does.
- **Slots and cycles.** Each column has S = 48 slots. A slot's cycle is
  `phase = time / period + hash(slot)`; each cycle it either fires (a hash of
  `(slot, floor(phase))` below `keep`) or stays dark, and when it fires its
  impact point is hashed anew inside the hex top. `keep` follows `rain` and
  the material (puddle 1, bare wet ground 0.8, grass 0.5). A splash is alive
  for about the first 0.3 s of a 0.5 s cycle.
- **Droplets in the vertex shader.** An instance is one droplet (K = 8 per
  slot), drawn as a camera-facing quad of 4 vertices. The vertex shader takes
  the instance index, finds its column, slot and droplet, evaluates the
  ballistic arc `p = impact + v*t - g*t^2/2` for the droplet's hashed launch,
  and sizes the quad to whole pixels. A dead droplet, a droplet past the
  reach, or a droplet that has fallen back below its impact height is
  collapsed to a degenerate quad. No compute pass is needed: there is no
  state to carry from one frame to the next.
- **Which kind of splash** is decided per impact point in the vertex shader:
  the puddle mask (`puddle_noise`, the same two octaves and the same quantile
  edge the terrain uses) at the impact point picks a crown, the column's
  class picks grass spray or wet ground. The crown lands exactly in the
  puddles the ground draws.
- **Lit like the ground.** The vertex shader lights the droplet with the
  ground's own terms (sky, sun, the lamp list), from the column's ground
  colour mean (`plant_means`/the tile means the terrain already carries), so
  it sits in the scene rather than on it.

### Gates

- **Rain and wetness.** No draw at all when `rain` is zero (the draw is
  skipped on the CPU from one number, not culled per vertex). The rate follows
  `rain`; the crown's share follows the puddle mask at the current `wetness`,
  so the first minute of a shower throws only spray, and the crowns come as
  the puddles form.
- **None under cover.** A column whose top face has no sky light gets no
  slots: the same baked sky light (`cell.metadata.z`) that keeps the ground
  dry under a roof keeps it splash-free. The ground under a roof shows neither
  wet nor splash; the roof's own top takes the splashes.
- **None on vertical faces.** Impacts are only ever placed on the top face of
  a column; walls, terrace sides and trunks take the rivulets they already
  have, and no splash.
- **None under water, none above the altitude gate**, exactly where the near
  shower is suppressed (`rain_drawn_at`, under water, under ground).
- **Near only.** Past `rain_detail_reach_m` (20 m) a droplet is one pixel or
  less; droplets fade over the last 4 m (the same band as the rings) and are
  not drawn past it.

### Cost estimate (an estimate, not a measurement)

| Quantity | Value |
| --- | ---: |
| Columns within 22 m | about 220 |
| Slots (S = 48, K = 8 droplets) | 10,560 slots, 84,480 droplets |
| Vertices per frame | 337,920 (4 per droplet), all run, most degenerate |
| Droplets alive in a storm | about 28,000 (4,700 splashes: keep 0.5-1 by class, 0.3 s of a 0.5 s cycle, 4-8 droplets each) |
| Pixels drawn | about 2-30 px per live droplet near, 1 px at the reach: about 0.2-0.3 M |
| Upload | 3.5 KB on a cell crossing or an edit; nothing per frame |
| CPU | one draw call; no per-particle work |

On the desktop RTX 3070, 340 k short vertex invocations (two hashes, the
puddle mask, four lamps) and 0.3 M blended pixels are expected to cost about
0.1 ms; the M4 Max laptop similar. S = 48 is what the mockup needed for heavy
rain to read as heavy (24 looked sparse); it is the first knob to turn down.

**Budget:** at most **0.25 ms of GPU time at p50 in the `storm` and
`rain-stroll` scenarios** on the desktop and on the Mac, measured old build
against new, interleaved, in one sitting (`tools/perf_suite.py`, perf-measure
rule 2). If the A/B shows more, S and K come down before anything else does.
The instance count is fixed, so the cost has a hard ceiling whatever the
weather.

### Possible follow-on, not part of this change: the shower on the GPU

The near shower is the same shape of problem: every streak is already a pure
function of its index and the time (`streak_seed`, `column_fade`), yet the
CPU rebuilds 1,700 quads into a new mesh and uploads it every frame (about
190 KB a frame at full rain). It can move to the GPU the same way: one
instanced draw whose vertex shader evaluates `streak_seed` and the fall, with
the landing height read from the same near column list. That would remove the
per-frame mesh rebuild and upload. It is recorded here as a candidate in the
spirit of the owner's "the more gpu native rain we can have the better", and
is a separate change with its own A/B; the mockup draws its streaks that way
already, which is a demonstration, not a plan of record.

## Height fog with RAIN SHADOW off (owner, 2026-11-16)

**The owner's words:** "when on the planet, if we have rain shadow turned off
can we just do normal distance fog?", and then "or height fog?".

RAIN SHADOW is the whole rain volume (CLAUDE.md, "What the owner means by
rain shadow"): the rain fog, the streaks on its shells and its lighting, the
`rain` / `rain_half` / `rain_compose` passes in `water.wgsl`, run while
`rain_needed` (`planet_water.rs`). With the setting off, which is the
default, `rain_needed` is false and a storm gets no rain murk at all: the
terrain's own haze thickens a little with the cover and the rain
(`rain_fog_mult`), and that is all. A storm reads as a dark day.

### What is drawn instead

With RAIN SHADOW off, and only then, a standard analytic **exponential height
fog** takes the volume's place:

- **Density** at a height `h` above a reference level `h0`:
  `sigma(h) = sigma0 * exp(-(h - h0) / H)`, with `sigma0 = rain *
  rain_volume_density` (the volume's own extinction per metre of full rain,
  0.003: a kilometre of full rain is an optical depth of 3) and `rain` the
  rain over the camera (`haze_weather.y`, the haze's own).
- **Height** is along the planet's local up, in the body-local frame: `h =
  |p| - R`, never world Y. The reference `h0` is the ground or the sea under
  the eye, whichever is higher (`terrain::surface_height` of the eye's
  direction, at least 0), so the fog lies on the ground where the player is:
  the valley below fills with murk (its density grows as it sinks below
  `h0`), a hilltop stands clearer, the view up thins out, and looking down
  from a height the murk is below.
- **Falloff** `H = rain_height_fog_falloff_m * (0.5 + 0.5 * rain)`: the
  layer is shallower in light rain and deeper in a storm. The knob is new in
  `weather.ron`, 250 m. (A first cut at 120 m left the horizon a third as
  murky as the volume: on a 4.8 km body a level ray rises 400 m in its 2 km,
  so the fog thinned out of it. At 250 m the sky at the horizon and the far
  sea read within a few levels of the volume's, measured on the beach shot.)
- **Integrated along the ray**, analytically: with the ray's heights at the
  eye and at its end, `h_a` and `h_b` (each `|p| - R - h0`, exact on the
  sphere), the optical depth over the length `L` is `sigma0 * L * e^(-h_a/H)
  * (1 - e^(-(h_b - h_a)/H)) / ((h_b - h_a)/H)` (the limit `sigma0 * L *
  e^(-h_a/H)` when level). Taking the height linear in distance between the
  two exact end heights is the flat-ground form laid on the sphere, and the
  error is in the curvature between them. Each end's height is held at no
  more than 2.5 `H` under `h0`, so a ray into a deep valley murks heavily but
  never overflows.
- **The ray's end** is the volume's: the scene's depth, the sea, the cloud
  base or `rain_volume_range_m`, whichever is first (`rain_far`). No fog
  above the cloud base, none under the sea, and none from an eye above the
  cloud base, where the clouds hide the storm, as the volume does.
- **Colour** is the colour the renderer it stands in for gives its rain,
  without its streaks, shadows and lamps. The shipped renderer is the volume
  (`rain_renderer: Volume`), which lights its rain by `rain_light`: the
  rain's tint times a light set by the sun's height and the cover, which is
  the sky's light at that hour (dark at night, lit by day, greyed under the
  deck). Under the shells it is their light: the tint times the sky's mean
  radiance (`rain_sky`) plus the sun through the forward phase (`rain_sun`,
  `rain_phase`).

### Where it runs: no new pass

It is folded into the end of the `clouds` composite in `water.wgsl`, the
full-screen pass that already lays the clouds over the scene and runs
whenever the volume could (a planet with air, no map overlay). That is where
the volume's light lands too (just after the clouds), so the fog lies over
the clouds as the volume did. The pass gains one branch, taken only when
`height_fog.x > 0` (a new `WaterView` lane): RAIN SHADOW off, rain at the
camera, the eye under the cloud base and out of the sea. With RAIN SHADOW
on, `height_fog.x` is zero and nothing changes, to the pixel.

### Matching the volume, so the switch does not jump

At the reference level along a level ray, the fog's optical depth is
`rain * 0.003 * L`, the volume's mean (the volume's streak term averages
about one: `0.3 + 1.4 * streak`, `streak` about one half). So the two agree
where the player stands, and differ in what the cheap one leaves out: the
volume's streaks, its sun shafts in the cascades, the lamps' glow in the
rain, and rain that is falling somewhere else than over the camera (the
volume reads the precipitation map per step; the fog takes the rain here).

### Proof

- Captures of a storm with RAIN SHADOW off, before and after, and with it
  on for comparison; and one looking over a valley in a storm.
- Its cost, in the `reflection-quality` sitting: `storm` with RAIN SHADOW
  off, the fog off (`PBD_HEIGHT_FOG=0`, a pricing instrument, the
  `PBD_REFLECT` kind: the fog's lane zero, so the frame is today's) against
  on.
