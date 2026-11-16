# Tasks

## 1. The field
- [x] `Weather` resource, `--rain`, a cycle key, wetness on `wet_fade_tau_s`;
      `weather.ron` with Tenebris's `rain_*` values; a test that wetness lags
      rain by the configured time constant.

## 2. The cap
- [x] `rain_ripple_grad` into `water.wgsl`, gated by `u_rain`, before the slope
      cap.

## 3. The terrain
- [x] The `hex.fs` wetness block in `planet_surface.wgsl`: sheet, rings,
      rivulets, darken, sheen, glint; thirteen knobs as uniform data; gated by
      sky light and the waterline.

## 4. Precipitation
- [x] The near shower as a per-frame CPU mesh, landing on `terrain_radius`,
      suppressed under water and under ground.

## 5. Proof
- [x] Captures at `--rain 1`: the shore at eye height (ripples, wet sand,
      streaks), the lens, and a wall face (rivulets).
- [x] The one-intensity requirement moved into `openspec/specs/world/weather/`
      with the wetness test. The landing and sky-gate requirements stay here:
      validated by capture, pinned by no unit test. Finding on the way: the
      shower drew nothing because the globe was queued LAST in the transparent
      phase (`f32::MAX` sorts nearest); at `f32::MIN` the streaks read as rain
      at Tenebris's own width.

## 6. Splashes where rain lands (owner, 2026-11-15)
- [x] 6.1 The write-up: design.md, "Splashes where rain lands": what is
      missing, the look (crown on puddles, softer spray on grass), GPU
      particles from a hashed seed and time with no per-frame upload, the
      gates (rain, wetness, sky light, tops only), and the cost budget.
- [x] 6.2 The mockup, `docs/mockups/rain-splashes.html` (M32): the game's hex
      ground and atlas, wet darkening, puddles and rings, Today / With
      splashes, rain strength, day and night with lamps, slow motion, density
      and size; splashes animated in the vertex shader. Stills in
      `docs/screenshots/mockup-rain-splashes/`.
- [x] 6.3 (2026-11-15, the owner: "approve rain splashes"; density and size start from the mockup's defaults, recommendation taken. The perf condition the owner set for the water smoothing applies here too: 6.5 gates the push.) The owner's approval of the mockup's look (and of the density and
      size the game should start from), on the tracker.
- [x] 6.4 (2026-11-15, `claude/rain-splashes` `49ceaa5`: `planet_splash.rs`
      and `rain_splash.wgsl`; 48 slots x 8 droplets per column, the mockup's
      sizes and speeds; the gates as written plus a town roof taking its
      column out of the list; `PBD_PASS_OFF=splash` prices it; the pipeline
      is queued from the first frame, READY with no `LATE_PIPELINE` in rain.
      Tests hold the copied terrain terms word for word, the uniform and
      record layouts, the classes and the near list.) The game port, after the approval and on a request: the near
      column list (rebuilt on a cell crossing or an edit), one instanced
      droplet draw in its own WGSL, gated by `rain`, `wetness`, sky light and
      the shower's own suppression; knobs (`splash_slots`, `splash_droplets`,
      `splash_period_s`, sizes and speeds) in `weather.ron`, validated.
- [ ] 6.5 (2026-11-15: NOT measured. The release exes are built and kept,
      `output/perf/rain-splash-exe/` (before `ea9df4d`, after `49ceaa5`), but
      the shared Mac never gave a quiet sitting: a showcase run, then Chrome
      relaunched mid-sitting and the suite refused; the water sitting before
      it lost window focus on every walk. Run
      `--scenarios storm,rain-stroll --repeats 2 --variant "before|<before exe>||ea9df4d" --variant "after|<after exe>||49ceaa5"`
      on a quiet machine. The push waits on it.) The perf A/B: `tools/perf_suite.py`, `--release`, old build against
      new interleaved in one sitting, on `storm` and `rain-stroll`; within the
      0.25 ms p50 budget or S and K reduced; report in `docs/benchmarks/`.
- [x] 6.6 (2026-11-15: `docs/screenshots/weather-rain/splash-*`, before
      with `PBD_PASS_OFF=splash` on the same binary. The path's crowns read;
      the grass spray is barely visible in a still, as faint as the mockup's
      but lost in the game's grass; see that README.) Proof: `--capture` stills of a puddled path and a meadow in rain,
      by day and at night under a lamp, before and after, in
      `docs/screenshots/`.

## 7. Rain artifacts (design "Rain artifacts"; owner, 2026-11-16)

- [x] 7.1 R1 (Built: `rain_sky_light`; a streak (110, 128, 180) to (67, 81, 104) over a wall of (45, 53, 56); `the_rain_takes_the_sky_a_standing_face_takes`.): the shower and the shafts lit by the sky's irradiance on a standing face under the new model. Verify: before and after at one spot; a test that a full deck at noon lights a streak no brighter than the old curve.
- [x] 7.2 R2 (Built with `flora-v2` 11.1: `crop3x-tufts.jpg`.): tufts and all clutter take no wet block (with `flora-v2` 11.1). Verify: close-up captures in rain, before and after.
- [x] 7.3 R3 (Built: `LensWet`, `the_lens_wets_and_dries_over_time_and_grows_no_drop_while_dry`; `seq-lens-into-cover.jpg`, `seq-lens-out-of-cover.jpg`, `seq-rain-stops.jpg`, `seq-rain-starts.jpg` and their traces.): the lens's wetness eases both ways (`lens_wet_tau_s`, `lens_dry_tau_s`), its drop clock stops while no rain reaches it. Verify: a test of the easing; capture sequences of rain stopping, rain starting, into cover and out of it.
- [x] 7.4 R4 (Built: `ShowerOpen`; the ground's wetness, the shower's count and the splashes all follow eased values (traces). The splashes end at 20 m in their shader.): the near shower thins over a moment under cover; the ground's wetness checked in a sequence. Verify: the sequences above.

## 8. Height fog with RAIN SHADOW off (owner, 2026-11-16)
- [x] 8.1 The write-up: design.md, "Height fog with RAIN SHADOW off", the
      owner's words, the form, the reference level, the colour, the pass it
      folds into and the match to the volume.
- [x] 8.2 `rain_height_fog_falloff_m` in `weather.ron`, validated; the
      `height_fog` lane; the fog at the end of the `clouds` composite;
      `PBD_HEIGHT_FOG=0` to price it.
- [x] 8.3 Captures: a storm, RAIN SHADOW off before and after and on; a
      valley in a storm (`docs/screenshots/height-fog/`). The falloff went
      from 120 m to 250 m on the first look: at 120 m the horizon was a
      third as murky as the volume's.
- [ ] 8.4 Its cost on `storm`, RAIN SHADOW off, fog off against on. Not
      measured yet: the sitting is `reflection-quality` 3.2's, which waits
      for a quiet machine.
