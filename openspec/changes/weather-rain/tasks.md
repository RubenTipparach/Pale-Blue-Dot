# Tasks

## 1. The field
- [ ] `Weather` resource, `--rain`, a cycle key, wetness on `wet_fade_tau_s`;
      `weather.ron` with Tenebris's `rain_*` values; a test that wetness lags
      rain by the configured time constant.

## 2. The cap
- [ ] `rain_ripple_grad` into `water.wgsl`, gated by `u_rain`, before the slope
      cap.

## 3. The terrain
- [ ] The `hex.fs` wetness block in `planet_surface.wgsl`: sheet, rings,
      rivulets, darken, sheen, glint; thirteen knobs as uniform data; gated by
      sky light and the waterline.

## 4. Precipitation
- [ ] The near shower as a per-frame CPU mesh, landing on `terrain_radius`,
      suppressed under water and under ground.

## 5. Proof
- [ ] Captures at `--rain 1`: the shore at eye height (ripples, wet sand,
      streaks), the lens, and a wall face (rivulets).
- [ ] Move the requirements below into `openspec/specs/world/weather/` in the
      commit that makes them true.
