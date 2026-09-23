# Tasks

## 0. Measure the before
- [x] Captures on the shipped binary at the spawn: ground and sky, clear and
      `--rain 1`, and the coast at 420 m in rain
      (`docs/screenshots/weather-before-*.png`). Ground light ratio 0.725 (the
      wet darkening alone), blue ratio 0.37 (the sheen subtracting sky light),
      clear-sky cloud share 0.0%, rain-sky share 94.1%.

## 1. Overcast light
- [x] The seven overcast knobs in `WeatherSettings` and `weather.ron` at
      Tenebris's values, validated; the RON-matches-defaults test covers them.
- [x] `cover` in `params.weather.z`; the terrain's direct sun and sky fill
      dimmed by it; the haze thickened by cover and by rain intensity.
- [x] The sky's Rayleigh, Mie and sun radiance modulated by cover
      (`sky::overcast_scatter`, tested at zero and full cover).
- [x] The water sheet's sun specular dimmed by the same factor as the ground.
- [x] The terrain uniform's size held against naga's layout of `Params` in both
      terrain shaders, replacing the hand-typed 416.
- [x] **Found on the way: the field was never sampled on foot.** `sample_field`
      asked for the single `Camera3d`, and walking keeps an inactive flight
      camera beside its own, so the weather stayed at the launch forcing. It
      follows the active camera now, and logs the cover whenever it moves a
      twentieth.
- [x] Capture: ground in rain at the spawn, light ratio to clear **0.569**
      (was 0.725).

## 2. Wet ground
- [x] The sheen replaced by a Fresnel reflection of the sky along the rippled
      normal; `rain_sky_sheen` removed.
- [x] The puddle mask (flatness, two-octave value noise, wetness); waves, rings
      and the full mirror inside it only; grass, grass sides and leaves take no
      puddles.
- [x] The glint dimmed by cover.
- [x] `rain_puddle_scale_m`, `rain_puddle_share`, `rain_mirror_strength`.
- [x] Capture: ground in rain, blue ratio **0.46** against red 0.54 and green
      0.58 (was 0.36 against 0.72 and 0.75); the shore in rain at a grazing
      angle for the puddles.

## 3. Clouds
- [x] The cloud constants out of `sky.rs` into `weather.ron`, applied by one
      function at spawn and every frame; the storm threshold and a storm base
      darkness (`cloud_storm_dark`) are knobs of their own.
- [x] `--weather-at SECONDS` so a capture can be taken under a chosen cover:
      the spawn sits at 0.59 at launch, and at 0.20 (the body's average is
      0.18) 2,520 s in.
- [x] Retuned against the measured sky share: cover 0.20 **28.4%** (was 0.0%),
      cover 0.59 93.9%, a storm 100% (was 94.1%). Clear threshold 0.61,
      extinction 0.0115 per metre (optical depth 3.0 over the slab), storm base
      0.12.

## 4. Rain
- [x] Shafts of streaks over every raining cell inside `rain_detail_range_m`,
      thinning to `rain_lod_far_frac`; the near disk kept.
- [x] Far curtains over raining cells of a body-fixed lattice out to
      `rain_range_m`, cross-faded (`cell_lod`, tested for no pop); a test on
      the pure field that every raining cell in view gets one and no dry cell
      does, and one that the lattice does not move with the eye.
- [x] The lens drops and all rain gated at `rain_lod_alt_m`
      (`weather::rain_drawn_at`, tested; the coast at 420 m has none).
- [x] Rain colours converted from display values to linear.
- [x] Capture: a storm from outside it.

## 5. Dry caves
- [x] Capture the before: a cave, a mouth and an overhang in `--rain 1`. Inside
      the cave and under the overhang, rain ran down the lens: the whole
      before/after difference in those frames (0.38% and 0.40% of pixels) is
      the drops, and it is gone.
- [x] Measured the surface leak: of 744 cave floors under rock in the spawn's
      tier, **96 (12.9%)** were reached by sky light and so wetted, at a mean
      of 0.53 and up to 14/15 (`rain_under_rock_report`, ignored, prints).
      Each is dry now by the column rule.
- [x] `Column::open_to_sky` in the core, tested against a roof, a placed block,
      water and a torch; a test that the topmost DRAWN run's top is the same
      boundary, merged runs included, and is the word field the shader reads.
- [x] `rain_open` per face in the vertex shader (the terrain pass open; a
      column-pass floor open only as the topmost run; a ceiling never; a flank
      only against the neighbour's top gap); it replaces the sky light in
      `wet_amt`; no rivulets on a downward face.
- [x] `PlanetContact::open_to_sky`, tested on a real generated chamber;
      `Weather.sheltered` from the eye; no lens drops and no near shower when
      sheltered; the lens on its own lane (`WaterView.rain`), the sea's rings
      keep `fx.z`.
- [ ] A capture of a lit cave floor just inside a mouth, and of a block placed
      over wet ground. The harness has no view that stands inside a mouth
      looking at its floor in daylight; the rule is pinned by the tests above
      rather than by a picture.

## 6. Held
- [ ] Per-fragment cloud shadow on the ground.
- [ ] Wind-slanted streaks; snow particles; puddles on their own clock.
- [ ] The sea's mirrored sky colours are not yet greyed by the overcast; only
      its sun specular is dimmed.
- [ ] In-engine confirmation by the owner. The four requirements stay in this
      change until then rather than moving into `openspec/specs/`: two of them
      are pinned by tests, and the other two (the light and the mirror) only by
      captures.
