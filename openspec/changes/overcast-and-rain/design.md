# Design: one cover, four readers

## The one number

`Weather.cover` is the field's cloud cover over the player, 0 to 1. It already
drives the cloud slab's threshold (`sky.cloud_slab.y`) and it is the same number
that decides whether it is raining (`rain_cover_min`). Every part below reads
it and nothing else, so the ground cannot go stormy while the sky stays clear,
which is the bug Tenebris's own comment names.

It is the cover over the PLAYER, applied to everything in view, which is
Tenebris's model too (`FRAME_OVERCAST`). A clear valley seen from under a storm
is dimmed with it. The honest answer is a cloud shadow per fragment, marching
the slab toward the sun from the ground; the sky shader already has that march
(`cloud_shadow`). It is named here and held, because the per-player overcast
is what fixes the picture the owner sent and costs nothing.

## 1. Overcast light

Knobs into `weather.ron` with Tenebris's shipped values, validated in 0..1:

| Knob | Value | Applies to |
| --- | ---: | --- |
| `overcast_sun_dim` | 0.72 | terrain direct sun, water specular, grass and foliage direct |
| `overcast_amb_dim` | 0.48 | terrain fill and ambient floor |
| `overcast_sky_blue_cut` | 0.75 | sky Rayleigh scattering |
| `overcast_sky_haze` | 0.6 | sky Mie scattering (additive gain) |
| `overcast_sky_dim` | 0.6 | sky sun intensity |
| `cloud_fog_add` | 0.35 | distance haze density per unit cover |
| `rain_fog_mult` | 1.6 | distance haze density while raining |

`cover` goes to the terrain in `params.weather.z`, a spare lane, so the uniform
does not grow. The terrain's `direct` is multiplied by
`1 - cover * overcast_sun_dim` and its `fill` by `1 - cover * overcast_amb_dim`.
Because the sun loses more than the fill, the contrast between a lit face and a
shaded one collapses too: at full cover the sun is 0.28 of itself and the fill
0.52, which is the flat grey light of an overcast day rather than a dim sunny
one. The sky's three scattering coefficients take the dome terms exactly as
Tenebris's `renderer.rs` modulates them, so the sky and the ground dim together.
The haze density is `0.00036 * (1 + cover * cloud_fog_add) * (raining ?
rain_fog_mult : 1)`.

Test: the knobs load, validate and match the code defaults
(`the_shipped_ron_matches_the_code_defaults`, extended); the shader carries the
lane. Capture: the ground in rain at the spawn, whose light ratio to the clear
ground should fall well below the 0.725 the darkening alone gives.

## 2. Wet ground that is water

**The sheen is replaced, not tuned.** The current term is non-positive on every
upward face by construction (the proposal shows why), so no value of
`rain_sky_sheen` can make a ring bright. What replaces it is what water does:

- **Fresnel reflection of the sky.** `F = 0.02 + 0.98 * (1 - cos)^5` against the
  view, reflecting the sky colour along `reflect(-view, wet_n)`: the same sky
  gradient the distance haze already uses, taken through the overcast dim.
  Mixed as `mix(color, sky, F * wet * skylight)`. Looking down at a puddle you
  see the ground through it; at a grazing angle the puddle mirrors the grey
  sky, and a ring is the mirror bending.
- **A puddle mask.** Water stands where a face is flat and low: `flatness`
  (the face's `dot(n, up)` past 0.9) times a value noise over ~0.4 m cells,
  thresholded by `wetness` so puddles grow as the ground soaks. Rings and the
  full mirror live in the puddle; outside it the face is wet (darkened,
  saturated, F0 0.02 at a quarter of the weight) with no rings.
- **Grass is not a floor.** The `grassy` codes take the darkening and a
  quarter-weight mirror, no puddles and no rings; stone, sand, dirt and snow
  take puddles. Walls keep the rivulets.
- **No sun glint under cloud.** The glint takes the same
  `1 - cover * overcast_sun_dim` as the sun, so in a storm it is gone and in a
  passing shower it sparkles.
- **Wet darkens and saturates.** `rain_wet_darken` stays 0.72 (Tenebris's
  number); the darkening moves toward the albedo's own hue rather than toward
  grey, `color * mix(1, 0.72, wet)` with a saturation lift of the same weight.

New knobs: `rain_puddle_scale_m` (0.4), `rain_puddle_share` (0.35, of a flat
face at full wetness), `rain_mirror_strength` (1.0). `rain_sky_sheen` is
removed, not kept beside its replacement: a knob that no longer does anything is
where the next reader stops looking.

Capture: the ground in rain, where the blue ratio should rise from 0.37 to
the red and green ratios (the ground is no longer losing sky light), and a
grazing shot of a flat stone floor where the puddles mirror the sky.

## 3. Clouds that read

The cloud numbers are `Vec4::new(CLOUD_RADIUS, 0.72, 0.52, 0.045)` and
`Vec4::new(CLOUD_THICKNESS, _, _, 0.34)` in `sky.rs`: threshold, extinction,
night floor and base darkness, as Rust constants. They move into `weather.ron`
(`cloud_threshold_clear`, `cloud_threshold_overcast`, `cloud_extinction`,
`cloud_night_floor`, `cloud_base_dark`) so they are tuned where the rest of the
weather is.

Then three changes, each measured:

- **The clear threshold comes down** so the average place (0.18 cover) has
  fair-weather cumulus. The target is a share of the sky, measured the way the
  0.0% was: 15 to 30% grey at no forcing, closing to 90% or more at full.
- **A storm's base goes dark.** The slab's `lit` floor falls with cover, which
  is Tenebris's "storm clouds tower into big dark masses" (`rain_cloud_scale`
  1.7) said as light rather than as size: an overcast base is grey, a raining
  one is slate.
- **Extinction up** so a crossing of solid cloud is opaque. The current
  optical depth of a vertical crossing is about 1.6; the target is 3, so a
  full overcast hides the sun.

Capture: the sky at `--rain 0`, `0.5` and `1` from the ground, with the share
printed beside each.

## 4. Rain that can be seen

- **Near streaks to the detail range.** `shower_radius_m` from 18 to
  `rain_detail_range_m` (150), with the count scaled by area and thinned with
  distance the way Tenebris's `rain_lod_far_frac` (0.25) does, so the near
  field stays as dense as it is and the far field does not cost a hundred
  times more. Width stays a half-width of 0.012 m, which is Tenebris's
  convention and ours.
- **Far curtains.** One translucent camera-facing sheet per raining field cell
  in view past the detail range, spanning the cell's width from the ground to
  the slab, tinted `rain_impostor_{r,g,b}` (0.55, 0.58, 0.62) at
  `rain_impostor_alpha` (0.5), cross-faded over `rain_lod_blend_m` (60 m) so a
  curtain never pops. Which cells rain is `pbd_core::weather`'s own answer,
  sampled on the cell lattice in view, so a curtain is exactly where the field
  says it rains. Drawn in the same mesh as the streaks.
- **Lens drops stop at 200 m.** `rain_lod_alt_m` gates the lens term and the
  near streaks, as in the reference; from above that the storm reads through
  the clouds, the curtains and the fog.

Test: curtains are placed on every raining cell in view and on no dry one, on
the pure field; the lens gate is a pure function of altitude. Capture: a storm
seen from 400 m outside it.

## What is held

- A per-fragment cloud shadow on the ground (the march exists in the sky
  shader).
- Wind: streaks slanted by a wind field, which lands with grass sway.
- Snow as its own particle; the field already answers where it falls.
- Puddles that persist and drain on their own clock: today they follow
  `wetness`, which already lags the rain by `wet_fade_tau_s`.

## What the build did differently, and why

Written after the build, against the plan above, so a reader can tell the
design from what shipped.

- **The overcast dims the sky's share of the fill, not the floor under it.**
  `AMBIENT_FLOOR` is what keeps a cave legible, and a cave is not darker for a
  cloud over the hill it is dug into. The fill term is
  `max(AMBIENT_FLOOR, sky_fill * (1 - cover * overcast_amb_dim))`.
- **The ground's sky colour is one function, `ground_sky`**, used by the haze and
  by the wet mirror, greyed by `overcast_sky_blue_cut` and dimmed by
  `overcast_sky_dim` exactly as the dome is. That took the terrain uniform's
  `rain` array from four vec4s to six. The hand-typed 416-byte size test that
  guarded it is replaced by one that asks naga for the `Params` struct's size
  in BOTH terrain shaders and holds the Rust struct to it.
- **The haze thickens by rain intensity, not by a raining flag**:
  `mix(1, rain_fog_mult, rain)`, so the edge of a shower does not step.
- **Rain kept its near disk and gained a lattice beside it.** Widening the disk
  to 150 m would have meant tens of thousands of streaks a frame to hold its
  density, and it would still only ever draw rain falling on the camera. Rain
  seen from outside it is Tenebris's shape instead: a lattice of 60 m cells
  FIXED TO THE BODY (`pbd_core::weather::lattice_near`, so a curtain belongs to
  a place and does not slide as the camera moves), every cell the field says is
  raining within `rain_range_m` (1400 m) drawn as shafts of streaks inside the
  detail range and one grey curtain beyond it, cross-faded over the blend band
  (`cell_lod`). New knobs: `rain_cell_m`, `rain_range_m`, `rain_cell_density`
  (Tenebris's 52 streaks per 30 m cell), `rain_cell_width_mult` (its 6x, reached
  linearly from 1x at the camera, or a shaft beside you reads as a pole) and
  `rain_max_cell_streaks` (its 9000 cap).
- **The rain colours are display values.** Tenebris writes its colours straight
  to the screen; Bevy's vertex colours are linear. Taken as linear, the 0.55
  grey curtain drew at 196/255 and read as a white wall behind the trees, so
  `weather.ron`'s rain colours are converted from sRGB on the way into the mesh.
- **The lens gate also stops the sea's rain rings above 200 m**, because both
  read one lane (`fx.z`). At that height a ring is sub-pixel.
- **`cloud_extinction` is per metre.** The shader's hidden `* 0.012` is gone
  and the knob carries it: the old 0.52 is 0.00624 per metre, an optical depth
  of 1.6 over the 260 m slab.
- **The weather was never sampled on foot.** `sample_field` asked for the one
  `Camera3d`; walking keeps an inactive flight camera beside the walker's, so
  the query failed every frame and the weather stayed at the launch forcing.
  It takes the active camera now, as the shower already did, and logs the
  cover under it whenever it moves a twentieth.
- **`--weather-at SECONDS`** starts the field that far into its own time, a
  measurement instrument: the spawn is a wet meadow at cover 0.59 at launch,
  and the average sky the tuning targets (0.18) is there 2,520 s in.
- **Curtains are feathered.** Hard-edged 60 m sheets stood as panels with
  seams; each is now a cell and a half wide with its outer halves fading to
  nothing, so neighbours blend into one wall of rain.
