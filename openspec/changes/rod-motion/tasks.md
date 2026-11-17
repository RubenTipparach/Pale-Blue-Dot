# Tasks

## 1. The hand's faces

- [x] 1.1 `hexel::prism`: shade each side face by its true normal; a test that
  a round prism is unchanged and a flattened one's slanted faces point along
  the edge's normal
- [ ] 1.2 Capture the hand before and after (`docs/screenshots/rod-motion/`)

## 2. The rod moves

- [ ] 2.1 `held.ron` `rod` block, validated, with defaults in one place
- [ ] 2.2 The rod's pose from the line's phase, events, charge and tension; a
  spring between targets
- [ ] 2.3 `RodTipNow`: the line and the cast leave the tip as drawn
- [ ] 2.4 Tests: the pose's target per phase, the spring settles, the tip
  follows the pose
- [ ] 2.5 Captures of charge, cast, bite and fight

## 3. The mockup

- [x] 3.1 `docs/mockups/tool-motion.html` (M36), Fish tab: a Today / Proposed toggle, phase
  buttons, hold to cast, day and night, sliders with the `held.ron` printout
- [x] 3.2 Publish it, add it to the tracker, record it in `mockup-work.md`

## 4. Close

- [ ] 4.1 Checks: fmt, clippy, tests, `openspec validate --all`
- [ ] 4.2 The owner's look at the mockup and the game (frame cost not measured
  in the cloud: no GPU)
