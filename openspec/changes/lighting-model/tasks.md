## 1. Look

- [x] 1.1 Capture the meadow today at 09:00, 13:00, 17:30 and 19:00.
- [x] 1.2 Mockup `docs/mockups/lighting.html` (https://claude.ai/artifact/LLzP5j8G8k3Do9j4Z7Qyaj; stills in `docs/screenshots/lighting-model/mockup-*.jpg`): the game's model now beside
  L1–L6, toggle by toggle, with time of day, cover and night lamps.
- [x] 1.3 G3 answered by the owner on the mockup, 2026-10-28: "perfect, we
  have all requirements, add this new rendering model to the game!" (all
  parts, L1–L8). G1: Khronos Neutral; G2: sun size ×12.

## 2. L1 + L2

- [x] 2.1 `sun_radiance` from the atmosphere on the CPU, with a test that
  noon matches today and sunset is redder. (`pbd_core::sky_light::sun_air`;
  the shaders compute it per fragment, a test holds their constants.)
- [x] 2.2 Sky SH-L2 with the ground bounce, with tests (constant sky gives
  constant irradiance; at noon the light from above is blue and from below
  the ground's own colour: a sunlit meadow bounces about as much as the sky
  gives, so they differ in colour, not brightness).
- [x] 2.3 Terrain, towns and field-lit PBR read both. `lighting.ron`'s
  `new_model`, or `PBD_LIGHTING=today`, draws today's model for A/B.
- [x] 2.4 Captures and a `perf_suite` A/B (`docs/benchmarks/2026-10-28-lighting`).
  Cloud flight p50: today 10.5 ms, the new shading 11.1 (+0.6), with bloom
  and the tone pass 14.1. The sky's light was first projected per call
  (+6.6 ms); it is now a table built once.

## 3. L6

- [x] 3.1 Exposure with clamps (CPU-metered, design L6), and Bloom.
  Captures `docs/screenshots/lighting-model/game-*-{today,new}.jpg`.
- [x] 3.2 Khronos PBR Neutral (owner's pick) in our own final pass, with the
  camera on `Tonemapping::None`, a dither, a test against the reference
  values, and the grade in `lighting.ron`.
- [x] 3.4 Bloom off on a Mac and a player option (owner): Settings >
  GRAPHICS > BLOOM, kept in `saves/preferences.ron`; `lighting.ron`'s
  `bloom` sets the default (off on macOS, on elsewhere).
- [x] 3.3 A cheaper bloom: Bevy's costs about 2.4 ms on the cloud flight on
  an M4 Max, the same at mip sizes 512, 256 and 128, so the cost is not the
  blur. Ours (`bloom.rs`, `bloom_lite.wgsl`): the HDR image straight to a
  quarter size through a soft Karis-weighted threshold, the dual filter down
  and up five levels, added at the bloom's strength. `lighting.ron`'s
  `bloom_lite` (default) or Bevy's; `PBD_BLOOM=lite|bevy` for A/B. Capture
  `game-bloom-none-bevy-lite.jpg`; A/B in `docs/benchmarks/2026-10-29-lighting-all`.

## 4. L3

- [x] 4.1 PCSS-lite in cascades 0–1, with sun size ×12 (owner) and cover
  (`sun_shadow_soft`): an 8-tap blocker search, a 12-tap Poisson disc as wide
  as the blockers' height times the sun's radius, slope-scaled bias per tap,
  the far cascades' 3 x 3 at three-quarter-texel spacing. The cover widens the
  sun by 2 at full deck, not the mockup's 5: at 5 the rain captures lost their
  shadows (recommendation taken, ask only with screenshots). Terrain and
  field-lit alike. Captures `game-l3-*`; A/B in the batch.

## 5. L4

- [x] 5.1 The bounce channel (`pbd_core::bounce`): sunlit faces (a walk over
  the columns toward the sun says which) seed the air beside them with albedo
  x sun x N.L, flooded five cells. Stored sparsely, a 32-layer window round
  each column's surface, 5:6:5. Baked on the async pool when the set changes
  or the sun moves 2 degrees (`planet_bounce.rs`). Tests: a sunlit floor
  lights the air over it in its colour, it stops at a solid wall, a slab
  shades the floor under it.
- [x] 5.2 The shader term (`bounce_near`), dimmed by the cloud at the face; a
  face turned up takes a fifth (most of the air over a floor was lit by that
  floor). Blended across the hex border so no cell edge shows. Captures
  `game-l4-*` (a desert's shaded walls, the overhang); A/B in the batch.

## 6. L5

- [x] 6.1 What a PBR surface mirrors (`field_lit.wgsl`, `sky_specular`): the
  sky's radiance along the reflected ray (L2, un-convolved), blurred toward
  its irradiance as the surface roughens, through Fresnel, shut out where the
  field says the sky does not reach; a metal takes no diffuse light. As
  built, no cube: what was left of the probe after L8 took the ground is the
  ship's hull, drops and fish, and rendering the land into a cube means the
  whole planet pipeline in extra views. Those surfaces mirror the sky, not
  the land (recommendation taken, ask only with screenshots).
- [x] 6.2 Movers take the SH plus the field: the held tool, drops, fish,
  vehicles and towns all draw with `field_lit.wgsl`, which reads L2's SH, the
  field, and now L3's soft shadows. (There are no townsfolk yet.)

## 7. L7 Lamp bake

- [x] 7.1 Direct lamp light (N·L, windowed inverse square, voxel line of
  sight), with tests: a lamp behind a wall adds nothing, and one in front of
  a face is brighter than one beside it (`pbd_core::lamp_bake`). As built,
  per CELL, not per vertex: the column faces are made on the GPU, so there
  is no CPU vertex to bake into. Each air cell keeps the four brightest lamps
  it can see; the shader lights each pixel from exactly those, analytically,
  blended across the hex border, so the falloff is smooth and a hidden lamp
  adds nothing.
- [x] 7.2 One bounce in the albedo's colour: the lamp light on the faces
  round each cell, flooded four cells (test: a lamp on sand bounces sand
  coloured).
- [x] 7.3 On the async pool, re-queued when the set or its light changes (a
  lamp placed, lit or put out, an edit), whole bakes only: a bake is
  published when complete, with the versions it was made from. Two halves on
  the GPU: the newest fades in over the one before over half a second (over
  the flood where none held this set); outside every lamp's reach and the
  window, the flood lights it. No partial-region test: nothing is ever
  published partially, a task returns the whole field or nothing.
- [x] 7.4 Captures `game-l7-*` (the lamp row in the desert at night, a
  torch in the cave) and the A/B in the batch.

## 8. Lit rain fog

- [x] 8.1 The rain shells' slabs take the sky's mean (L2), the sun
  scattered forward (HG g 0.55) with twelve cascade taps over the first 60 m,
  and every lamp near the eye in closed form (`water.wgsl`, the rain's group
  3 now carries the cascades). No moon yet: there is no moon key light.
- [x] 8.2 Captures `game-rainfog-{18.5,22.5}-*`; the storm scenario of the
  batch is the A/B.

## 9. Wet ground

- [x] 9.1 The glossy wet film: GGX, reflecting the real sky (L2's radiance,
  never darker than the haze's sky at the horizon), the sun's glint and every
  near lamp's highlight (the eight nearest, gated by the field's lamp light),
  specular anti-aliasing from the pixel's footprint, the sub-pixel ring fade.
  The masks are today's.
- [x] 9.2 New `weather.ron` knobs (`rain_wet_roughness`,
  `rain_puddle_roughness`, `rain_lamp_specular`), validated.
- [x] 9.3 Captures `game-wet-{13,22.5}-{today,new}.jpg` (noon storm, night
  storm by the lamps); the storm scenario of the batch is the A/B.
- [x] 9.4 Rain rings as many as it is raining: each drop of each cycle
  kept with a chance of the rain's intensity, a second offset grid above
  half; none when it stops. The sea and lakes take the same rings
  (`water.wgsl`, under the new model).

## 10. L8 Reflections: voxels first, then the screen

- [x] 10.1 The voxel walk in WGSL over the column data near the camera
  (`reflect_trace`, 72 cells, `reflection_reach_m` 48), shading the face it
  reaches (the sky by the field's light in the air before it, the sun with
  the cascades, the lamps by the field, a lit lantern glows). The column
  centres are a new binding. The walk's test is on its Rust twin
  (`pbd_core::bounce::sees`: a slab shades the floor under it); the WGSL copy
  is not run by a test.
- [x] 10.2 The screen trace where the walk finds nothing (`ssr_trace`), over
  the previous frame's colour and distances captured after the water's
  composite (`planet_ssr.rs`, a fixed 960 x 600 history, so nothing rebinds
  on resize), reprojected with that frame's matrix; loads in the loop, the
  home-pixel rule and the crossing rule.
- [x] 10.3 Wet ground and the sea read it; the key light's glint is hidden
  where the reflection finds a surface. The sea traces the same history
  (`sea_ssr` in `water.wgsl`, the compose and cap's group 3) with a longer
  reach. Captures `game-l8-desert-rain-10-*`, `game-l8-sea-shore-*`.

