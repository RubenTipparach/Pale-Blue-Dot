# The desert cascade's frame (`cascade-perf`), 2026-11-17

The owner's report: "im getting serious frame drops near these areas", at a
long shallow river cascade in a desert canyon (map 23.2255S 3.012W, 79.6 m;
also 20.885S 6.393W). The write-up and plan: `openspec/changes/cascade-perf/`.

`cascade-view.jpg`: the held view, `--at 23.2255 -3.012 --yaw 0 --pitch -10`
on a fresh world (the HUD reads `map 23.226S 3.012W | 79.6 m elevation`, the
owner's spot).

## Findings

1. **At the cascade the frame is the GPU's.** A 12 s CPU sample of the game
   standing there: the render thread blocked in `-[MTLCommandQueue
   commandBuffer]` (waiting for the GPU to return a command buffer) in 2,866
   of 7,282 samples; the main world waiting on the render world. The main
   world's update is 4.4-6 ms at p50 (`cpu_main_ms`), the same at the cascade
   and with the camera turned away from it.
2. **The water sim costs nothing while standing.** The river is settled: no
   water tick in 40 s, no remesh, no upload. The mesh: 1,720 cells, 575 tops,
   4,478 triangles, made once in 1.3-1.5 ms.
3. **The splash costs the CPU 0.07-0.1 ms a frame**: 131 feet, all within the
   140 m reach, about 1,230 quads of the 4,000 budget, 64-97 us
   (`SPLASH_STATS`). At the second spot: 67 feet, about 600 quads, 33 us.
4. **The water's fragment ran for every face of the cascade**, front and back,
   behind the canyon walls too: it can `discard`, so the GPU could neither
   reject a fragment by depth before running it nor remove hidden surfaces,
   and its test against the scene's depth came after all of its waves.
5. **Walking along the river, the spikes are fine-set uploads**, 8-11 ms on
   the render thread each (`UPLOAD_PARTS`: records 3-4.5 ms, `gpu_light`
   2.2-3.3 ms, materials 2.8-3.3 ms, for 3,078 columns and 387,828 light
   words). A landing every 3 s makes one, and so does every edit and every
   water tick. **7 of 12 water ticks after three digs by the bank changed no
   cell and still made one**, and a 1.5-2 ms remesh.

## What changed (the same picture)

| Cut | Check |
| --- | --- |
| The water's discards first (`water_hidden`) | captures at frame 300, two views, main against the branch: 0 of 5,184,000 pixels differ |
| A depth pre-pass for the flowing water; its shading tests against it and writes no depth | the same captures; `flowing_depth` validates; READY with 120 pipelines and no `LATE_PIPELINE` |
| A water tick that moves nothing keeps its version | the walk: the 7 empty ticks made no upload and no remesh (water meshes 20 to 13) |

Dropped: skipping the still waves where the flow-map's copies replace them
wholly. It changed up to 200,000 pixels by 1-141 levels (the shader
compiler's arithmetic), so it is not the same picture.

## `claude/fall-size-rules` at the same view

Its mist rule (the owner's request, a look change on its own branch,
`35a9622e`): 131 feet, quads 1,230 to 113 (-91 %), `animate_splash` 66-69 us
to 25-31 us a frame.

## Limits

- **No valid wall-clock A/B in this sitting.** The screen was locked for the
  whole sitting: macOS paces a window it cannot show (`acquire_ms` 7-37 ms
  once Bevy's 60 Hz unfocused cap was lifted for frame-log runs), and other
  sessions ran up to three more game windows and cargo builds on the same
  GPU and CPU. `perf_suite.py` refuses both. The interleaved stands below
  were taken anyway and say nothing: the same build reads 19 ms in one round
  and 40 ms in the next.
- **No GPU time per pass on a Mac.** Bevy's spans read nothing on Metal. A
  pass-boundary timer on the water's compose and the splash (the clouds'
  method) read the same 11-20 ms for both, at the cascade and facing away:
  an Apple GPU overlaps passes, so a boundary pair brackets its neighbours'
  work. It was taken out.
- Owed: `python tools/perf_suite.py --scenarios cascade,cascade-walk,walk
  --repeats 2 --variant "before|<main exe>||2de3c03a" --variant after` on an
  unlocked, quiet machine.

## The interleaved stands (paced by the locked screen; not a result)

M4 Max, release, windowed, `--no-vsync`, `--stand 40`, last 28 s of each,
rounds interleaved (A B C D E, then E D C B A, then A B C D E). ms.

| Run | wall p50 | wall p95 | wall p99 | main p50 | render p50 | acquire p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| before, round 1 | 19.7 | 93.7 | 131.0 | 8.8 | 16.3 | 6.8 |
| after, round 1 | 18.7 | 79.2 | 116.2 | 5.6 | 16.4 | 11.5 |
| before, round 2 | 39.8 | 81.4 | 108.5 | 10.1 | 37.5 | 28.1 |
| after, round 2 | 39.2 | 87.1 | 113.5 | 9.4 | 35.9 | 23.6 |
| before, round 3 | 37.3 | 52.6 | 71.8 | 5.2 | 36.7 | 33.1 |
| after, round 3 | 36.4 | 48.4 | 60.4 | 4.9 | 36.4 | 32.7 |

Facing away (control), wall p50 by round: before 20.3 / 38.1 / 42.1, after
18.9 / 25.1 / 22.5. Splash off (`PBD_PASS_OFF=splash`), before: 14.9 / 16.1
/ 18.2.

Machine: Apple M4 Max (macOS 15, Metal), the game's default window, release
build of `origin/main` `2de3c03a` with the instruments ("before") and the
branch ("after"), each run from its own folder with its own `assets/`.
