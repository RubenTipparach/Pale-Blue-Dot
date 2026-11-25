# Tasks

## 1. Before anything moves

- [x] 1.1 Measure version 10's swamps (`swamp_report`): share, where, why.
- [x] 1.2 Pin version 10's ground on its own digests, in a commit before any
      swamp code (`version_10_keeps_its_ground`).

## 2. Version 12

- [x] 2.1 `Swamps`, `swamp_holds`, `swamp_patch`, `TENEBRIS_V11`,
      `GENERATOR_VERSION = 11`; `rivers::beside`.
- [x] 2.2 Tests: the share on three seeds, version 10's swamps inside
      version 12's, no height moved, the ground rule (pools held, mud and
      moss), sites and climate read the ground without swamps.
- [x] 2.3 `settled-g11-l{3,5}` (version 10's bytes, decision 7).
- [x] 2.4 `mountain-ground` amended to version 12.

## 3. The draw

- [x] 3.1 Reeds for the swamp's fern piece (decision 5).
- [x] 3.2 The map's biome layer shows the swamps (no code change: it reads
      `biome_at`).

## 4. Proof

- [x] 4.1 Map biome layer, version 10 and version 12, and an eye-level
      swamp, in `docs/screenshots/swamps/` with a README.
- [ ] 4.2 The owner's look at the captures.
- [ ] 4.3 Generation time and frame cost on the owner's machine (not
      measured in the cloud).

## Swamp water cached (owner's playtest, 2026-11-22; `docs/plans/2026-11-22-playtest-todo.md` 16)

- [x] S.1 (measured as S.4 instead: the walk cold, warm and with the cache off) `SWAMP_TIME`: the pool test's and the swamp's river test's calls and time in a rebuild, measured cold and warm, in a swamp and by a river.
- [ ] S.2 Memoize `swamp_pool_held` and the swamp's `rivers::beside` in the river cache's store and file; tests cold, warm and from the file in a swamp.
- [ ] S.3 Filled ahead with the rivers after READY.
- [x] S.4 Measured again; before and after reported (2026-11-23, fast build, RTX 3070, a 600 m `--walk` from `--at -14.433 -7.594`, yaw 0, F3 graph and `--frame-log` on, one game at a time; the owner's cache folder backed up and put back):

  | Run | fps | frame p50 / p90 / p99 ms | frames over 50 ms | fine set lands after |
  | --- | ---: | --- | ---: | --- |
  | `PBD_RIVER_CACHE=off` (no memo: the old cost) | 94 | 12.3 / 14.3 / 33.0 | 37 | 3.4 - 4.0 s |
  | cache on, empty folder | 92 | 12.6 / 14.8 / 33.1 | 34 | 3.7 - 4.4 s |
  | cache on, warm (the folder the cold run left) | 89 | 13.1 / 15.0 / 33.1 | 33 | 3.5 - 4.3 s |
  | no swamp: the default spawn, same walk | 80 | 13.9 / 17.2 / 32.8 | 39 | 3.3 - 4.6 s |

  The swamp fill runs on its own thread: cold, 1.0 - 2.4 s per 200 m patch (up to 31,000 cells); warm, 0 cells to fill. Finding: the cache changes no frame number and no load time a player would see, and a swamp loads no slower than anywhere else. The wait the owner saw is the fine set itself: about 3.5 s from request to landing everywhere, on 2 threads, in the fast build. That is the next thing to measure (a release build, the fine set's build split by step), written up before any change.
  The `PLACE spawn` line reports a different latitude and longitude on each launch for the same `--at` (it drifts about 6 degrees a launch, with the clock): the place is the same (the warm run found every swamp cell already in the file), the report is not. A bug in the log, noted, not fixed.
