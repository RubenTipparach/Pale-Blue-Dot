# Tasks

## 1. Write-up
- [x] 1.1 Proposal, design, spec delta (from the near-ground pricing sitting).

## 2. Build
- [x] 2.1 `rain_renderer` setting and `PBD_RAIN_RENDERER` override; the Volume path unchanged.
- [x] 2.2 Streaks and flakes from a hash per column (design "As built"); a test holds every cycle to the fall's wrap.
- [x] 2.3 The lean and the accumulated fall (`rain_shell_step`), tested: still air falls at its own speed, flight rushes past at the slanted speed, it wraps, a bad frame does not poison it.
- [x] 2.4 `rain_shells` in the volume's pass, behind `rain_shell.w`.
- [ ] 2.5 The shader test's validator accepted `from` as a name; the game's naga refused it at run time and every composite pass went missing. Make the test validate with the game's naga.

## 2b. Distant rain (design 6)
- [ ] 2.6 Curtains per raining puff site: the layout in `pbd_core::cloud_puffs`, with tests (no precipitation gives no curtain, a curtain spans base to ground).
- [ ] 2.7 The curtain's look in the puffs' pass; the hand-over from the shells at 1.2-2 km.
- [ ] 2.8 Captures of a storm from 5 km and from 1.5 km, and a `storm` sitting.

## 3. Verify
- [ ] 3.1 Captures, volume against shells (design 5): 2 m and 150 m in rain taken; a far storm, snow and looking up still to take.
- [x] 3.2 `perf_suite` sittings (design 5); report in `docs/benchmarks/2026-10-28-rain-shells/`: near the ground the shells cost 1.4 ms at p50 over no rain, the volume 3-4.5 ms.
- [ ] 3.3 fmt, clippy, tests, `openspec validate --all`.
- [ ] 3.4 The default renderer to the owner, with the captures (R1).
