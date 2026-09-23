# Design: the storm

## 1. One cloud march, and clouds over the sea

The sea sheet is drawn by `WaterCompositeNode` after `EndMainPass`, against
the main pass's depth. The cloud shell is a transparent material that writes no
depth, so the sheet overwrites whatever cloud was blended over the pixel.

The fix is to march the clouds again where the sea is drawn, over the sheet's
colour, with the SAME code: the cloud density, shadow and slab march move out
of `sky_atmosphere.wgsl` into `shaders/clouds.wgsl` (`#define_import_path
pbd::clouds`), imported by the sky and by the water pass. The water pass
composites the slab between the camera and the sheet point over the sheet. A
ray that does not cross the slab (every ray from below the cloud base to the sea)
returns at once, so the cost is paid only from above.

The cloud numbers are the sky's: `sky::apply_weather` already writes them for
the shell, and it now also writes a `CloudNow` resource, extracted to the render
world, from which the water view copies them. One writer; two readers.

## 2. Volumetric rain

**Where it is drawn.** Rain has to be in front of the ground, which the sky
shell cannot do (it is behind everything). The after-scene pass has the scene's
colour and depth, so a new fullscreen pass, `rain`, runs after the sea cap and
before the lens: it marches each pixel's ray from the camera to the nearer of
the scene depth, the sea sphere and the cloud base, and composites rain over the
pixel. That puts it in front of the ground and the sea, and under the clouds
seen from below: a ray from under the cloud base reaches the rain before any
cloud, and the march stops at the base. From above the cloud base, and above
`rain_lod_alt_m`, it is not drawn; the storm reads through the clouds there.

**Where it rains.** The field is on the CPU (fBm moisture), so the CPU fills a
**rain map**: 64 x 64 cells of `rain_map_cell_m` (50 m) on a tangent plane at a
snapped anchor near the camera, each the field's rain intensity there (negative
for snow). It is refilled when the anchor moves or a second of weather time has
passed, and uploaded as a storage buffer; the shader projects a point onto the
plane gnomonically and samples bilinearly. The near streaks and the curtains'
lattice read the same field, so the volume stands where the streaks fall.

**What it looks like.** Density is the map's intensity times `rain_volume_density`
per metre, times an animated streak texture: value noise stretched along the
local up by `rain_volume_stretch` and scrolled down at `rain_fall_mps`, so the
volume reads as falling shafts rather than fog. It is lit like the rain quads
(`rain_light`, handed over), tinted `rain_impostor_color`, and flashed by
lightning. Twelve steps, jittered per pixel. The curtains are removed: they were
this volume's stand-in.

## 3. Snow

`Precip::Snow` columns already exist. Where it snows:
- the near shower draws flakes: `snow_color`, `snow_fall_mps` 3.0 (Tenebris),
  `snow_size_m` squares that sway, instead of streaks;
- the volume is white and falls slowly (negative map values);
- no lens drops, and the ground does not get wet (the wetness follows rain only).

## 4. Lightning

A strike is a pure function of the field time, so every client agrees without a
message: time is cut into `lightning_slot_s` slots; a slot's hash decides
whether it strikes (`lightning_chance`, scaled by how hard it is raining over
the map), and where: the heaviest-raining map cell among a hashed few. The
flash is an envelope with two or three flickers over `lightning_flash_s`.
`pbd_core::weather::lightning` owns the rule and a test holds it deterministic
and silent where it does not rain.

What it lights:
- **the clouds**: the sky adds a flash term, strongest near the strike's
  direction;
- **the rain volume**: the same term, so shafts light up;
- **the ground**: the terrain adds the flash as a cool-white ambient, falling
  off with distance from the strike;
- **a bolt**: a jagged line from the cloud base to the ground at the strike,
  in the shower mesh, while the flash is up.

## 5. A weather slider

The pause menu gains a row: a bar from clear to storm that is the storm forcing
(`StormForcing`), dragged or clicked, with Clear / Rain / Storm presets. It moves
the field, exactly as P and `--rain` do; one path.

## What the build did differently, and why

- **The volume is drawn from anywhere under the cloud base**, not only below
  `rain_lod_alt_m`. That gate was for streaks, which are sub-pixel from a
  few hundred metres; a volume is exactly what reads from there. Above the
  cloud base it is not drawn, since the clouds are in front of it.
- **The curtains' knobs are gone** (`rain_range_m`, `rain_curtain_solid`,
  `rain_impostor_color`, `rain_impostor_alpha`) rather than left beside the
  volume, since a knob that no longer does anything is where the next reader
  stops looking. The streak shafts keep their lattice inside the detail range
  and fade out over `rain_lod_blend_m`.
- **A name clash in the shader import.** The first cut exported a function
  `cloud_flash` from `pbd::clouds` while the water view carries a field of the
  same name, and the import processor rewrote both: every water pipeline failed
  to compile with "invalid field accessor". The function is `cloud_flash_at`.
  It was caught by the first capture's log rather than by a test, because the
  water and sky shaders go through Bevy's preprocessor and the raw naga check
  covers only the terrain's two; that gap is recorded in `shader_tests.rs`.
- **A near streak fades out inside four metres of the eye.** One passing a
  metre from the camera drew as a bar across the screen.
- **`--spawn snow`** starts at the nearest land where the field snows, a
  measurement instrument like `--weather-at`, so snow can be photographed.
- **The water pass took a `WaterSky` parameter struct.** The volume took
  `prepare_water_views` to seventeen parameters, one over Bevy's limit.
