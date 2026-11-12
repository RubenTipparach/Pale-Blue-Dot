## Context

Today's terrain light (`planet_surface.wgsl`, `fragment`):

```
colour = albedo * ( fill * max(0.05, mix(night, 1, daylight) * skylight * fill_dim)
                  + (1.12, 1.03, 0.87) * facing * sun_dim * shadow * skylight ) * gain
       + lamps + lightning
```

- `fill` is a constant per face kind, and `skylight` is the column field's sky
  channel times the heightfield occlusion.
- The towns' pieces use the same terms (`LitLikeTerrain`). The held tool, ship
  and fish use Bevy PBR with the field (`field_lit.wgsl`).
- The camera is HDR, TonyMcMapface, at a fixed exposure, with no bloom.
- The sun's shadow is four cascades with a 3 x 3 bilinear PCF
  (`sun_shadow.wgsl`).

Captures (`docs/screenshots/lighting-model/meadow-*.jpg`
with the before set): meadow at 09:00, 13:00 (rain), 17:30 and 19:00.

## Goals / Non-Goals

Goals:
- The time of day shows on the ground, not only in the sky.
- Shade reads as sky-lit, and is blue where the sky is blue.
- Shadows are shapes, and are harder near the caster.
- Bounce under canopies, overhangs and in doorways.
- One exposure story from noon to a cave to night.

Non-goals:
- Ray-traced or path-traced light.
- Changing the pixel art or nearest sampling.
- Per-body tuning (that is `per-body-rendering`).

## Decisions

### L1 Sun colour from the air
- The CPU evaluates the atmosphere's transmittance from the eye's ground
  point toward the sun once a frame (`sky.rs` already has the density
  model). The result is `sun_radiance: vec3` in the terrain's params.
- It replaces the literal `(1.12, 1.03, 0.87)`. It is normalised so noon
  matches today's brightness, so the change shows at golden hour and dusk,
  not at noon.
- It applies to every receiver of the sun: terrain, towns, field-lit PBR,
  clouds' sun term (already atmospheric) and water.
- Cost: none on the GPU.

### L2 Sky light probe and ground bounce
- One SH-L2 (9 RGB coefficients) of the sky's radiance: the sky model sampled
  on the CPU (64 directions is enough for L2) with the cover's grey mixed
  in. Updated when the sun moves more than 0.5 degrees or the cover changes.
- The lower hemisphere is the ground: mean ground albedo times the sun's
  irradiance on the ground, divided by pi. That is the bounce from below,
  which lifts a wall's lower half and the underside of leaves.
- The terrain's `fill` becomes `irradiance_sh(n) * skylight`. The per-kind
  constants go. The wall's paler fill happens because a wall faces the
  horizon. `AMBIENT_FLOOR` stays.
- Cost: 9 MADs a fragment.

- **Night (owner, 2026-10-28):** "There should be more bounce light at night
  with cloud cover. I don't like it being pitch dark ... the moon and star
  light would bounce lighting in the clouds giving the planet more even soft
  lighting at night".
  - A clear night's sky carries the moon's and the stars' light: a dim
    blue sky term in the SH. The moon is a second key light with its own
    shadow, dim enough that lamps read.
  - Under cover at night, the deck is lit by the moon and stars and glows.
    The sky's SH goes to an even grey-blue (about `0.026, 0.029, 0.038` at
    full cover in the mockup), brighter than a clear night's fill. The
    moon's direct light and its shadows fade out with cover. An overcast
    night is soft and even, never black.
  - The sky's colour comes from `assets/config/lighting.ron`, and the moon's
    phase can scale it later.

### L7 Lamp bake with shadows and bounce (owner, 2026-10-28)
The owner: "Can we increase the accuracy of lamps vertex color bake? Have some
bounce and shadow shading? We'll need to time slice this a bit over many
frames so we don't add too much overhead".

Today a lamp is the block channel of the flood fill: a level that drops by
one per cell around corners, read through `lamp_strength`. It has no
direction (a face turned away from a lantern is as lit as one facing it), it
has no shadow (light flows round a pillar into the space behind it), and it
does not bounce.

- **Direct, per vertex.** For every lamp within reach (16 m):
  - N·L;
  - a windowed inverse square, `1/(1 + (d/2.2)²) × (1 − (d/R)⁴)²`;
  - a voxel DDA from the vertex to the lamp, so a lamp the vertex cannot see
    adds nothing.
  - The sample point is pulled 15% toward the face centre and 3 cm off it,
    so corners do not self-shadow.
- **One bounce, per face.** 14 cosine rays to 8 m. Each ray that hits a face
  takes that face's direct lamp light times its albedo, so a lantern by a
  plaster wall lights the room cream and a lantern on grass lights its post
  green.
- **Stored** as RGB bounce plus scalar direct, 4 bytes a vertex, beside the
  column mesh. The colour tint (`TORCH_TINT`) stays in the shader.
- **Time-sliced.** A region's bake runs on the worker pool:
  - it takes whole faces until a per-frame budget runs out (mockup: 4 ms on
    the main thread; the game: off-thread, results taken once a slice is
    complete);
  - it is re-queued when a lamp is placed, lit or put out, or when an edit
    lands within reach.
- **No pop** (priority 1). Until a region's bake is complete, it shows the
  flood-fill light it has now. When it completes, it cross-fades over half a
  second, never a face at a time.
- **Cost:** GPU 0 (one more vertex attribute). CPU: the mockup's 5 lamps and
  about 9,000 faces bake in a few dozen 4 ms slices.

### Rain fog, lit (owner, 2026-10-28)
The owner: "I would add volumetric rain fog to a slider as well since that
would be interesting to see with this new lighting".

The rain volume today (`rain`, and `rain-shells`) is a grey veil. Under the
new model, the rain's in-scattering takes the new lights:
- the sky probe's mean (L2) from all round;
- the sun, or the moon at night, scattered forward (Henyey-Greenstein,
  g 0.55), shadowed by marching 12 steps of the shadow map over the first
  60 m: shafts under trees and eaves;
- every lamp in reach, by the closed-form in-scatter of a point light along
  a segment, so a lantern glows in the rain at night.

In the game, this goes into the rain shells' haze slabs. They already cut
the ray into segments, so each segment takes these terms instead of a flat
colour. Cost to measure against `rain-shells`' 1.4 ms.

### Water in the mockup
The mockup's pond is the game's water model (`water.wgsl`, `water.ron`): see
through to the lit seabed with per-channel absorption, the authored sky
gradient mirrored with Fresnel × `sky_horizon_strength`, fbm ripples with
the slope cap, and the sun glint. Its sheet covers exactly the pond's cells.
On the new side, the mirrored sky is the real one, or the L5 probe.

### Wet ground (owner, 2026-10-28)
The owner: "I never really liked the shading of our wetness shader. It looks
kinda dull".

Why today's wet ground (`planet_surface.wgsl`, the `wet_amt` block) reads
dull:
- **It reflects a fixed dim colour, not the sky.** It mirrors
  `ground_sky`: a gradient from `(0.10, 0.20, 0.29)` that greys and dims
  by 60% under cover. In rain, the cover is full, so a puddle reflects a
  flat dark grey, and the mirror only darkens.
- **The glint is not a reflection.** It is `pow(dot(wet_n, sun), 24)`. It
  asks where the sun is, not where the eye is, so it sheens every face
  turned to the sun the same from every angle. It is also cut by cover, so
  a storm has none.
- **No lamps in the water.** At night a wet street shows nothing of the
  lantern beside it. That reflection is what makes a wet night read.
- **Darkening is most of the effect.** Darken ×0.72, plus a quarter-weight
  mirror of a dark sky.

The new wet model (in the mockup; the game's masks kept as they are):
- **The same masks.** Puddles on flat non-grass floors by the noise
  quantile, rings on every wet top (lighter on grass), rivulets on walls,
  indoors dry by the field.
- **A glossy film.** GGX with roughness 0.18 when wet and 0.04 in a
  puddle, Fresnel F0 0.02, and a darker albedo in a puddle.
- **It reflects the real sky** (L2), at the roughness's blur, occluded by
  the field; under the horizon it sees ground. Not the L5 probe: one cube
  taken at one spot pasted what it saw (a full-bright lamp) onto ground
  metres away, in the wrong place, and the ripples sparkled on its edge as
  the camera moved (owner, on the mockup's halo). Lamps reach wet ground
  only through their own highlight below, and a probe never captures a lamp
  at its emissive brightness.
- **Highlights from the key light** (sun, or moon at night), widened by
  cover so a storm keeps a soft sheen, and **from every lamp** in reach.
- **As many drops as it is raining (owner).** Today every ring cell
  fires one drop a cycle, whatever the rain, and the rain only makes them
  louder; they never stop. New: each drop of each cycle is kept with a
  chance equal to the rain's intensity (picked anew each cycle), and a
  second, offset grid fills in above half intensity. A drizzle dots a
  puddle, a storm boils it, and when the rain stops the rings stop, while
  the puddle's slow waves stay. The pond takes the same rings.
- **Specular anti-aliasing.** Roughness widens by the normal's
  screen-space change, and rings fade once they fall under a pixel. Without
  it the rings shimmer as speckle.
- The knobs carry over from `weather.ron`'s `rain_*`. The new ones go
  beside them: `rain_wet_roughness`, `rain_puddle_roughness`,
  `rain_lamp_specular`.

### Rain streaks in the mockup
Rain streaks in a box that follows the eye. Today's side is grey, like the
game's shells. On the new side, each drop is lit by the sky probe, glints
toward the key light, and brightens near lamps. In the game this is the
`rain-shells` streak term taking the same lights.

### The mockup's tuning page
A "Rain and wet ground" section has the knobs above, presets (dry, passing
shower, storm, after the rain), and export and import of every setting as
JSON. It is the shape of the live tuning page (priority 7): the JSON keys
are the `.ron` field names to be.

### L8 Screen-space reflections on wet ground and water (owner)
- The owner's reference is a rained-on path: puddles mirror the sky, the
  kerb and the trees sharply, ripple rings show as bends in that mirror,
  and the wet ground around them is darker with a soft sheen.
- Wet floors (puddle mask and wet tops) and the pond march their reflected
  ray through the frame's own depth, taking the colour where it goes behind
  something. The ripples have already bent the normal, so the rings warp
  the image as in the photo.
- Off screen or hidden, it fades to the sky (L2). It also fades near the
  screen's edge and at the end of the march, never cut.
- Roughness blurs it, so a wet field gets a smeared sheen and a puddle gets
  a mirror.
- A lamp the trace sees is already in the image, so its analytic highlight
  is scaled down there (no double lamp).
- This replaces the L5 probe on the ground: one cube taken at one spot
  pasted lamps onto the wrong ground. The probe stays for the ball and the
  pond's misses.
- **Voxels first (owner).** The screen trace cannot see what the camera
  cannot see: an eave's underside, a slab's, a canopy's. It returns the
  top face's colour there instead. So the reflected ray first walks the
  block grid itself, a DDA over the voxels, up to 72 cells or 48 m. It
  stops at the face it really reaches and lights it the way the terrain is
  lit: sky by the face's openness, sun or moon with its shadow-map test,
  the lamps, and a lantern's glow. Only where it finds no block does the
  screen trace run, for what is not a block (the ball, the water). Most
  reflecting pixels then skip the screen trace, so the two together cost
  less than the screen trace everywhere. In the game the walk reads the
  column data near the camera, the same data the light field floods.
  Cost to measure.
- The mockup draws the view once more, at half size, to have that depth and
  colour. The game would read the depth prepass and the previous frame's
  colour instead (no second draw).
- Inside the march, textures are read at an explicit level. In a loop that
  can stop early, screen derivatives are undefined: Direct3D warns (X3595)
  and compiles the loop slowly. On the owner's Intel laptop the frame then
  ran long enough for Chrome to reset the GPU. Only pixels where the film
  shows a reflection are traced.
- Cost: half size, 28 steps and a 5-step refine, only on wet pixels. About
  0.5-1.5 ms at 1080p, to measure. Behind a setting, with a `perf_suite`
  A/B before it ships.

### L3 Harsher, contact-hardening shadows
- The sun-to-sky ratio follows from L1 and L2. A clear noon's shade is about
  a fifth of its sun, where today it is closer to a third.
- The filter becomes PCSS-lite in the finest two cascades:
  - an 8-tap blocker search;
  - penumbra = (d_receiver − d_blocker) × tan(sun radius × size × (1 + 5 × cover)),
    size ×12 (owner);
  - a 12-tap Poisson filter of that width, at least half a texel.
  - The far cascades keep the 3 x 3, at a tighter spacing.
  - **Each tap's bias grows with its distance and the light's slope**
    (tan of the angle off the normal × the tap's offset). A fixed bias over
    a wide penumbra lets the ground shadow itself (acne): the owner saw it
    as z-fighting speckle in the mockup.
  - **The taps' rotation is fixed to the surface** (per texture cell), not
    to the screen pixel, so the filter's grain does not swim when the
    camera moves.
- The sun's size widens under cover, so an overcast day softens its shadows
  instead of only dimming them.
- Cost: +0.3–0.6 ms at 1080p (to measure). Behind a setting.

### L4 Sun bounce in the voxel field (the game's SVOGI)
- SVOGI/VXGI (cone tracing a voxel mip pyramid per pixel) costs 3–8 ms at
  1080p on this class of GPU. Near the ground the frame is already at
  24–26 ms p95 with rain off. **Rejected on cost.**
- The column tier is already a voxel grid with a flood-filled light field
  (sky and block channels, 4 bits each). L4 adds a bounce channel:
  - Each air cell next to a sunlit face (the face's sun visibility from a
    voxel march toward the sun, on the CPU) is seeded with that face's
    albedo × sun × N·L.
  - The seed floods 4–6 cells with falloff, as lamp light does, in RGB at
    4 bits (or 5:6:5).
  - It is re-baked in slices on the worker pool when the sun moves 2
    degrees, or when an edit lands near.
- The shader adds `albedo * bounce` gated by daylight. Lamps can seed the
  same channel for coloured lamp bounce later.
- Cost: the field grows from one byte a cell to three. The bake runs off the
  main thread (`frame-budget` 3.2); GPU cost is one more read.

### L5 Probes
- **Reflection:** one 64² cube around the eye, re-rendered one face a frame
  from a cheap pass (the far terrain levels plus the sky, no clutter or
  rain), then GGX-prefiltered into 5 mips. It is read by:
  - water and wet ground, replacing the analytic sky;
  - glass, metal and the ship's hull through Bevy's environment map.
- **Light probes for movers:** ships, fish, held tools and townsfolk take
  L2's SH plus the field at their position, so they match the terrain.
  Placed probes in town interiors are later work.
- Cost: one face of 64² a frame, about 0.1–0.2 ms.

### L6 Simulated HDR, tonemapping and grade
- **Exposure, as built:** metered on the CPU (`tone.rs`), not Bevy's
  `AutoExposure`.
  - `Tonemapping::None` skips Bevy's tonemapping node, and its auto exposure
    is applied there, so it would do nothing.
  - The meter reads what the game knows at the eye: the sky's light (L2),
    the sun after the air (L1), the cover, and the voxel field's sky and lamp
    light at the camera. Walking into a cave or a house opens the eye.
  - There is no GPU readback and no histogram pass. The eye goes toward
    `log2(key / mean) x 0.62` at 2 EV a second, within -1..+2 EV.
- **Night cap, tuned on the game's captures (recommendation taken, ask only
  with screenshots):** the top comes down 0.6 EV at night, not the mockup's
  1.2. At +0.8 EV the meadow at 22:30 went near black where today's is
  legible (`game-meadow-22.5-*.jpg`). At +1.4 EV it matches today's
  brightness, cooler, with the lamp-lit grass warmer.
- **Bloom** (owner, 2026-10-28: "How about some bloom?"):
  - It is visible, not a trace. It catches lanterns, flames, the
    glow-flowers, the sun's disc and the hot sky round it at sunset.
  - In the mockup: a soft threshold from 0.75 to 2.5 (exposed), four
    widening blur passes at quarter resolution, strength 0.6.
  - In the game: Bevy's `Bloom` (energy-conserving), tuned to match those
    captures, with its intensity in `lighting.ron`.
  - **Off on a Mac, a player option (owner, 2026-10-28: "2.4 is bad for
    bloom. Id turn it off on mac and have a config in options to turn it on
    in pc").** It costs about 2.4 ms on the cloud flight on an M4 Max,
    whatever its mip size. Settings > GRAPHICS > BLOOM turns it on or off.
    The choice is kept in `saves/preferences.ron`. Before the player
    chooses, `lighting.ron`'s `bloom` decides, and left out it is off on
    macOS and on elsewhere. The Settings panel now holds within the screen
    and scrolls with the wheel; its binding list ran off both ends.
- **Tonemapper: Khronos PBR Neutral** (owner, 2026-10-28, on the mockup:
  "Kronos is the best").
  - Bevy 0.18 does not ship it. Its `Tonemapping` enum has TonyMcMapface,
    AgX, ACES fitted, Blender Filmic, Reinhard and SomewhatBoring, and no
    PBR Neutral.
  - So the camera takes `Tonemapping::None` (still HDR), and the curve,
    exposure, bloom composite and grade run in one final full-screen pass
    of our own. It is the Khronos reference function, about 15 ALU ops.
  - That pass also takes a dither (interleaved gradient noise, half an
    8-bit step), which `None` turns off in Bevy.
  - A test holds the curve against the reference's worked values (mid grey
    0.18 to 0.14, white to 1 - 0.24^2/0.44): `pbd_core::sky_light::pbr_neutral`.
  - As built: Bevy's `FullscreenMaterial` after `Node3d::Tonemapping`
    (`tone_grade.wgsl`). Bevy's bloom runs before it, on the HDR image.
  - Lantern panes and flames are drawn at `lamp_glow` (6) under the new
    model, so they sit above white and bloom.
- **Grade:** white balance, saturation and lift/gain keyed on sun elevation,
  in `assets/config/lighting.ron`:
  - slightly warm at golden hour;
  - neutral at noon;
  - neutral at night (owner, 2026-10-28: "Lighting at night not quite the
    same warmth with the new model"). Night's cool comes from the moon's own
    colour, not the grade, because a grade that cools the frame cools the
    lanterns with it. Night adaptation is capped at +0.8 EV, so lamp-lit walls
    stay under the curve's shoulder and keep their orange.
- Cost: auto-exposure and bloom together are about 0.3–0.5 ms.

### As built in the game (2026-10-29)
Every part is in the game behind `lighting.ron` (and an environment switch
for A/B runs). Where the game differs from the mockup:

- **L3** (`sun_shadow_soft`): as designed. The cover widens the sun by 2 at a
  full deck, not 5: at 5 the rain captures lost their shadows
  (recommendation taken, ask only with screenshots). `PBD_SOFT_SHADOWS=0`.
- **L4** (`pbd_core::bounce`, `planet_bounce.rs`): not a flood channel in the
  light field's byte but a field of its own: a 32-layer window round each
  column's surface, 5:6:5. That holds every face the sun can reach and
  costs about 1 MiB on the GPU, against 16 MiB for every layer. A walk over
  the hex columns toward the sun says which faces it reaches. It is baked
  on the async pool when the set changes or the sun moves 2 degrees. A face
  turned up takes a fifth of it, because the field does not keep where light
  came from and the air over a floor was lit mostly by that floor.
  `PBD_BOUNCE=0`.
- **L7** (`pbd_core::lamp_bake`): per CELL, not per vertex. The column faces
  are built on the GPU, so there is no CPU vertex to bake into.
  - Each air cell in the same window keeps the four brightest lamps it can
    see, by a walk from the cell to the lamp. The shader lights each pixel
    analytically from those lamps (N·L, the windowed inverse square), so
    the falloff is per pixel and only the shadow's edge is per cell.
  - Both terms are blended across the hex border.
  - There are two bakes on the GPU, and the newest fades in over the older.
    The flood lights what no lamp of the bake reaches, including the glowing
    flowers, which are not baked.
  - `PBD_LAMP_BAKE=0`.
- **L8**: the walk is `reflect_trace`, with the column centres as a new
  binding. The screen trace reads the previous frame's colour and distances,
  captured after the water's composite into a fixed 960 x 600 texture
  (`planet_ssr.rs`), not a depth prepass: the terrain is not in Bevy's
  prepass. `PBD_REFLECT=0`.
- **A border between two columns** is the plane midway between their
  directions, square to the step between them. A plane through the planet's
  centre hangs on a difference below f32's resolution, and comparing two
  unit directions to pick a column flipped pixel to pixel as speckle.
- **Wet ground**: L2's sky radiance is soft and dim at the horizon, where a
  puddle seen from eye height looks. The reflection is never darker than the
  haze's sky there.
  - Lamp highlights come from the eight lights nearest the eye, gated by the
    field's lamp light so none shines through a wall.
  - The sea takes the new rings.
- **Rain fog**: the rain's pass now binds the sun's cascades. There is no moon
  term, because the game has no moon key light yet.
- **Bloom**: our own (`bloom.rs`), described under L6.
- **L5**: the movers already read L2 and the field through `field_lit.wgsl`.
  As the probe, a PBR surface mirrors the sky's radiance (L2), blurred by
  its roughness, through Fresnel and the field's sky light. There is no cube
  of the land: that would draw the planet pipeline again into extra views.
- **The sea** traces the same screen history as wet ground.

## Order

1. L1 and L2: cheapest, and the biggest change to "weird".
2. L6, so judging is done through the final curve.
3. L3.
4. L4.
5. L5.

Each lands with before-and-after captures at 08:00, 12:00, 18:30, 19:30 and
22:30 in the meadow, a town and a cave mouth, and a `perf_suite` A/B.

## Risks / Trade-offs

- **Sunset too dark or too red.** The transmittance is normalised at noon,
  and the grade can pull it back.
- **Night.** L1 turns the sun off. Moonlight is L2's sky at night plus a dim
  moon term, kept dim enough that lamps read (the mockup rule).
- **Bounce leak** through one-cell walls: the flood respects solids, as
  block light does.
- **Exposure pumping** going into caves: slow adaptation, and clamps.

## Open Questions

- **G1. Tonemapper.** Answered: Khronos PBR Neutral (owner, 2026-10-28).
- **G2. Shadow hardness.** Answered: sun size ×12 (owner, 2026-10-28). Sharp
  where a shadow meets its caster, a soft penumbra a few metres out.
- **G3. Which parts to build.** L1, L2 and L6 recommended first. The owner
  sees all six in the mockup.
