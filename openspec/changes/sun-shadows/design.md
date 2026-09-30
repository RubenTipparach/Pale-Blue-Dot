# Design: the sun casts shadows

## Context

The survey behind the proposal found the following.
- **The sun.** One sun is published: `Sun`, from `pbd_core::daylight::Clock::sun()`.
  - The sky, the terrain, the water and the field-lit objects all read it,
    and a test keeps it that way (`the_app_keeps_no_sun_of_its_own`).
  - A day is 2,880 s, so the sun moves 0.125 degrees a second, unless the
    time slider or L moves it.
- **The terrain is not a Bevy material.**
  - It is `PlanetPipeline`, a vertex-pulled pipeline with indirect draws in
    `Transparent3d`, drawn from persistent cell records.
  - A per-view compute pass (`planet_visibility.wgsl`) culls those records
    against the camera's horizon and frustum.
  - Its bind groups are its own. It has no Bevy view bindings.
- **Field-lit objects** are towns, vehicles, drops and fish:
  `ExtendedMaterial<StandardMaterial, FieldLit>`.
  - Their shader runs Bevy's PBR, then scales it by `daylight * sky` and adds
    an ambient.
- **Scale.**
  - The planet is 4,800 m in radius, with summits near 300 m.
  - The render frame is planet-centred f32, and the camera uses Bevy's
    infinite reversed-Z.
  - The walker's eye is at 1.6 m. The terrain's detail bands reach 2.4 km
    and trees reach 2.4 km.
  - Flights cruise at up to 3 km, and orbit captures are taken from
    6–8 km.
- **Budget.** The owner's is 8.3 ms at p95.
  - The baseline walk is 2.2 ms of GPU time on an RTX 3070 at 1440×900.
  - The clouds dominate where there are clouds.

## Goals / Non-Goals

**Goals**
- A low sun reads from its shadows: ridges, cliffs, the column tier's
  steps, trees and houses cast them, across the view.
- Shadows hold still: no crawling edges as the walker moves or turns, and
  no shimmer as the sun moves.
- In shadow a surface keeps its sky fill, lamps and ambient: the dark is
  the sky's, never black.
- The terrain and the objects on it take the same shadow from the same
  cascades.

**Non-Goals**
- Clouds casting into the cascades (the cloud map already shades the
  ground), and shadows on the clouds.
- Shadows on the water sheet.
- Point and spot light shadows (lamps).
- Moving objects (vehicles, fish, drops, the held tool) casting. They
  receive.

## Decisions

**1. The planet's own cascades, not Bevy's.**
- One depth texture array of four cascades is drawn by a render-graph node
  of the planet's.
- It is read by `planet_surface.wgsl` and `field_lit.wgsl` through a shared
  `sun_shadow` WGSL module and one uniform of cascade matrices.
- *Alternative: Bevy's own cascades* (`shadows_enabled` and a
  `CascadeShadowConfig`), with the planet drawn into Bevy's `Shadow` phase
  and reading Bevy's view bindings. Rejected:
  - the terrain pipeline would take on Bevy 0.18's view bind group layout
    and queue items into its internal shadow phase, the coupling to
    engine internals that `lamps-and-lanterns` decision 14 turned down;
  - Bevy's cascades are set in fixed distances for a flat world, where
    ours change with altitude (decision 2);
  - Bevy's shadowed sun would still pass through the field-lit shader's
    `daylight * sky`, so the objects would take two different sun
    occlusions.
- *Alternative: horizon maps* (a baked per-cell horizon angle per
  direction). Rejected as the whole answer: cell-sized at best, and blind to
  trees and houses. They could be a far fallback later.
- *Alternative: screen-space shadows* (marching the depth buffer toward the
  sun). Rejected as the whole answer: a ridge behind the camera casts
  nothing. They could add contact detail later.

**2. Cascades by view, fitted stably** (`pbd_core::shadow`, pure and tested).
- **Four cascades.** Their far edges follow the practical split (λ 0.95)
  between 0.5 m and the reach.
  - The reach is 900 m on foot, which gives edges near 14, 43, 165 and
    900 m.
  - In flight the reach grows to the horizon's distance, `sqrt(2Rh)`,
    at most 6 km.
- **Each cascade is the bounding sphere of its slice of the view frustum.**
  So turning in place changes nothing. Its radius is rounded up to a 1 m
  step, so it only changes when the slice grows.
- **Its centre is snapped to a whole texel in the light's frame.** So a
  move of less than a texel moves nothing, and a longer one moves the
  shadow in whole texels.
- **Its near plane is pulled toward the sun to take in every caster.**
  That is anything between the slice and where the sun ray leaves the shell
  at R + 320 m (the highest summit, plus trees). For a low sun that is up
  to about 2 km.
- **The matrices are built in f64** in the planet's frame and cast to f32,
  as `clip_from_body` is.
- **2048² texels a cascade, 32-bit depth**: 64 MiB for the four.
  - On foot the texels are about 1.5 cm, 4.5 cm, 18 cm and 1 m, near to
    far.
  - The far cascade in flight is about 6 m a texel, fine for a mountain's
    shadow.
- **Refresh.** The two near cascades are redrawn every frame. The third is
  redrawn every other frame and the fourth every fourth frame, since the
  sun moves 0.125 degrees a second. A cascade not redrawn keeps the
  matrices it was drawn with, so what is sampled always matches what is in
  the texture. A time jump (the slider, L) redraws all four.

**3. What casts.**
- **The terrain, the column tier's faces and the trees.** The planet draws
  them depth only, from its own records, with a shadow vertex entry.
  - Each cascade gets its own cull: `planet_visibility.wgsl` gains a light
    view that tests the cascade's box, and skips the horizon cull.
  - Each cascade keeps the camera's detail levels, so a shadow is cast by
    the very terrain that is drawn. That leaves no gap where the two
    disagree.
- **Clutter (grass) does not cast.** At its size a shadow is noise, and
  each cascade would pay for its instances.
- **A town casts.** Its meshes' positions are one static buffer per town,
  drawn by a small position-only depth pipeline. Door leaves do not cast.

**4. What receives, and how.**
- **One WGSL function**, `sun_shadow(position, normal)`. It returns 0
  (shadowed) to 1 (lit).
  - It picks the cascade by view depth, and blends into the next over the
    last 10% of a cascade.
  - It uses a comparison sampler over a 3 × 3 tap kernel, so an edge is
    about a texel and a half soft. The mockup's is PCF-soft too.
  - The biases: a normal offset of 1.5 texels of the cascade in use, and a
    slope-scaled depth bias in the caster pipeline.
- **The terrain** (`planet_surface.wgsl`). The direct sun term is multiplied
  by it:
  `direct = max(dot(n, sun), 0) * sunlight * cloud_sun * sun_shadow`.
  Skylight, fill, the night floor and the block light are not.
- **Field-lit objects** (`field_lit.wgsl`). Decision 14's
  `sun_up = daylight * sky` becomes `sunlight * sky * sun_shadow`. So in
  shadow an object is lit by its ambient and lamps only, as a shadowed
  cliff is.
- **A face turned away from the sun** is already unlit by `dot(n, sun)`, so
  its shadow is not sampled at all.

**5. The sun ends at the horizon** (`preview-scale-and-shader-parity`
task 7, landed here).
- **Direct light, specular and glints** take
  `sunlight = smoothstep(-0.0145, 0.02, s)`, where `s` is the sine of the
  sun's elevation.
- **Ambient, fog and rim** take `twilight = smoothstep(-0.105, 0.05, s)`.

A shadow cast by a sun under the horizon would say something false. The
two curves are pinned by the existing shader-constant tests, which change
in the same commit.

**6. Frame cost, and what gives if it is over.**
- **The hypothesis** is that the shadow pass costs at most 1.2 ms of GPU
  time in the `walk` scenario on the owner's RTX 3070.
  - It is measured with `tools/perf_suite.py`, old and new builds
    interleaved.
  - Its GPU span is `planet_shadow`.
- **If it is over**, these give, in this order:
  - the fourth cascade every eighth frame;
  - 1536² cascades;
  - trees out of the far cascade.
- **Not measured in the cloud.** A cloud session has no GPU, so the
  write-up and the PR say so. The owner runs the suite on real hardware.

**7. Indoors, and faces in shade** (the owner, 2026-09-30, on the slice 2b
shots: "Why is lighting indoor so harsh?"). Three causes in `field_lit.wgsl`,
read from the code and the shots:
- **The sun reaches inside.** Bevy's sun has no shadows, so a wall inside
  a house that faces the sun is lit as if it stood outdoors. That is the
  bright white plaster in `game-2b-newel-below.png`.
- **A house does not darken its inside.**
  - The field's sky term is sampled at the eight corners of each mesh's
    bounds and blended between them.
  - A town is one mesh per texture across the whole village. Its corners
    sit out in the air, or off the lit tier, which reads as open sky.
  - The voxel sky field has never heard of a house anyway, since
    buildings are pieces, not voxels.

  So every room reads as open sky. `sun_up = daylight * sky` is about 1,
  and the dimming that makes a cave dark never applies.
- **A face turned from the sun gets almost nothing.** With `sun_up` near 1,
  the shader keeps only Bevy's picture: `pbr * sun_up + fill *
  (1 - sun_up)`. Bevy's fill is its default ambient (80, nothing is set),
  not the terrain's sky fill. That is the near-black wall in
  `game-2b-flight-foot.png`. It also darkens house fronts outdoors that face
  away from the sun.

The fixes:
- **The sky fill is always added, never blended away.**
  - Bevy's own ambient is set to the terrain's daytime sky fill, and follows
    the clock. So a face in shade is lit as a shaded cliff is, indoors or
    out.
  - It is one resource, kept in step by the shader-constant test, not a
    second copy of the sun.
- **A building's inside has its own openness.**
  - The cutter tags every face inside a building: inner wall faces, floors,
    ceilings, stairs.
  - The town draws those faces as their own meshes, with a sky of 0.3 when
    a door stands open and 0.2 when shut. Round the day, a room is about a
    quarter as bright as the street, as the mockup's rooms are.
  - Candles and hearths light rooms when slice 3 brings them.
- **The sun stays out** once the cascades land: the house's walls and roof
  cast, so its inside is in their shadow (decision 4). Until then, the
  inside meshes take no direct sun at all.

These land ahead of the cascades, in tasks 4b.1 and 4b.2, because they
need none of them.

## Risks / Trade-offs

- **Acne and peter-panning.** The biases trade one against the other.
  - The captures include a low sun on a flat field, where acne shows,
    and a thin wall, where peter-panning shows.
  - The biases are tuned against both and recorded here.
- **A cascade's seam.** The 10% blend hides it. A capture looks straight
  along a street across the seam at 40 m.
- **Terrain popping in the shadow.** The detail fade (`detail-fade`)
  dithers the drawn terrain as its level changes. The cascades draw a
  record at the level the camera draws it, so a shadow can change level
  with the ground under it. A popping shadow is looked for on the fly-in
  capture.
- **Cost at altitude.** From flight height the far cascade covers kilometres
  of terrain. The light-view cull and the far cascade's slower refresh are
  what hold it. It is measured in the `far-side` scenario too.

## Migration Plan

Shadows are drawn, derived state: no save changes. Rollback is the previous
build.

## Open Questions

None for now. Taken as recommendations, because a question goes to the owner
only with screenshots:
- soft edges of about a texel and a half;
- shadowed means sky-fill only.

Once the first captures exist, softness and depth are asked about beside
them in the survey.
