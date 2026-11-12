# Tasks

Each fix is its own A/B with `tools/perf_suite.py`, against the exe before
it, interleaved, with at least two repeats. Nothing heavy runs beside it.
The report goes in `docs/benchmarks/<date>-<name>/` and the HTML page is
republished (CLAUDE.md). A fix that changes a picture goes to the owner with
`--capture` shots first.

## 0. Read (done, 2026-11-12)

- [x] 0.1 Read Crab Breeze `main` `8f9ed158` (render setup, perf changes,
      benchmark reports) against Pale Blue Dot `86361b1`. Write the
      comparison and the ranked list (`design.md`).

## 1. Frame latency (rank 1)

- [x] 1.1 Instrument: a `--frame-latency 1|2` launch flag, written into the
      frame log's header. The default stays 1.
- [ ] 1.2 A/B on one exe: `rain-stroll`, `storm`, `walk` and `town`, three
      repeats, uncapped. Then the same with vsync on, counting refreshes
      missed. Write the result into the design.
- [x] 1.3 If 2 beats the spread, make 2 the default and keep the flag.
      Record it as "recommendation taken (ask only with screenshots)" and
      update `docs/flight-controls.md` and the
      `voxel-engine-foundation` note.

## 2. Cheap, no picture change (ranks 9, 10)

- [ ] 2.1 `perf_suite.py`: an over-16.7 ms count per run and the ten worst
      frames, each with its time and the nearest `SPENT` line.
- [ ] 2.2 `codegen-units = 1` in `[profile.release]`: the same commit built
      both ways, interleaved on `town`, `walk` and `storm`. Keep it only if
      the saving is past the spread.

## 3. Town batching (rank 2)

- [ ] 3.1 Trace the town walk (`--features trace`, `tools/trace_rank.py`)
      and record the render-world spans for `FieldLit`: prepare, queue and
      batch.
- [ ] 3.2 Split `FieldUniform`. The view-wide part goes in one extracted
      uniform. The per-mesh part goes in a storage buffer indexed by
      `MeshTag`.
- [ ] 3.3 Town meshes that share a texture share one `FieldLitMaterial`.
      A field or light change writes a row, not a material.
- [ ] 3.4 Shots of Ashenstead and Holbrook (10:00 and 22:00) before and
      after: identical, or the difference shown. Then the A/B on `town`
      and `walk`.

## 4. Ground cache (rank 3)

- [ ] 4.1 A `pbd-core` tile cache of the generator's ground, keyed by
      generator version. Test: bit-identical to the uncached call at
      seeded points, on tile edges and at the pentagons.
- [ ] 4.2 Route `vehicles::ground_under`, walking's off-tier ground and the
      town bakes through it.
- [ ] 4.3 A `harbour` scenario (craft under way) and the A/B, with
      `VEHICLE_STEP` from the log.

## 5. Captures and casters (rank 5)

- [ ] 5.1 Price shadows in a town: `town` with `--no-shadows`.
- [ ] 5.2 Cull town casters per cascade by bounds. Add a cast distance in
      data for small town props. Shots at Ashenstead at 10:00 and 17:00.
      A/B on `town` and `walk`.
- [ ] 5.3 A census of closed solids among the `cull_mode: None` materials
      and casters. Draw any that never show their back one-sided.

## 6. Rebuilds off the main thread (rank 6)

- [ ] 6.1 With `town-frame-time` 3.6: the towns' ground goes in the first
      planet build, or the rebuild goes on the async pool and the old
      planet stays until it lands. Measure the first frame of a fresh
      world and of a reopened one.

## 7. Loading screen hand-off (rank 8)

- [ ] 7.1 A late-pipeline counter. The render world walks
      `PipelineCache`, logs `pipeline late:` and writes a count column in
      the frame log and the suite's report.
- [ ] 7.2 Give the loading screen's ready test "no pipeline `Queued` or
      `Creating`, then 1 s under 50 ms" (Crab Breeze's `STEADY_S` and
      `HITCH_S`).
- [ ] 7.3 Show the town, vehicle and fish materials behind the screen.
      Claim nothing until 7.1's counter shows fewer late pipelines over
      three runs.

## 8. MSAA (rank 7, a look decision)

- [ ] 8.1 Instrument: `PBD_MSAA=1|2|4` and `PBD_FXAA=1`. No default change.
- [ ] 8.2 A/B on `walk`, `rain-stroll` and `far-side`, plus shots (a shore,
      a town roofline, a tree line) for each mode.
- [ ] 8.3 Put the shots and numbers in the owner survey and fold the answer
      back here.

## 9. Half-resolution reflections (section 2, after `lighting-perf` 2.1-2.3)

- [ ] 9.1 Re-price L8 after the early exits. Go on only if it still costs
      more than 0.8 ms p50 on `rain-stroll`.
- [ ] 9.2 Move the reflection trace into a half-resolution pass, upsampled
      through the clouds' depth-aware upsample. Shots of puddles under
      walls and at the sea's edge. A/B on `rain-stroll` and `storm`.
