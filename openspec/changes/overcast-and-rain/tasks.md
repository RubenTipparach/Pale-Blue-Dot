# Tasks

## 0. Measure the before
- [x] Captures on the shipped binary at the spawn: ground and sky, clear and
      `--rain 1`, and the coast at 420 m in rain
      (`docs/screenshots/weather-before-*.png`). Ground light ratio 0.725 (the
      wet darkening alone), blue ratio 0.37 (the sheen subtracting sky light),
      clear-sky cloud share 0.0%, rain-sky share 94.1%.

## 1. Overcast light
- [ ] The seven overcast knobs in `WeatherSettings` and `weather.ron` at
      Tenebris's values, validated; the RON-matches-defaults test extended.
- [ ] `cover` in `params.weather.z`; the terrain's direct sun and fill dimmed
      by it; the haze thickened by cover and by rain.
- [ ] The sky's Rayleigh, Mie and sun intensity modulated by cover.
- [ ] The water sheet's sun specular dimmed by the same factor as the ground.
- [ ] Capture: ground in rain, light ratio against clear well under 0.725.

## 2. Wet ground
- [ ] The sheen replaced by a Fresnel reflection of the sky along the rippled
      normal; `rain_sky_sheen` removed.
- [ ] The puddle mask (flatness, noise, wetness); rings and full mirror inside
      it only; grass without puddles or rings.
- [ ] The glint dimmed by cover.
- [ ] `rain_puddle_scale_m`, `rain_puddle_share`, `rain_mirror_strength`.
- [ ] Capture: ground in rain with the blue ratio back near red and green, and
      a grazing view of a flat stone floor.

## 3. Clouds
- [ ] The cloud constants out of `sky.rs` into `weather.ron`.
- [ ] Clear threshold, storm base darkness and extinction retuned against the
      measured sky share at `--rain 0`, `0.5`, `1`: 15-30%, between, 90%+.

## 4. Rain
- [ ] Near streaks to `rain_detail_range_m` with distance thinning.
- [ ] Far curtains over raining field cells, cross-faded; a test on the pure
      field that every raining cell in view gets one and no dry cell does.
- [ ] The lens drops and streaks gated at `rain_lod_alt_m`; a test.
- [ ] Capture: a storm from 400 m outside it.

## 5. Held
- [ ] Per-fragment cloud shadow on the ground.
- [ ] Wind-slanted streaks; snow particles; puddles on their own clock.
