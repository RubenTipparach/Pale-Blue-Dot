# Tasks

## 1. The field, in the core

- [x] `pbd_core::weather`: `solar`, `cloud_density`, `cloud_cell`,
      `precip_kind`, `Precip`, and a `WeatherField` of the reference's knobs
      (`solar_scale`, `solar_drift`, `solar_floor`, `arid_gamma`, `cloud_min`,
      `cloud_full`, `min_alpha`, `rain_cover_min`, `rain_min_s`,
      `weather_moisture_scale`, `moisture_boost`).
- [x] Tests that the field is a field: the same direction and time give the same
      answer; a desert direction clouds less than a jungle one; rain implies
      cover at or above the threshold; the trail keeps a column raining after
      the cloud thins and stops after `rain_min_s`; snow falls over tundra and
      mountains and nowhere else.

## 2. The app samples it

- [x] `Weather` gains `cover`; `rain` comes from the field under the player
      rather than from a key.
- [x] P cycles the moisture boost, so forcing a storm goes through the field.
- [x] The knobs into `weather.ron` beside the rain ones, validated.
- [x] A test that walking far enough leaves a storm, on the pure field.

## 3. The clouds get a thickness

- [x] The slab march in `sky_atmosphere.wgsl`: inner and outer radius, three
      octaves, Beer's law transmittance, a short sun march per step.
- [x] The app's `cover` drives the coverage threshold.
- [x] Measure the frame cost of the march against the flat shell, in a release
      scene with the sky filling the frame, and report both.

## 4. Prove it

- [x] Captures: a clear sky, an overcast one, and the same spot raining, from
      the ground and from orbit.
- [x] `docs/tenebris-comparison.md`: the cloud and weather rows measured against
      the reference's own frames.

## 5. Held, and named so the gap is visible

- [ ] Puff geometry: per-cell placement, lump clusters, the slab LOD and the far
      texture shell. Its own change, and a fifth indirect draw here.
- [ ] The moisture fBm in WGSL, so DISTANT clouds sit over wet ground rather
      than over the shader's own noise, pinned against the Rust on the GPU.
- [ ] Wind (`wind01`, `wind_dir`), which lands with grass sway or not at all.
- [ ] Snow as its own particle; the field already answers where it falls.
