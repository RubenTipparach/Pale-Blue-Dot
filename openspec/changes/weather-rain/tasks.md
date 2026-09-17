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
