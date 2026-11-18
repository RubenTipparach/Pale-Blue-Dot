# Tasks

## 1. The hand's faces

- [x] 1.1 `hexel::prism`: shade each side face by its true normal; a test that
  a round prism is unchanged and a flattened one's slanted faces point along
  the edge's normal
- [x] 1.2 Capture the hand before and after (`docs/screenshots/rod-motion/`)

## 2. The rod moves

- [x] 2.1 `held.ron` `rod` block, validated, with defaults in one place
- [x] 2.2 The rod's pose from the line's phase, events, charge and tension; a
  spring between targets
- [x] 2.3 `RodTipNow`: the line and the cast leave the tip as drawn
- [x] 2.4 Tests: the pose's target per phase, the spring settles, the tip
  follows the pose
- [x] 2.5 Captures of charge, cast, bite and fight
- [x] 2.6 The overhead cast (decision 7): the fist raised above the shoulder
  on an arc, in the mockup and the game; `cast_raise_m`, `cast_back_m`,
  `cast_reach_m` in place of `draw_lift_m`; a capture of the charge
  (`docs/screenshots/rod-motion/charging-raised.jpg`)
- [x] 2.7 A technical montage of the mockup's motion
  (`docs/videos/tool-motion/`), fixed-step on SwiftShader

## 3. The mockup

- [x] 3.1 `docs/mockups/tool-motion.html` (M36), Fish tab: a Today / Proposed toggle, phase
  buttons, hold to cast, day and night, sliders with the `held.ron` printout
- [x] 3.2 Publish it, add it to the tracker, record it in `mockup-work.md`

## 3b. The flex, the follow-through and the cast's line (design 8)

- [x] 3b.1 `RodSettings` and `held.ron`: `flex` 0.07, `bend_max_rad` 0.6, `bend_hz` 4.5, `bend_damping` 0.3, `follow_rad` 0.55, `follow_reach` 0.8, the defaults equal
- [x] 3b.2 The rod's mesh bends past the fist by bend L s^2 on the CPU when the bend changes (`held::bend_rod`); the bend a damped spring toward -flex times the rod's own pitch rate, capped; the line and the cast leave the bent tip (`RodMotion::tip`)
- [x] 3b.3 The follow-through: while the float flies the rod's target is -follow_rad with the arm at -follow_reach of the arc
- [x] 3b.4 The cast's line: the float's places every 0.15 m while it flies (`fish::draw::CastTrail`, drawing only), the line laid along tip, release and trail, eased to the hanging curve over 0.6 s after landing
- [x] 3b.5 Tests: the bend's response and cap, the follow-through target, the line's trail from the tip to the float
- [x] 3b.6 Captures of a cast in flight before and after (`docs/screenshots/rod-motion/`)

## 4. Close

- [x] 4.1 Checks: fmt, clippy, tests, `openspec validate --all`
- [ ] 4.2 The owner's look at the mockup and the game (frame cost not measured
  in the cloud: no GPU)
