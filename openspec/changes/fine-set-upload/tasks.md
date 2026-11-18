# Tasks

## 1. Write-up

- [x] 1.1 Proposal and design (this change). Verify: `openspec validate fine-set-upload`.

## 2. Build

- [ ] 2.1 The light as a cast (D1). Verify: `the_light_cast_is_the_packed_light`.
- [ ] 2.2 `TierImage`, `FineGpu`, `pack_fine_gpu` (D2) and the landing's image on its task (D3). Verify: `a_partial_update_writes_what_a_full_one_does` for an edit, a water change, a relight and a landing.
- [ ] 2.3 The upload writes ranges on its base only (D4, D5). Verify: `a_delta_is_used_only_on_its_base`; a version without its image is not uploaded.

## 3. Measure

- [ ] 3.1 `UPLOAD_PARTS` and `render: fine-set upload` before and after, release, on `--walk-distance` and `--walk --at 23.2255 -3.012 --stand`. Verify: `docs/benchmarks/2026-11-17-fine-set-upload/report.md`.
- [ ] 3.2 The same picture: `--capture` at the cascade and at a town, before and after, 0 differing pixels.
- [ ] 3.3 The frame-time A/B (`perf_suite.py`) on a quiet, unlocked machine.
