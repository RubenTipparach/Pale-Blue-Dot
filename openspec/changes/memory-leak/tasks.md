# Tasks

## 1. Measure

- [x] 1.1 `PBD_MEMORY_TRACE` (`memory_trace.rs`): entities and asset counts every 2 s. Verify: a walk's CSV, flat after loading.
- [x] 1.2 The `alloc-trace` feature (`alloc_trace.rs`) and `tools/alloc_trace.py`. Verify: a walk's table names `fish::draw::build` and Bevy's mesh extraction as the churn.
- [x] 1.3 RSS against `phys_footprint` standing and walking (proposal, "What was measured").

## 2. Cut (the same picture)

- [x] 2.1 `fish::draw::build` lays into the buffers the mesh holds; flat normals written in place. Verify: `a_school_is_laid_into_the_buffers_it_holds`, `a_school_relaid_is_the_school_laid_fresh`.
- [x] 2.2 The kelp's sway writes into the positions it holds. Verify: `cargo test -p pbd-app --lib fish` (the kelp tests unchanged).

## 3. Prove

- [x] 3.1 The 15-minute walk, `main` against the branch, release, RSS, footprint and frame time by minute, screen locked. Verify: `docs/benchmarks/2026-11-19-memory-leak/report.md` and `memory.png`.
- [ ] 3.1b The same with the screen unlocked, where the walking RSS climb shows (report, "Owed").
- [x] 3.2 The `alloc-trace` table of the branch's walk. Verify: no `fish::draw` stack above 256 KiB after the first minute.

## 4. Left for the owning changes

- [ ] 4.1 The river cache save builds its whole file in memory every 30 s while the cache grows (`river-cache`).
- [ ] 4.2 Ambience beds are decoded whole each time one comes up (`audio-and-ambience`).

## 5. The reef (owner, 2026-11-25)

- [x] 5.1 The sway in the vertex stage (`sway_vertex.wgsl`, `ATTRIBUTE_SWAY`, `FieldView::sway`); `bend_all` gone. Verify: `a_swaying_vertex_carries_its_reach_and_phase`, `the_shader_keeps_the_sway_distances`.
- [x] 5.2 Before and after at the reef (13.096, 94.896), 11:00, standing 150 s, fast build, desktop RTX 3070, two pairs interleaved: frame p50 98 to 52 ms, p99 164-170 to 71-75 ms, main thread p50 59 to 47 ms, GPU 11 to 10 ms, memory 1.87 to 1.37 GB; the sway and half density together. What is left is the ambient life: `ambient_ms` 34-38 of the 47 ms.
- [ ] 5.3 The ambient life and vent smoke off the per-frame CPU write.
- [ ] 5.4 Frustum culling for the reef's meshes, then instancing by piece.
