# Tasks

## 1. Write-up

- [x] 1.1 Proposal and design (this change). Verify: `openspec validate fine-set-upload`.

## 2. Build

- [x] 2.1 The light as a cast (D1). Verify: `the_light_cast_is_the_packed_light`.
- [x] 2.2 `TierImage`, `FineGpu`, `pack_fine_gpu` (D2) and the landing's image on its task (D3). Verify: `a_partial_update_writes_what_a_full_one_does` for an edit, a water change, a relight and a landing.
- [x] 2.3 The upload writes ranges on its base only (D4, D5). Verify: `a_delta_is_used_only_on_its_base`; a version without its image is not uploaded.

## 3. Measure

- [x] 3.1 `UPLOAD_PARTS` and `render: fine-set upload` before and after, release, on `--walk-distance` and `--walk --at 23.2255 -3.012 --stand`. Verify: design, "Measured".
- [x] 3.2 The same picture: `--capture` at the cascade and at a town, before and after, 0 differing pixels.
- [ ] 3.3 The frame-time A/B on a quiet, unlocked machine: `python tools/perf_suite.py --scenarios cascade,cascade-walk,walk --repeats 2 --variant "before|<71b7868c exe>" --variant after`. Not run: the screen was locked for the sitting (the suite refuses).
- [ ] 3.4 A landing's level records staged on its task and copied on the GPU (design, "Measured", next). Write-up first.
