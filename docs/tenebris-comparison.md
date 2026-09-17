# Tenebris reference and the Bevy visual prototype

## Reference captured from the actual application

The reference below is a native Tenebris framebuffer capture, not an illustration or a recreated scene.

![Tenebris native orbit reference](screenshots/tenebris-orbit.png)

The first capture attempt used the existing release executable at `C:/Users/santi/repos/tenebris/tenebris-rs/target/release/tenebris-client.exe`, dated 2026-06-27. That executable did not contain the `TENEBRIS_DEV_SHOT` or `TENEBRIS_DEV_LODCAM` hooks and produced no PNG. The attempt was stopped, and the existing debug executable dated 2026-07-20 was used instead. No Tenebris build was performed and no source file in that repository was modified. The source checkout inspected during the capture was at `82b562718b9ec3a5365654d6ec0d776f4b744334`; the exact source revision of the prebuilt executable is unknown.

The successful capture used the native `dev_shot.rs` framebuffer path with these inputs:

| Setting | Value |
| --- | --- |
| Executable | Existing `tenebris-rs/target/debug/tenebris-client.exe` |
| Working directory | `C:/Users/santi/repos/tenebris/tenebris-rs` so its own assets resolve |
| Window launch | Hidden background process |
| World | `pbd-reference-20260915-orbit-debug` |
| Camera | `TENEBRIS_DEV_LODCAM=650`, aimed at the planet by the source hook |
| Capture | `TENEBRIS_DEV_SHOT=<workspace>/output/captures/tenebris-orbit-debug/orbit.png` |
| Save isolation | Child-only `APPDATA=<workspace>/output/captures/tenebris-orbit-debug/isolated-user-data` |
| Stable copy | `orbit_reference.png`, 1280 × 720 |
| Committed reference copy | `docs/screenshots/tenebris-orbit.png` |

The LOD hook disables normal SP autosaving in the inspected source. Save isolation additionally keeps the capture's profile/world store separate from the user's existing Tenebris save data. Only the processes started for these captures were stopped. Logs and isolated data remain under their capture directories for review.

A second capture used `TENEBRIS_DEV_DIVE=-12`, `TENEBRIS_DEV_TIME=0.46` and clear weather in another isolated world/store. The deepest-ocean camera is on the dark side at that global time; the script's “noon” preset is not local noon at every longitude. This is a useful wave-detail/night-water reference, not a daylight color reference.

![Tenebris native near-water reference](screenshots/tenebris-water.png)

The surface capture shows fine, continuous reflective wave ripples over very dark water, with a blocky forest silhouette beyond the curved shoreline. Its low-light result reinforces that the prototype needs a chosen sunlit landing viewpoint for material review, rather than treating a global clock preset as a camera lighting guarantee.

The two actual native captures are also fixed side by side below as the water
parity **baseline**. This is evidence of the starting mismatch, not an after
image and not a claim that parity has already been achieved.

![Water parity baseline from actual native captures](screenshots/water-parity-baseline.png)

## What the captured reference actually looks like

- Terrain keeps small, high-contrast pixels at the coast: pale sand borders saturated green vegetation and deep blue water. This visible material contrast matters more than continuous physically based roughness.
- The planet is a readable blocky globe. Cloud silhouettes are chunky squared puffs rather than photorealistic volume noise.
- A nearly black space background carries many small white pixel stars, with a small neighboring moon visible to the right.
- The terminator is strong. The night hemisphere is very dark, while the sunward coast and ocean highlights are bright.
- The atmosphere forms a broad, soft pale-blue/white limb. Parts of this capture's limb and water highlights are overexposed; that is observed behavior, not a requirement to reproduce the clipping.

The runtime reference planet in this shot has radius about 300 m according to its log. The new prototype's radius is 4,000 m. Compare silhouette, materials, lighting and atmosphere rather than claiming that the same 650 m camera altitude gives equivalent framing at both scales.

## Integrated sky material

`crates/pbd-app/src/sky.rs` exports `SkyPlugin`, `SkyMaterial`, and shared environment constants. The plugin registers a real Bevy 0.18.1 `MaterialPlugin` and spawns an atmosphere sphere. The application must add `SkyPlugin` to its visual scene; the headless simulation does not acquire rendering implicitly.

| Contract | Current value |
| --- | --- |
| Planet center | Local-world origin |
| Ocean/solid reference radius | 4,000 m |
| Atmosphere shell radius | 4,800 m, 800 m above sea level |
| Cloud layer radius | 4,600 m, 600 m above sea level |
| Exponential density scale height | 176 m; smooth fade over the outer 28% of the shell |
| Sun direction | Normalized `(0.65, 0.75, 0.35)` |
| Shader asset | `assets/shaders/sky_atmosphere.wgsl` |
| Blend/depth | Premultiplied alpha, depth writes off, reverse-Z GreaterEqual |
| Camera contract | Bevy view position and planet center in the same local world frame |

The integrated shader imports Bevy's mesh output and view binding. It takes the default Bevy mesh vertex stage, traces the shell from the current camera, and adapts the standalone Tenebris atmosphere integration to sixteen view/four sun samples. Its fragment math includes soft planetary shadowing, Rayleigh and Mie terms, warm dusk, an altitude-dependent daylight veil that hides stars near the ground, and sparse quantized cloud patches. It renders front faces from orbit and back faces from inside the shell, discarding the other side so the sphere is not blended twice.

The planet renderer owns ocean geometry and its water shader. There is no duplicate transparent sea sphere in `SkyPlugin`. Its water prototype uses ocean-depth coloration, procedural small waves, Fresnel response and sun glints within the planet surface pass. The more detailed standalone `water.wgsl` remains a separate port; this integration does not claim screen-space depth refraction simply because that standalone asset exists.

The new sky is deliberately restrained at the limb, using a thin scale-height density profile over a larger geometric shell. The revised positive terrain elevations are halved before 6 m quantization, with a sampled peak near 426 m; the cloud layer at 600 m and air shell at 800 m clear those observed peaks. Blue scattering ratios, lower Mie strength and reduced radiance target the initial bright-white rim. Terrain edges and pixel colors should stay readable beneath it. Cloud patches are an inexpensive stylized approximation on a spherical layer, not the source's individual cube clouds or a volumetric cloud simulation. The material assumes the planet is at the stated local origin; moving/rebasing the planet requires updating its center uniform and mesh transform together.

## Native prototype capture history

The first release desktop build produced actual `pbd-orbit.png`, `pbd-coast.png` and `pbd-surface.png` screenshots under `output/captures/`. These images were inspected against the Tenebris reference. They demonstrate that GPU column compaction/indirect drawing, the Bevy-composed sky material, textured hex/pentagon surfaces, ocean shading, cosmetic trees and HUD rendered in the application. They used subdivision 7: 163,842 cells and approximately 20 MiB of uploaded column data.

The first orbit view showed a complete textured globe and a blue limb, but mountain tops protruded above the original 4,260 m shell, and the original 4,108 m clouds lay below much of the terrain. Extensive snow/rock materials produced the bright upper hemisphere; it was not all atmospheric glare. The surface view showed textured stepped columns and trees beneath blue sky. The original coast camera was 700 m above local ground and outside most of the dense air, so black space above its horizon was a camera-altitude effect.

The revised source uses subdivision 8: **655,362 columns**, approximately **80 MiB** of column storage, and roughly **19 m** average cell width. Draws above 3,200 m camera altitude use **54 vertices per visible column**; nearer views use **162** to include cosmetic tree cubes. Actual revised orbit, coast, surface, night and pole captures were produced and inspected. They show clearer coastal greens/sand, a shell above the mountain peaks, visible cloud patches and finer surface columns. The original screenshots remain historical evidence for the first build, not measurements of this configuration.

Night-side review found sharply separated diagonal atmosphere bands. These were consistent with too few integration samples and binary shadow rejection, rather than the intended cloud pattern. The integrated WGSL was refined to sixteen midpoint samples and a smooth 36–60 m planetary penumbra; its interfaces and Rust material stayed unchanged. The final native recapture shows a smooth limb with those diagonal wedges removed. All views were rerun after this change. The standalone source-mapped atmosphere shader retains the original eight-view/four-sun march.

## Final native views

![Pale Blue Dot native orbit view](screenshots/pbd-orbit.png)

The orbit view preserves the reference's saturated land, dark blue ocean, pixel stars and small moon. The revised air shell clears the mountains, and blue scattering leaves the green and rocky surface legible. Its wispy cloud patches differ visibly from Tenebris's block-shaped clouds; they are a deliberate inexpensive approximation.

![Pale Blue Dot native coast view](screenshots/pbd-coast.png)

The coast view separates turquoise shallows, pale sand and green inland terraces. The brighter sunlit camera makes the water/material transition easier to assess than the captured Tenebris night-water reference. Water retains warm sun glints, but uses the simpler documented ocean shader rather than the standalone refraction port.

![Pale Blue Dot native surface view](screenshots/pbd-surface.png)

The surface view shows pixel textures, stepped hex columns and block-shaped trees below blue sky and elevated clouds. The approximately 19 m cells remain visibly large; this is a whole-globe surface preview, not the planned metre-scale editable voxel world.

![Pale Blue Dot native night view](screenshots/pbd-night.png)

The night view retains a dark hemisphere and a bright twilight rim, with a warm ocean highlight on the lit side. The final atmosphere has a continuous shadow transition after the sampling/penumbra correction. Separate pole and completed survey-tour captures also rendered the closed globe without missing seams in the reviewed views.

The committed orbit and night frames are paired below as the baseline for the
planned night-rim and uniform refactor. Any eventual after capture must use the
same views rather than relying on a differently framed beauty shot.

![Live shader orbit and night baseline](screenshots/shader-parity-baseline.png)

## Measured dimensional comparison

The complaint that started this section was that the camera "feels really short
compared to the hexagons". It is correct, and the reason is not the camera. The
avatar was taken from Tenebris at 1:1 while the world around it was built about
six and a half times larger.

Every number below is read out of the two source trees. The tile widths are
measured rather than derived: `planet::tile_widths` walks the real
`dual_sphere(8)` and reports the centre-to-centre spacing of every neighbouring
pair, which is the flat-to-flat width of the shared tile. The startup log prints
it, and `measured_tile_width_follows_the_subdivision_law_and_pins_the_shipped_scale`
pins it, so this table cannot quietly drift from the code again.

| Quantity | Tenebris | Pale Blue Dot | Ratio |
| --- | ---: | ---: | ---: |
| Sea-level radius | 300 m | 4,000 m | 13.3x |
| Dual subdivision level | 7 | 8 | +1 |
| Surface cells | 163,842 | 655,362 | 4.0x |
| Tile width, mean | 2.83 m | **18.88 m** | 6.67x |
| Tile width, range | 2.59 - 3.10 m | 17.30 - 20.67 m | 6.67x |
| Vertical quantum | 1.00 m | **6.00 m** | 6.0x |
| Eye height | 1.60 m | 1.60 m | 1.0x |
| Walk / sprint | 8 / 14 m/s | 8 / 14 m/s | 1.0x |
| Jump velocity | 12 m/s | 12 m/s | 1.0x |
| Surface gravity at 1 g | **25.00 m/s^2** | 9.00 m/s^2 | 0.36x |
| Atlas tile across a cap | 2.83 m (one tile per face) | 28 m | 9.9x |
| Atlas tile down a wall | 1.00 m (one tile per face) | 18 m | 18x |
| Texel at the surface | 8.8 cm | 87.5 cm | 9.9x |
| Tree height | about 6 m | 28 - 45 m | about 5.5x |

Sources: `planet.rs::BODIES_INIT` and `subdivisions_for_radius` (which caps at 7
and is what the runtime actually calls, not the level in the body table),
`world.rs::BLOCK_HEIGHT`, `player_ctrl.rs::EYE_HEIGHT`/`WALK_SPEED`/`SPRINT_SPEED`,
`config.rs::jump_velocity_mps` on the Tenebris side; `planet.rs::SUBDIVISIONS`,
`planet_terrain.rs::PLANET_RADIUS`/`ELEVATION_STEP`, `walking.rs::EYE_HEIGHT` and
`WalkConfig` on ours. The atlas figures are the `/28.0` cap divisor and the
`/18.` wall repeat in `planet_surface.wgsl` against Tenebris's one atlas tile per
block face; the texel is that over the 32-pixel tile grid both use.

### What the ratios mean in the frame

Expressing the two world scales in units of the player makes the complaint
arithmetic rather than taste:

| Read in eye heights (1.6 m) | Tenebris | Pale Blue Dot |
| --- | ---: | ---: |
| Width of one hexagon | 1.8 | **11.8** |
| Height of one terrain step | 0.63 | **3.75** |
| Height of one tree | 3.8 | 21 |
| Size of one art pixel | 0.055 | 0.55 |

A hexagon is under two people wide in Tenebris and nearly twelve people wide
here. A terrain step is something you walk up there and a wall well over twice
your height here. That is the whole of the effect: the eye is at a human
1.60 m in both, so the only thing that changed is everything else.

It also changes what the same jump input does, and gravity is the second half
of that. Both worlds fire the player off at 12 m/s, but Tenebris's surface
gravity is **25 m/s^2**, not Earth's: `SURFACE_GRAVITY_MPS2_PER_G` is 25.0,
arcade-scaled on purpose to its ~300 m planets, and the comment there says so.
Ours is 9.0. So the apex is 2.88 m there and 8.00 m here.

Against a 1 m block that clears **2.9 steps**; against a 6 m quantum it clears
**1.3**. Terrain Tenebris lets you hop over is a cliff here. And the fall is
where it is felt most: dropping one step height takes **0.28 s** there and
**1.15 s** here, four times longer, so a walker who can only just clear one step
also floats down from it.

### Against this project's own target

The `voxel-engine-foundation` change's design already specifies the intended
surface: near-player
radial layers 1 m high and a lateral cell area of roughly 1 to 4 square metres,
which it works out as level 12 on a 4 km body, `4*pi*R^2/cells = 1.198 m^2`, or
about 1.18 m across. The prototype runs level 8. So it is **16x coarser
laterally and 6x coarser vertically than the design it is a preview of**, and
the gap is a known one rather than a regression. The doc says so in passing
already ("coarse surface columns, not the one-meter microvoxels specified
below"); what was missing is that a reader could not tell how coarse without
measuring it.

Level 12 is not reachable the way the prototype is built. It is 167,772,162
cells, and at this pipeline's 128-byte topology record that is **21.5 GB** for
one globe, before heights, meshes or collision. The architecture doc's own rule
covers it: the engine must never allocate the whole detailed globe.

### What can actually be done about it

Three options, cheapest first. None of them is a camera change: the eye is
already at a human height, and raising it only turns the walker into a giant
without making a tile smaller.

1. **Shrink the preview body.** Tile width is `1.209 * R / 2^L`, so at level 8
   a radius of **600 m** lands Tenebris's 2.83 m tiles and **250 m** lands the
   architecture doc's 1.18 m. This is one constant, keeps the globe closed and
   eagerly built, and costs nothing per frame. What it costs is the 8 km world:
   flight altitudes, the atmosphere shell at 4,800 m, the cloud layer at
   4,600 m, the 2,300 m foliage range, the 3,200 m draw-budget switch and the
   terrain amplitude all scale with the radius and would have to move together.
2. **Scale the avatar to the world.** For the same tiles-per-second and
   steps-per-jump as Tenebris the walker wants an eye around 10.7 m, walk and
   sprint around 53 and 93 m/s, and a jump velocity near 28 m/s. Cheap and
   self-consistent, and it abandons the premise of a person standing on a
   planet.
3. **Keep level 8 as the far tier and stream a finer near-player grid.** This
   is what the `voxel-engine-foundation` change already plans in its
   chunk/radial-slab section, and it is the only option that gets metre-scale ground on an 8 km
   world. It is also the whole remaining engine.

Option 1 is the honest one for a preview whose stated job is the whole-globe
silhouette. It is recorded here as a measurement and a set of options, not
applied: rescaling the body moves nine other tuned numbers and every capture in
this document, and that is a decision to take deliberately.

## Measured gravity comparison

Tenebris runs **two gravity fields for two modes**, anchored to one shared
surface constant. Pale Blue Dot runs one field. That is the substantive
difference, and the surface number differs too.

| | Tenebris | Pale Blue Dot |
| --- | --- | --- |
| Surface gravity at 1 g | **25.0 m/s^2** (`SURFACE_GRAVITY_MPS2_PER_G`) | 9.0 m/s^2 (`planet_at_origin`) |
| Fields | two: an anchor field and an orbital field | one |
| Anchor field | full pull to `1.4 R`, linear taper to zero at `1.8 R`, then nothing | n/a |
| Space transition | outside `1.8 R` the query returns no body, which IS `is_in_space` | none |
| Orbital field | sum of true inverse-square from EVERY body, plus a central star | single well |
| Multi-body | strongest pull wins, so a moon beats its parent when you hover over it | one well, no contest to resolve |
| Interior | clamped at `0.5 R` so a query at the centre cannot blow up | linear to zero at the centre, uniform density |
| Exterior falloff | inverse square, uncapped (orbital field) | inverse square, uncapped |

### The surface number, and why it is not Earth's

Tenebris's `SURFACE_GRAVITY_MPS2_PER_G` is **25.0**, not 9.81, and the comment
beside it says why: it is arcade-scaled to that project's ~300 m planets, and
the orbital field anchors to the same value so space and surface physics agree
at the surface. One shared constant, deliberately, rather than two.

Ours is 9.0 on a body thirteen times larger. Combined with the identical 12 m/s
jump both projects use, that is:

| | Tenebris | Pale Blue Dot |
| --- | ---: | ---: |
| Jump apex | 2.88 m | 8.00 m |
| ... in steps of that world's cell height | 2.9 | 1.3 |
| Fall time through one step height | 0.28 s | **1.15 s** |

So the walker here jumps nearly three times higher in metres while clearing
less than half as much terrain, and takes four times as long to come down. The
floatiness is a second defect independent of the tile size: fixing the hex scale
alone would leave it, because 9.0 m/s^2 on a 6 m step is slow whatever the tile
is doing.

### Two fields is the part worth copying

The split is not redundancy, it is two different jobs:

- **`gravity_at`** picks the single anchor body and returns a multiplier that is
  1 inside `1.4 R`, tapers linearly to 0 by `1.8 R`, and is absent beyond. It
  drives the surface basis, the atmosphere shell, the walker's down, and the
  flip into 6DOF flight. A hard edge is exactly what a *mode* boundary wants:
  there is a definite radius where you stop being on a planet.
- **`orbital_gravity_at`** sums true inverse-square pull from every body and the
  star, with no cutoff. It is what makes a prograde burn raise a real transfer
  arc and an unpowered coast fall toward the nearest mass.

A single uncapped inverse-square field, which is what we have, cannot express
the first job. There is no radius at which it says "you are in space now", so
the walker/flight handoff has to be decided somewhere else, by something that
can drift out of step with the field.

Tenebris's tie-break is worth keeping too, and its comment records the bug that
produced it: pick the strongest pull rather than the nearest centre, because
Crag's well overlaps Quartz's and the moon you are hovering over should win even
when the parent planet's centre is closer.

### What is NOT a difference

Both use inverse square outside the body and both guard the centre. The interior
models differ in shape (uniform-density linear here, a `0.5 R` clamp there) but
neither is reachable in normal play, and ours is the more principled of the two.

### Status

This is a measurement, not a decision. Whether Tenebris is the definitive spec
for gravity the way it now is for hex size is the owner's call; see the
`gravity-model` change for what adopting it would mean.

## Measured shader comparison

There are three shader families in play, not two, and knowing which is which is
most of the answer to "why does it not look like Tenebris".

| Job | Tenebris, GLSL 410 | Our standalone port, WGSL | Our live prototype, WGSL |
| --- | --- | --- | --- |
| Terrain | `hex.vs` + `hex.fs`, 76 + 351 lines | `hex_terrain.wgsl` 117, `hex_faces.wgsl` 71, `voxel_light.wgsl` 50 | `planet_surface.wgsl` 194 + `planet_visibility.wgsl` 55 |
| Water | `water.vs` + `water.fs`, 28 + 311 | `water.wgsl` 162 | a 12-line branch inside `planet_surface.wgsl` |
| Sky | `atmosphere.vs` + `atmosphere.fs`, 17 + 166 | `atmosphere.wgsl` 102 | `sky_atmosphere.wgsl` 147, a Bevy material |
| Bound to a pipeline | yes | **no** | yes |

`shader-port.md` already records that the standalone ports are unbound and that
the live modules are a simpler prototype rather than the port. What that table
does not say, and what matters when comparing pictures, is that **the faithful
Tenebris port and the shader that actually renders are different shaders**, and
the faithful one is the one that is not running. Everything below follows from
that single fact.

### Terrain, term by term

| Term | Tenebris `hex.fs` | Port `hex_terrain.wgsl` | Live `planet_surface.wgsl` |
| --- | --- | --- | --- |
| Terminator | `smoothstep(lo, hi, dot(radial, sun))`, uniform band | same, uniform band | same shape, band inline as `(-0.13, 0.20)` |
| Ambient | `ambient * max(0.05, mix(night, 1, bright) * sky)` | same, floor is `settings.y` | `(0.16,0.21,0.27) * mix(0.12,1,day) * sky`, **no floor** |
| Direct | `max(dot(n,sun),0) * bright * sky` | same, times `sun.w` | same, sun colour inline as `(1.12,1.03,0.87)` |
| Block / torch light | `torch.rgb * torch.w * v_torch_light` | `rgb * ambient.w`, 4 bits per channel | **absent** |
| Light is sampled | per **vertex**, interpolated | per **face**, flat | per **cell**, flat across a 19 m tile |
| Underwater absorption | `exp(-absorption * depth)`, then mix to water colour | identical | **absent**, surface tint only |
| Cutout foliage | 4x4 Bayer screen door | identical | **absent** |
| Limb rim | `fres^p * (0.25 + 0.75*day) * intensity * (1 - ff^2) * sky` | identical, every term a uniform | `pow(...,4) * day * (1 - air) * 0.55`, **night floor dropped**, every term a literal |
| Distance fog | its own composite pass | not ported | `(1 - exp(-d * 0.00036)) * air * day`, mixed at 0.55 |
| Rain wetness | ripples, rivulets, sheen, glint; about 150 lines driven by `weather.yaml` | not ported | absent |
| Mining cracks | `hex_fs_crack[10]` and a crack texture | not ported | absent |
| Per-planet palette | `lod.yaml`, one section per tileset | uniforms | one planet's palette inline |

Four of those differences are visible in a still frame:

1. **The night limb goes black.** Tenebris keeps a night-side rim floor, and it
   is a designer knob rather than an accident: `lod.yaml` carries
   `distant_rim_floor: 0.25`, which is the `0.25 + 0.75 * day` in `hex.fs`. The
   live rim here is multiplied by `daylight` outright, so the unlit edge falls
   to zero. `hex_terrain.wgsl` has the floor and is not the shader running.
2. **Light is flat across a whole tile.** Tenebris carries sky and torch light
   as vertex attributes and interpolates them, so a wall grades from its lit top
   to its shaded foot. Ours is `@interpolate(flat)` per column, so an 18.9 m
   hexagon is one brightness and every tile edge is a hard step. At Tenebris's
   2.8 m tiles a flat sample is nearly free; at ours it is the single biggest
   reason the ground reads as faceted plates instead of terrain, and it
   compounds the scale problem measured in the section above.
3. **There is no second planet.** Every colour, threshold and falloff in the
   live shader is a numeric literal, so a second tileset means editing WGSL.
   Tenebris answers this with a `lod.yaml` section per tileset (water colour and
   depth, sky-reflection tones, rim colour, fog colour and scale), and the
   standalone port already exposes all of them as uniforms. Our own
   `CLAUDE.md` asks for "tunable values in validated data with units, and one
   source for defaults"; the live shader is where that is not yet true.
4. **The ocean is two different shaders.** `water.wgsl` is the real port:
   screen-space refraction with depth reconstruction, absorption over the
   reconstructed path length, Fresnel between horizon and zenith reflection
   tones, foam by height and by slope, an underwater back-face path, and
   distance fog, all as uniforms. The ocean that renders is a depth tint, a
   Fresnel to the fourth, two sines and a specular to the 160th.

### Water, term by term, and at five heights

The ocean that renders is the 16-line branch in `planet_surface.wgsl`
(lines 166-181). It is captured here at five camera heights above the polar
shore, with `--view shore --height N` (the `shore` preset walks east from 72 N
to the first water cell, lifts the eye N metres above the last land cell and
aims N metres out to sea, so every frame looks down at about 45 degrees):

| Height | What the live water shows |
| ---: | --- |
| 1.6 m | A flat, opaque teal sheet to a hard horizon line. No surface relief, no reflection of the sky or the coast, no foam or wet band where sea meets land. The only texture is the "wave" term, and it resolves as 3 m **rectangles**, because the wave phase is sampled at `floor(position / 3) * 3`. |
| 10 m | The hexagon mosaic appears **in the water**: the depth tint is per cell, so shallows are a tiling of flat hexagons rather than a gradient. The 3 m rectangles are legible as rectangles. |
| 50 m | The mosaic dominates the frame; each cell is one flat tone, stepped at every edge, the same defect the terrain section records for skylight. |
| 200 m | Reads well at this range: a depth gradient from the sand into deep water, the coast, the snow cap. The cell mosaic is still visible but no longer the subject. |
| 1000 m | **The sky is black.** `ATMOSPHERE_RADIUS` is `PLANET_RADIUS + 800.0` (`sky.rs:22`), so at 1,000 m the camera is outside the sky shell and looks back at a lit limb under stars. The water is a uniform teal with one broad specular. |

The 1,000 m frame is a finding about the sky rather than the water. Tenebris's
main planet sets `radius_mult: 1.24` in `atmosphere.yaml`, which is 72 m of
atmosphere on a 300 m body; ours is 800 m on 4,000 m, or 1.20 R. Proportionally
the two agree within a fifth. In absolute terms, a walker's jump apex or a 1 km
survey flight is inside Tenebris's shell and outside ours only because the
bodies differ by 13x in radius, and this is one more number that moves with the
rescale rather than a separate defect.

Term by term, across the three implementations:

| Term | Tenebris `water.fs.glsl` | Port `water.wgsl` (unbound) | Live branch in `planet_surface.wgsl` |
| --- | --- | --- | --- |
| Geometry | its own water mesh at sea level, one sheet per chunk | its own vertex stage, expects a water mesh | the **cap of each water cell** at `R + max(height, 0)`, so a flat hexagon at `R` |
| Wave field | 3-octave gradient-noise fbm, hash constants 374761393 / 668265263 / 1274126177 / 1103515245 | identical fbm and hash constants | product of two sines on a 3 m-quantised position |
| Vertex displacement | 3 sines, 0.18 / 0.12 / 0.06, scaled by `swell_amplitude` 0.5 m, faded off steep faces | identical | none |
| Normal | fbm gradient x12.5, projected off the face, slope capped by `slope_max` 1.6 | identical, cap is `refraction.z` | the **radial** direction; the wave only nudges the specular dot by `0.008` |
| Depth source | scene depth reconstructed per fragment, so shallows grade continuously | identical, via `reconstruct_local` | `-height` of the cell, `@interpolate(flat)`, so depth is one value per hexagon |
| Refraction | screen-space offset from the gradient, clamped to `refract_max_uv` 0.03, rejected if it lands on sky or in front of the surface | identical | none |
| Absorption | `exp(-absorption * path_length)` along the reconstructed ray, `[0.60, 0.20, 0.10]` per metre | identical | `exp(-depth * 0.028)` mixes two fixed colours |
| Underwater view | back-face path, deep colour by camera distance, Snell's-window edge | identical | none; the cap is opaque from below |
| Fresnel | Schlick, `0.02 + 0.98 * (1 - n.v)^5`, floored by `sky_horizon_strength` 0.5 | identical, floor is `horizon_color.w` | `(1 - radial.v)^4` against the radial, not the wave normal |
| Reflection colour | horizon to zenith by reflected-ray height, from `lod.yaml` per tileset | identical, uniforms | one literal `(0.09, 0.22, 0.34)` |
| Foam | by crest height and by slope, `foam_crest_lo/hi` 0.35 / 0.60, `foam_slope_lo/hi` 0.50 / 1.20, `foam_intensity` 0.10 | identical | none |
| Specular | Blinn-Phong to `specular_power` 140, `specular_intensity` 0.30, `sun_tint` | identical | to the 160th, one literal colour |
| Day/night | terminator band times `sky`, floored | identical, floor is `absorption.w` | `mix(0.18, 1, daylight)`; no separate floor |
| Torch / block light | `fs_params[13]` and baked sky | `baked_rgb_sky * lighting.z * (fresnel + foam)` | none |
| Distance fog | own composite pass, ceiling `fog_max` 0.82 | inline, ceiling `limits.x` | the terrain's haze, applied after the branch |
| Rain ripples | Zavie raindrop gradient into the normal, gated by rain intensity `fs_params[14].y` | **not ported** | none |
| Flow | `v_flow_uv` scrolls the wave field along river flow | **not ported** | none |
| Horizon fade / wet band | `horizon_fade_m` 600, `wet_fade_band_m` 0.5, `partial_band_m` 0.8, `underwater_distortion` | not ported | none |
| Configuration | `water.yaml`, 36 look knobs plus 12 flow knobs, per world | one `WaterView` uniform of 16 vec4, every knob a field | about 20 literals inline |

Three conclusions follow, and they are the ones the captures show:

1. **The port is complete for the look and missing only weather and rivers.**
   Every term Tenebris draws on a still ocean is in `water.wgsl` with the same
   constants; what it lacks is rain ripples, river flow UVs and the horizon and
   wet-band fades, all of which need inputs (a rain intensity, a flow field, a
   wetness flag) that this project does not yet have either. Binding it is the
   whole of the water task, as `preview-scale-and-shader-parity` already says.
2. **The mosaic in the water is the terrain's flat-tile defect, not a water
   bug.** The live branch reads depth off the cell's own height, and that is
   flat per hexagon by construction. Tenebris never has this problem because
   it reads scene depth per fragment; so does the port. No tuning of the live
   branch removes it, and it gets worse, not better, at the 19 m tile.
3. **The rectangles are the wave term.** `floor(position / 3.0) * 3.0` was
   meant as a pixel-art quantisation and at 1.6 m it is the only surface
   detail in view. The port's fbm has no such step.

The captures live under the session scratchpad and are not committed; the
command that reproduces each one is the height column above.

### Structural differences that are not defects

These are deliberate and worth keeping straight from the list above.

- **Vertex pulling instead of vertex buffers.** Tenebris uploads a CPU-built
  mesh per chunk carrying position, normal, UV, sky light, torch light and a
  wetness flag. We upload a 128-byte `Cell` per column and reconstruct caps,
  walls and trees in the vertex shader with no vertex buffer at all. That is
  what makes a 655,362-column globe cost 80 MiB, and it is also exactly why
  nothing can vary *inside* a tile: there are no vertices to carry it.
- **Texture addressing.** Both sample nearest. Tenebris uses a texture array
  with a nearest sampler; `pixel_tile` here uses `textureLoad` with an explicit
  floor to the 32-pixel tile grid and a 0.025/0.95 inset to dodge the sheet's
  soft seams. Ours is the more defensive of the two.
- **Language and interface.** Tenebris is desktop GL 4.1 with hand-indexed
  `uniform vec4 name[N]` arrays, by its own standing rule. Ours is WGSL under
  Bevy 0.18.1 and wgpu 27, with named structs. No parity is owed here.

### The cheap part of closing the gap

Ranked by what it costs against what it shows, and none of it is started:

1. **Restore the night-side rim floor** in `planet_surface.wgsl`: one
   `0.25 + 0.75 * daylight` in place of the bare `daylight`, matching both
   Tenebris and our own unbound port. One line, and it is the difference
   between a visible planet edge at night and none.
2. **Lift the live shader's literals into the existing params uniform.**
   `Params.settings` already carries radius, cell count, time and the vertex
   budget; rim colour, rim power, rim intensity, fog height, fog density,
   terminator band and ambient/sun tint are the ones Tenebris keeps per
   tileset. This is the step that makes a second planet possible at all.
3. **Interpolate skylight instead of flattening it**, which needs a per-corner
   value rather than a per-cell one, so it is real work on the upload side.
4. **Bind `water.wgsl`**, which needs its scene colour and depth inputs and its
   own pipeline. `shader-port.md` already lists that as outstanding.

Steps 1 and 2 are the ones that pay immediately, and neither changes the
topology or the upload. They are recorded here rather than applied: both change
what every capture in this document looks like, and a look change wants the
owner's eye on a before and after rather than a green test.

## Acceptance and comparison boundaries

The original five standalone shaders and the two integrated planet pipeline modules (`planet_surface.wgsl`, `planet_visibility.wgsl`) have explicit contracts in the dedicated Naga validation tool. `sky_atmosphere.wgsl` is explicitly deferred by name because it includes Bevy imports and material substitutions; it must be validated through Bevy's shader composer and the running visual application rather than treated as standalone WGSL. Unknown shader names are still errors, so adding a new module cannot silently skip validation.

The seven-module Naga 27.0.3 validation run passed, including the planet pipelines' semantic, binding, entry-point and storage-layout checks. Separately, the final native desktop runs compiled and rendered the Bevy sky material and the full 655,362-column globe without reported GPU pipeline errors in orbit, coast, surface, night, pole and survey-tour views. The final Rust verification run passed 35 tests, and the two scripted circumnavigation checks completed with zero protection events. These checks validate the tested prototype paths; they do not establish full parity with Tenebris or universal hardware performance.

The reviewed coast and surface views show textured grass/sand/stone, stepped hex columns, block trees, blue sky and elevated cloud patches. Orbit/pole views retain a closed globe and surrounding air layer; the ocean uses the documented simplified reflection/glint model. Future render changes should repeat these views and inspect logs for pipeline errors. Measured frame timing with hardware/configuration details is performance evidence; an illustration, HUD counter or shader parser pass cannot substitute for a benchmark.
