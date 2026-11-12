# Proposal: the lighting model's frame cost, found and cut

## Why

The owner, 2026-11-11, on merging the lighting model (PR #27, `f15a647`):
"Make sure to benchmark the new lighting model on my desktop", then "And
figure out optimizations".

The lighting model was priced part by part on an M4 Max on 2026-10-29
(`docs/benchmarks/2026-10-29-lighting-full`). It had never been measured on
the owner's desktop (RTX 3070, Vulkan), and the Mac cannot time passes on
the GPU. The sitting of 2026-11-11 on the desktop
(`docs/benchmarks/2026-11-11-lighting-model`,
https://claude.ai/artifact/JvXq67Da4F59y2ZocmwKG9) measured it there:

- **On foot in the rain it costs 2.8 ms at p50 and 3.7 ms at p95**
  (`rain-stroll`, 11.2 to 14.0 ms p50, 14.3 to 18.0 ms p95; the spread
  between repeats is 0.2 ms). It takes the walk under rain from inside the
  60 fps floor at p99 (15.3 ms) to outside it (27.0 ms). This is the case
  that costs the most.
- **Almost all of that is L8, reflections.** With `PBD_REFLECT=0` the stroll
  reads 11.1-11.2 ms, the same as the old model (10.9-11.3). Every other
  part (soft shadows, sun bounce and lamp bake, bloom, the tone pass) is
  within 0.5 ms of the whole model, most of them inside the spread.
- **The cost depends on where the eye looks.** The stroll walks a lane up
  and back. One direction reads 15-18 ms, the other 11-13 ms; the old model
  reads 11 ms both ways. That is the shape of a ray march whose length
  depends on what the reflected ray meets.
- **In flight and walking in the dry it costs about 0.5-1.2 ms at p50**:
  clouds 8.9 to 10.0 ms, far-side 5.2 to 6.2 ms, walk 8.3 to 9.2 ms
  (the new model against the old in the same exe).

Reading the code (`f15a647`) finds why L8 is expensive and where it does
work that buys no picture: the wet-ground voxel walk keeps marching after
its ray has left everything it could hit, then falls back to a 32-step
screen trace; the sea's screen trace runs on every far sea pixel whatever
its distance; the reflection capture runs every frame whether or not
anything wet or any sea is on screen.

## What Changes

This change is a measurement and a plan. **No source changes are made by
proposing it.** Each fix below is written in the design, ranked by measured
cost times expected saving, with its evidence, its risk to the look and how
it is measured after. The lighting must look the same: every fix that could
move a pixel is checked with `--capture` shots before and after at the same
place and time.

1. **Instruments first** (still no behaviour change):
   - a `PBD_REFLECT` split (`terrain`, `sea`, `capture`) so the three parts
     of L8 can be priced apart;
   - per-pass GPU columns in `--frame-log` (the spans Bevy already records:
     bloom, the water composite, shadows, the reflection capture, the
     terrain pass) so a pass's cost is read, not inferred from a toggle.
2. **L8 terrain trace: stop when nothing can be hit** (rank 1).
3. **L8 sea trace: limit by distance and step size on screen** (rank 2).
4. **L8 capture: skip when nothing reflects** (rank 3).
5. **The CPU work beside it**: the cloud-puff site rebuild every frame,
   environment variables read per call, `near_lamps` sorted every frame
   (rank 4-6, small).
6. **Bloom and tone in one fewer full-screen pass** (rank 7).

## Not in this change

- No tuning constant, shader term or default moves before its fix is
  approved on its numbers and its shots.
- Nothing about how the lighting looks. A fix that changes a pixel is shown
  beside the unchanged frame before it is kept.
- Saved worlds are untouched: all of this is derived render state.
- The flight tails (single frames of 50-300 ms during fine-set rebuilds)
  are not the lighting model's: they are there with the old model and in
  the build before it. They belong to `detail-fade` / pop-in work.

## Impact

- `assets/shaders/planet_surface.wgsl` (`reflect_trace`, `ssr_trace`, the
  wet-ground gate), `assets/shaders/water.wgsl` (`sea_ssr`),
  `crates/pbd-app/src/planet_ssr.rs`, `crates/pbd-app/src/config.rs`,
  `crates/pbd-app/src/desktop.rs` (frame log), `tools/perf_suite.py`.
- Spec: `platform/performance` (new requirements, below).
