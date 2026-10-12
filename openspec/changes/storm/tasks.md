# Tasks

## 1. Clouds over the sea
- [x] Cloud march into `shaders/clouds.wgsl` (`pbd::clouds`), imported by the
      sky and the water pass.
- [x] `CloudNow` from `apply_weather`, extracted; the water view carries it.
- [x] The sea cap composites the slab over the sheet. Captured from orbit under
      a natural sky: clouds over ocean and land alike.

## 2. Volumetric rain
- [x] Rain map: `pbd_core::weather::precipitation_map`, tested against the
      field cell by cell; `weather::RainMap` refilled on a stray or each second.
- [x] The `rain` pass after the cap: marched to the scene depth, the sea and the
      cloud base; animated streak density; lit by `rain_light`; lit by a strike;
      not drawn from above the cloud base.
- [x] Curtains removed with their knobs. Captured: a storm from inside (a veil
      over the far hills) and from a dry spawn outside it.

## 3. Snow
- [x] Flakes in the near shower and the shafts where `Precip::Snow`: slow,
      swaying, opaque, `snow_density` times as many; a white slow volume.
- [x] No lens drops and no wetness under snow (`Weather::liquid`).
- [x] The rain gate measures height above the GROUND: from sea level it took
      every flake off a snowfield 220 m up.
- [x] `--spawn snow` for the capture; captured.

## 4. Lightning
- [x] `pbd_core::weather::Lightning`: deterministic, silent without heavy rain,
      striking only where it pours; `flicker` for the strokes. Tested.
- [x] Flash in the clouds (`cloud_flash_at`), the rain volume and the terrain;
      a jagged bolt in the shower mesh. Captured at night: the frame of a strike
      is 82 of 255 against 28 between strikes.
- [ ] A capture with the bolt itself in frame (the strikes so far landed out of
      view).

## 5. Weather slider
- [x] A slider and NATURAL / RAIN / STORM presets in the pause menu, driving
      `StormForcing`; captured.

## 6. Held
- [ ] The cloud slab still reads the cover over the PLAYER everywhere, so a
      forced storm covers the whole planet seen from orbit.
- [ ] The sea's mirrored sky is not greyed by the overcast.
- [ ] Sea-cave water still takes rain rings.
