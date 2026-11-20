# Tasks

## 1. The write-up
- [x] 1.1 The sizes the engine carries, and the L7 and L12 gaps (design 1).
- [x] 1.2 What scales with size, and how to measure it (design 2).
- [x] 1.3 The viewer's design (design 3); `pbd-core` checks for wasm32.

## 2. The viewer (on a request)
- [ ] 2.1 `tools/planet-viewer/` wrapper crate over `pbd-core`, built to wasm with one script.
- [ ] 2.2 The page: Globe and Ground tabs, rung, seed and version controls, the size card; lit at night, nothing spinning on its own.
- [ ] 2.3 A test: wasm and native altitudes agree at fixed directions.
- [ ] 2.4 Publish it as an artifact with the `.wasm` beside it, and list it on the tracker (Guides).

## 3. Measuring (on a request)
- [ ] 3.1 A CPU bench of the planet-wide data at L8 to L11 (time, memory), run in a cloud session; report in `docs/benchmarks/`.
- [ ] 3.2 Frame time on the owner's desktop once a body exists at another rung (`perf_suite.py`).

## 3b. The far planet as a cube-sphere (owner, 2026-11-20; on a request)
- [ ] 3b.1 A mockup first: the home world from orbit and from a high flight, cube-sphere maps baked from `pbd-core` (through the viewer's wasm build) beside today's hex base, by day and at night, nearest-sampled per cell; the owner judges it with screenshots.
- [ ] 3b.2 The bake: albedo, normal and height per cube face from the generator, cached by seed and generator version; a test that a texel's colour is its cell centre's biome colour.
- [ ] 3b.3 The renderer: the cube-sphere beyond the hex tiers, the dither hand-over, lit and fogged as the terrain; the hex base trimmed to a fixed reach.
- [ ] 3b.4 Measured on the desktop: orbit and flight frame time before and after (`perf_suite.py`), and at a 9,600 m and a 19,200 m test body.

## 4. Widening the key (a separate change, on a request)
- [ ] 4.1 A key format covering L7 to L12, read beside today's, with saves migrating forward on open.
