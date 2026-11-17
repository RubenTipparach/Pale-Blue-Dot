# Tasks

## 1. Mockup

- [x] 1.1 M37 Fish Behaviour: Today (the game's rules and drawing, ported)
  beside Proposed, three tabs, sliders with units, night lit by a lantern.
- [x] 1.2 Published and listed on the tracker.
- [ ] 1.3 The owner's approval of M37 (the tracker's review, or their words).

## 2. Draw side (after approval, on request)

- [ ] 2.1 `FishPose` per drawn fish: heading turned by yaw and pitch, eased
  and rate-limited; bank from yaw rate; tail phase and amplitude from speed.
- [ ] 2.2 `bodies::sway` takes the pose's tail phase and amplitude.
- [ ] 2.3 The line's fish: approach settle, hover, pecks timed to the nibble
  dip, the take, the bolt, the fight, as a pure function of line state.
- [ ] 2.4 The drawn lure and float follow the tug and the take.

## 3. Sim side (after approval, on request)

- [ ] 3.1 The lured fish steers to a stand-off point with arrival.
- [ ] 3.2 Turn limit, burst-glide cycle and depth wander in `School::step`.
- [ ] 3.3 `behaviour` block in `fauna.ron`, validated, with units.
- [ ] 3.4 Tests: the nibble still starts inside `nibble_m`; the hook window
  and the events are unchanged (the existing `fishing` tests pass as they
  are); a fish's heading never turns faster than the limit.

## 4. Check

- [ ] 4.1 Before-and-after under-water video at 1x and 1/4x, the shots of the
  owner's video, beside the mockup (complex-bug rule: video, not stills).
- [ ] 4.2 Move the requirements into `openspec/specs/player/fishing` in the
  commit that makes them true.
