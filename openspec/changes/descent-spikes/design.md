# Design: descent-spikes

## Measured (2026-11-12, RTX 3070, release, `storm` route, one run each)

Instruments, all measurement only and taken out again: `STAND_TIMING` (a
town's standing frame, part by part), `MAIN_SPLIT` (each main schedule's time
on a frame over 20 ms, and each fixed schedule's and physics set's summed
over the frame's fixed steps), `RENDER_SPLIT` (each render stage's on a
render frame over 20 ms) and the thread each `SPENT` line ran on. Spikes from
the frame log, lined up with the log (`frame_graph.py`'s 0.1-0.25 s):

| spike | what lands that frame | main thread | of it, the fixed step | of that, from Avian's solver mark to the next |
| --- | --- | ---: | ---: | ---: |
| 3544 m, 42-50 ms | Dunmouth stands (5.7-6.7 ms of `Update`) | 36-43 ms | 23-32 ms | 20-31 ms |
| 3770-3790 m, 47-52 ms | fine set lands; its upload 11-13 ms on a pool thread | 33-38 ms | 29-35 ms | 34 ms |
| 3808 m, 62-70 ms | Ashenstead stands (13.4-14.6 ms) | 59-65 ms | 37-41 ms | 37 ms |
| 4110-4150 m, 30-58 ms | a fine set lands; Mirhan stands (2.3-3.6 ms) | 21-43 ms | 14-40 ms | 16-32 ms |
| 4223 m, 35-38 ms | a fine set lands at touchdown | 22-23 ms | 18-19 ms | 19 ms |

- **It is the main thread, and it is the physics step.** No render frame
  passed 20 ms at any of them. One fixed step runs (two at most), and all of
  its time is in `FixedPostUpdate`, inside Avian's step, from the Solver
  set's mark on.
- **What it is not (each tested, 2026-11-12):**
  - **Not the task pool.** The solver's loops run on Bevy's compute pool,
    whose four threads hold long render jobs on these frames (the fine-set
    upload, 11.4-12.8 ms on `Compute Task Pool (0..3)`). Avian's `parallel`
    feature turned off made the step run on the main thread alone, and the
    spikes stayed (3544 m 42 ms, 3808 m 70.5 ms, touchdown 41-96 ms, worse),
    the solver window as long as before (15-44 ms). Reverted.
  - **Not freeing the old fine set.** A probe on `FineSet`'s drop: 1.7-4.2
    ms, 124,000-261,000 records, on whichever thread lets it go last, and
    the town stands (3544 m, 3808 m) free none.
- **What it is: the main thread waits for Avian's tree optimization.**
  Timing each of Avian's solver stages puts the whole of it (13-47 ms on
  every spike, every other stage under 0.5 ms) between the Restitution
  stage and Finalize. In Finalize, `block_on_optimize_trees`
  (`avian3d-0.6.1/src/collider_tree/optimization.rs:289`) waits on the
  task `optimize_trees` spawned on Bevy's `AsyncComputeTaskPool` at the end
  of the broad phase. The game runs its long jobs on that same pool: each
  fine set's build (about 3 s, one thread throughout) and each town's cut
  (tens of ms, several at once as towns come into range). On the frames a
  fine set lands (the next is requested at once) or towns stand (more are
  being cut), the pool's threads are all busy, Avian's small task waits
  behind a cut, and the main thread waits with it.

## Decision: the collider tree is optimized on the main thread

`ColliderTreeOptimization::use_async_tasks` is set to `false` in
`PaleBlueDotPlugin`: Avian optimizes its trees in place during the step.
The game has five colliders, so the optimization is a few microseconds, and
the step no longer waits on a pool it shares with jobs of seconds.

- **Look.** The trees answer the same queries; the bodies are solved the
  same way. Captures of the landing and of the walk, before and after.
- **Measure.** `storm`, before against after, interleaved, latency 2, the
  four route metres read off each run.
- **Not chosen now:** moving the game's long jobs off the async pool onto
  threads of their own. That would also unblock any other short task that
  shares the pool, and is the next step if one turns up.

## The town's casters on its cut task

Once the step no longer waited, Ashenstead standing still made a 31 ms frame
at 3808 m: 20 ms of `Update`, of it about 9 ms chunking its casters
(`STAND_TIMING`). `towns::casting` is pure work over the cut town, so the
cut's task does it (`Laid::casting`) and the standing frame takes the result;
a town cut any other way is chunked on the frame as before.

## Measured after (`docs/benchmarks/2026-11-12-descent-spikes`)

`storm`, `f160a7e` against this change, 3 runs each, interleaved, latency 2:
frames over 30 ms a run 5, 5, 4 before and 0, 0, 0 after; the worst frame
59-91 ms before and 19-22 ms after; p95, p99 and p99.9 within the spread;
p50 0.1 ms higher after, at the edge of the spread.

**Look.** `tools/capture_repeat.py --against`, GPU captures, before against
after: the Holbrook lane at 10:00, 0 pixels over 1/255; the storm descent at
frame 2600, 13,099 pixels (1.0%), worst 139/255, in the HUD's distance and a
fine set's fade mid-landing. The before exe alone differs from itself there by
442,538 pixels (34%): the descent capture does not repeat (its LOD landings
fall on different frames), so its difference is not this change's.
