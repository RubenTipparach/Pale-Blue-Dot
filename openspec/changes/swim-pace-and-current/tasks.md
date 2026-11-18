## 1. Swim pace

- [x] 1.1 `assets/config/swimming.ron` and `SwimSettings` (units, validated;
      the shipped file is the defaults, test).
- [x] 1.2 A swimmer's speed times `swim_speed_mult`. Test: swimming is half
      the old speed, surface, under and sprinting; wading is not.

## 2. The push

- [x] 2.1 `GroundState::feet_down`, set by the sweep before the submerged
      rule.
- [x] 2.2 The drift eases toward half the water's velocity (capped) while
      in the water with the feet off the bottom, toward zero otherwise.
      Tests: drift direction is the flow's; zero push with the feet down;
      continuous across cells (no step over a tick's ease).
- [ ] 2.3 In the game: a river, swum across and floated with no input
      (drifts downstream), and stood on the bed (no drift); a trace. Open:
      the owner asked to validate later ("unit tests and fast build is fine
      for now", 2026-11-18).

## 3. Close

- [x] 3.1 Spec delta into `openspec/specs/player/walking/spec.md` with the
      commit that makes it true.
- [x] 3.2 fmt, clippy, tests, `openspec validate --all`.
- [ ] 3.3 Owner's call: the two `swimming` 4/5 tests that half the pace
      breaks (design, "Found"). Until then they fail.
