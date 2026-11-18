## 1. Swim pace

- [ ] 1.1 `assets/config/swimming.ron` and `SwimSettings` (units, validated;
      the shipped file is the defaults, test).
- [ ] 1.2 A swimmer's speed times `swim_speed_mult`. Test: swimming is half
      the old speed, surface, under and sprinting; wading is not.

## 2. The push

- [ ] 2.1 `GroundState::feet_down`, set by the sweep before the submerged
      rule.
- [ ] 2.2 The drift eases toward half the water's velocity (capped) while
      in the water with the feet off the bottom, toward zero otherwise.
      Tests: drift direction is the flow's; zero push with the feet down;
      continuous across cells (no step over a tick's ease).
- [ ] 2.3 In the game: a river, swum across and floated with no input
      (drifts downstream), and stood on the bed (no drift); a trace.

## 3. Close

- [ ] 3.1 Spec delta into `openspec/specs/player/walking/spec.md` with the
      commit that makes it true.
- [ ] 3.2 fmt, clippy, tests, `openspec validate --all`.
