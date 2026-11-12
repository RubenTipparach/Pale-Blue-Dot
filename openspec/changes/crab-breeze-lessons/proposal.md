# Proposal: what Crab Breeze's frame rate can teach Pale Blue Dot

## Why

The owner, 2026-11-12: Crab Breeze (the same owner's Bevy game,
`RubenTipparach/Crab-breeze`, Bevy 0.20.0, Avian 0.8) "has higher frame
rates" than Pale Blue Dot and "might have some good insights". The owner
also suggested running the expensive passes, such as bloom and fog, at half
resolution.

Both games have been measured on the same desktop (RTX 3070, i7-9700F,
Windows 10, Vulkan). This change reads both repositories and both sets of
benchmark reports to answer two questions. Why is Crab Breeze cheaper? And
which of its techniques would make Pale Blue Dot cheaper? Nothing was run
for it: a benchmark was in progress on the machine. Every number comes from
a checked-in report in one of the two repositories, and every code claim
comes from reading the code at Crab Breeze `main` `8f9ed158` and Pale Blue
Dot `86361b1`.

## What the reading found

1. **Crab Breeze does less GPU work per pixel. It is not doing the same work
   more cheaply.** Its GPU frame is 1-2.5 ms at 1920 x 1080 in most of
   its scenes (and 8 ms in the capital when the scene fits in video memory).
   Pale Blue Dot's is 4.6-9.6 ms at 1440 x 900, which has 37 % fewer pixels.
   Crab Breeze's clouds are camera-facing quads in one draw, its sky and
   haze come from Bevy's atmosphere lookup tables, its fog is an analytic
   term in one compose pass, it traces no reflections, and it renders with
   MSAA off and FXAA. Pale Blue Dot ray-marches volumetric clouds, walks
   voxels and screen-traces for reflections, shades its terrain in a
   2,500-line fragment shader and renders with 4x MSAA. That difference is
   the look, and the look is the point of both games. It is not a fault to
   fix.
2. **On the CPU side the two games are close, and Crab Breeze has already
   cut its own costs.** Crab Breeze's frames are CPU-paced at about 9 ms in
   open country and 13-14 ms in the suburbs. Pale Blue Dot's walk is
   9-10 ms, and its town walk 9 ms on `town-frame-time` (17-18 ms on
   `main`). Crab Breeze reached its numbers by cutting per-entity render-world
   work: one material across many meshes, per-instance data in storage
   buffers, distance-limited shadow casters, mesh building off the main
   thread, and a ground-height cache. Pale Blue Dot's towns have the same
   problem those changes solved.
3. **One Pale Blue Dot setting probably costs throughput outright and has
   never been measured.** Interactive and benchmark runs ask the swapchain
   for a maximum frame latency of 1 (`desktop.rs:852`, set in the first
   prototype `f958f08` and never timed). Crab Breeze leaves it at wgpu's
   default of 2. With a queue depth of 1, the render thread waits for the
   GPU before it can start the next frame. `prepare_windows` (the swapchain
   wait) is 2.4 ms of the town walk's render thread
   (`town-frame-time/design.md`), and wall time sits 4-5 ms above GPU time
   in every lighting-model variant. This is the cheapest experiment on the
   list.
4. **On half resolution: Crab Breeze runs none of its own passes at reduced
   resolution. It avoids the expensive per-pixel work instead.** Pale Blue
   Dot already reduces its expensive passes: clouds at 0.4 scale with a
   depth-aware upsample, the reflection capture at 960 x 600, and bloom from
   a quarter-size image. Bloom and tone together cost 0.4 ms. The one
   expensive pass left at full resolution is the L8 reflection trace (2.2 ms
   p50 on foot in the rain). Half resolution belongs there, using the
   clouds' existing upsample, once `lighting-perf`'s early exits land.
5. **Crab Breeze's loading-screen practice is directly reusable by
   Pale Blue Dot's loading screen.** It holds the screen until frames stay
   steady, warms pipelines, and counts every pipeline that compiles after
   the game is ready. The counter is the part that proved its worth.
   Crab Breeze measured the warm-up itself and found it moved compiles
   without changing the median. The counter found the 83-107 late pipelines.

## What changes

This change is a write-up and a ranked plan only (CLAUDE.md, "Write it up
before touching code"). Each technique is a separate measured step in
`tasks.md`. Each runs on its own A/B in `tools/perf_suite.py` against the
exe before it, and anything that changes the picture goes to the owner with
screenshots first.

The ranked list is in `design.md`. Expected savings are estimates unless
they are marked measured:

1. Measure frame latency 1 against 2 (estimate: 1-3 ms p50 in GPU-heavy
   scenes, at the price of up to one frame of input latency).
2. Give town meshes one shared material, with per-mesh field data in a
   storage buffer indexed by `MeshTag`, as Crab Breeze does for buildings,
   grass and people (estimate: 1-2 ms of render thread in towns).
3. Cache the generator's ground in tiles for vehicles and every
   repeated-height caller, as Crab Breeze's `terrain-height-cache` does
   (Crab Breeze measured its mech step falling from 6-7 ms to 0.2 ms).
4. Rebuild the cloud-puff sites only when the eye moves past a distance, as
   Crab Breeze does (`lighting-perf` rank 4; 0.35-0.4 ms measured).
5. Spread per-frame captures over frames, or skip them when nothing reads
   them: the SSR capture, and the shadow cascades' town casters.
6. Put heavy rebuilds on a worker and keep the old result until the new one
   is ready (the planet rebuild in `towns.rs` `settle`: a measured 5.3 s
   frame).
7. Price 4x MSAA. Measured against 2x and against off with FXAA, it is a
   look change, so it goes to the owner with screenshots.
8. Count late pipelines, and gate the loading screen on steady frames.
   This is for the in-flight loading-screen work.
9. Build release with `codegen-units = 1` (Crab Breeze does).
10. Track each frame against a frame budget and list the offenders
    (Crab Breeze's `perf_gate`).

## Not in this change

- **The Bevy 0.18.1 to 0.20 upgrade** is `openspec/changes/bevy-0-20`, its
  own change and its own A/B.
- **Fixes `lighting-perf` already plans** (the L8 early exits, the sea
  trace, the capture skip, the bloom fold) are referenced, not restated.
- **Any change to the look** of clouds, water, reflections or terrain. Crab
  Breeze is cheaper there because it draws a simpler picture. Porting its
  billboard clouds or analytic fog would be a design decision for the owner,
  not an optimization, so it is not proposed.

## Impact

- `crates/pbd-app/src/desktop.rs` (a `--frame-latency` launch flag),
  `field_light.rs` and `towns.rs` (the shared material),
  `vehicles.rs` and `pbd-core` (the ground cache), `planet_cloud_puffs.rs`,
  `planet_ssr.rs`, `planet_shadow.rs`, `towns.rs` `settle`,
  `Cargo.toml` (`[profile.release]`), `tools/perf_suite.py` (the late-pipeline
  and over-budget columns).
- No save, generator or world-data change. The ground cache returns
  bit-identical heights, and that is a test.
