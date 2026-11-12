## 1. Data

- [x] 1.1 `rain_detail_reach_m` and `rain_detail_fade_m` in `WeatherSettings`, `weather.ron`, validated, with a test
- [x] 1.2 Carry them to `PlanetParams` (Rust and WGSL), size test updated

## 2. Shader

- [x] 2.1 `near_detail` fade in the wet branch of `planet_surface.wgsl`; the sheet, rings and rivulets gated and scaled by it (new lighting model only)
- [x] 2.2 A shader test that the gate is there and today's path is unchanged

## 3. Measure and show

- [x] 3.1 `rain-stroll` scenario in `tools/perf_suite.py` (Holbrook's lane in a forced rain; a stroll, not the 1 km walk, so the walk's dig fix cannot move it)
- [x] 3.2 Before/after sitting on `rain-stroll`, `clouds` and `storm` (`docs/benchmarks/2026-10-29-cull-wet`), report in `docs/benchmarks/`
- [x] 3.3 Before/after stills at the reach and past it in `docs/screenshots/wet-near-detail/`
