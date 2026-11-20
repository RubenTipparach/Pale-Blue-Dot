# Design

## Measured (2026-11-17)

- `held::hold` turns the tool by `phase.sin()` while `Swinging` is true;
  `phase` grows at `swing_rate` 9 rad/s, so a cycle is 0.70 s. The strike
  reaches the block in 0.175 s, the back-lift is about 10 degrees. A sine is
  fastest when it leaves and slowest when it lands: that is the weak look.
- `Swinging` follows `mining.progress().is_some()`; there is no per-hit timer.
  The stone axe takes 0.6 s on wood, so the block breaks before one swing ends.
- Dig sounds play at the start and each third of progress
  (`desktop/sounds.rs`), unrelated to the swing.
- On stop the phase resets to 0: the tool snaps to rest.

## Decisions

1. A stroke curve in `held.ron` replacing the three `swing_*` values:
   `windup_s` 0.08 / `windup_rad` 0.35, `strike_s` 0.07 / `strike_rad` 1.15
   (eased in, t cubed), `impact_hold_s` 0.05, `recoil_rad` 0.12,
   `recover_s` 0.18, `stroke_target_s` 0.4, `drop_m` 0.09, and
   `camera_kick_rad` (0 unless the owner wants it).
2. One function, `stroke_at(progress, break_s) -> (stroke, phase)`, gives the
   stroke and its stage from the dig's progress. `hold` draws from it and the
   sound code plays on its impacts: one schedule, two readers.
3. The hold on impact holds the tool only. Game time and the dig run on.
4. The mockup (Tools in Hand, M36, `docs/mockups/tool-motion.html`) shows
   TODAY and STRIKE in each of its Dig and Chop tabs' one view (a toggle),
   with the curve's values as sliders.
5. One steady beat from block to block (owner, 2026-11-17, on the Tools in
   Hand mockup: "why is it in sets of 3?", "it should be like continous").
   Timing each block's N strokes at progress k/N left the 0.15 s
   `between_s` pause, plus a whole stroke, between a block's last impact and
   the next block's first: three hits, a gap, three hits. Instead a block and
   its pause, L = T + between_s, take N = max(1, round(L / stroke_target_s))
   strokes of one period P = L / N. The k-th impact of cycle c lands at
   c·L + T − (N − k)·P, so the last one still lands on the break, and the
   strokes run on at one beat while the button is held. Break times do not
   change.
6. The arm swings from the shoulder (owner, 2026-11-17).
   - The owner's words: "I think hand should extend outward when doing
     actions", pointing at a first-person chopping animation
     (https://sketchfab.com/3d-models/chopping-animation-8f4e1c3d4ec74e7580eae93d045e3581).
     Then at Quaternius's Universal Animation Library viewer
     (https://quaternius.com/animviewer.html), its Sword_Attack_Standing:
     "thats a sword but similar idea".
   - What is wrong today: pitching the tool about a fixed fist reads as a
     wrist flick.
   - The motion is reference only. No asset is taken from either page.
   - The fist moves on an arc about a virtual shoulder (`shoulder`, eye
     space, about 0.2 m right, 0.25 m down and 0.05 m behind the eye):
     - on the wind-up it rises up and back toward the shoulder
       (`raise_rad`);
     - on the strike it sweeps forward and down to full reach at the block
       (`reach_rad`, the arm lengthening by `extend_m`);
     - it follows through a little past the impact (`follow_rad`);
     - it returns with the recover.
   - The tool's own pitch rides on top.
   - The sleeve is long enough that the arm still runs off the screen at full
     reach.
   - Shown in the Tools in Hand mockup's Proposed strokes before the game.
7. Four stages, by the animation principles (owner, 2026-11-17: "animation
   principles: windup, anticipation, hit, recoil"):
   - **Windup** (`windup_s` 0.10): the arm raises the tool up and back.
   - **Anticipation** (`anticipate_s` 0.07, `anticipate_rad` 0.08): at the
     top the tool settles a little further back and pauses, the beat before
     the release. Decision 1's curve had no such stage: the wind-up ran
     straight into the strike.
   - **Hit** (`strike_s` 0.06, eased in with t cubed, then `impact_hold_s`
     0.05): the fast sweep to full reach and a held frame on contact.
   - **Recoil** (`recoil_rad` 0.15 over `recoil_s` 0.06, then `recover_s`
     0.16): the tool bounces back off the block, with a slight overshoot,
     then eases to rest or into the next stroke.
   - The whole stroke fits inside the steady beat's period (decision 5). When
     it would not, the windup, the anticipation and the recover shorten in
     proportion; the hit never does.
8. The twelve principles of animation (owner, 2026-11-17,
   https://en.wikipedia.org/wiki/Twelve_basic_principles_of_animation). They
   apply to every motion in hand: the dig and chop strokes and the rod
   (`rod-motion`).

   **Already in decisions 6 and 7:**
   - **Anticipation:** the coil before the hit, and the rod's draw-back
     before the cast.
   - **Arcs:** the fist moves about the shoulder, and the rod tip arcs on the
     cast.
   - **Slow in and slow out:** ease into and out of the held poses. The hit
     is fast in, with no ease at contact.
   - **Staging:** the tool head lands at the crosshair.

   **Added:**
   - **Follow-through and overlapping action:** the head lags the fist on
     the wind-up and carries a little past it at the hit (`lag_s` 0.03,
     `overlap_rad` 0.06). The rod tip overshoots and wobbles after the
     flick.
   - **Squash and stretch:** the arm stretches at full reach (`extend_m`)
     and gathers at the top.
   - **Secondary action:** chips, the hit marker, an optional camera kick,
     and the breathing sway between strokes.
   - **Timing:** a heavier tool winds up slower and holds the impact
     longer.
   - **Exaggeration:** one `exaggerate` value (default 1) scales the raise,
     the recoil and the kick together.
   - **Pose to pose, solid drawing and appeal:** four keyed poses (rest,
     top, hit, recoil) blended by the curves, and the fist reads as solid in
     each.
9. The shovel tips down, slides into the dirt and scoops (owner, 2026-11-17: "the
   shovel should go down fast, lift slightly like its lifting dirt", then on
   the mockup: "shovel in the mockup looked like it was smashing dirt, the
   spade definiely need to take a stteep angle approach to dirt, angle up and
   lift"). It has no overhead chop arc:
   - **Tip down (windup):** the spade tips down from rest until its tip
     points at the ground ahead (`entry_deg`, 6° below level: the spade lies
     low and long in front, the way the digger in the owner's reference GIF
     holds it), and the fist comes low and a little back (owner, 2026-11-17, on version 5: "Digging
     is still wrong shovel tips down and should move forward like a sliding
     motion into the dirt, tip and pull up dirt").
   - **Slide (hit):** the spade slides forward along the ground into the dirt,
     tip first, at that angle (`slide_m` 0.28 forward, sinking `slide_drop_m`
     0.03). It is a push, not a stab: the steep 65° drive along the handle
     of version 5 is replaced.
   - **Tip (recoil):** the handle levers back about the blade's point in the
     dirt (`lever_rad`, 0.5). The spade's face turns up under the dirt.
   - **Pull up:** the spade rises and draws back toward the player with a
     clump of dirt on it (`lift_m` 0.14 up, `pull_m` 0.05 back).
   - **Return:** the clump tips off as the spade returns to rest.
   - **A straight wrist:** the arm follows the spade's whole turn
     (`arm_follow` 1) and the fist stays on its own side (`across_m` 0), so
     the wrist never bends back against the handle (owner, 2026-11-17: "looks
     like it hurst the angle"). Low in the stroke the hand leaves the bottom
     of the view, and that is fine (owner: "that might be a situation where
     the camera just cant see the hand, which is fine ... if the player looked
     down far enough they would see the arm digging").
   - **The shoulder stays low** (owner, 2026-11-17, on version 10: "with
     shoveling the shoulder is too high. I think the arm has to be lowered
     more. Remember I said it's ok if shovel animation moves hand off screen
     for a split second"): the fist drops 0.16 below rest at the hit
     (`fist_drop_m`, was 0.04) and the arm follows 0.7 of the spade's turn
     (`arm_follow`, was 1), so the sleeve no longer climbs the right of the
     view; the hand leaves the bottom of the view through the tip-down and
     the slide and comes back with the lift.

   The proposed hand's thumb has two segments, not three (owner, 2026-11-17:
   "Two thumbs segments not 3"); that is `hand-model`'s.

   It keeps the four stages (decision 7) and the steady beat (decision 5).

## As built (2026-11-17)

Ported from Tools in Hand version 8 (`docs/mockups/tool-motion.html`, the
Proposed mode) into `crates/pbd-app/src/held/stroke.rs`: the same curves
(`timings`, `keys`, `strokeAt`, `strokePose`, `shovelKeys`, `shovelAt`,
`Schedule`) and the same defaults, in `held.ron`'s `stroke` and `shovel`
blocks as the mockup's Copy RON prints them. Decision 1's three `swing_*`
values are gone. Where it differs from the mockup, and why:

- **The beat on the game's rule.** The mockup runs one clock while the
  button is held. The game's dig (`pbd_core::dig::Breaking`) restarts each
  block, so the stroke's clock is the cycle count times L plus the rule's
  time into the block and its pause (`DigBeat`, written by `digging.rs` from
  `Breaking::timing` and `pause_left`; read only). The rule's own events the
  mockup has no button for (aimed at another block mid-way, a tool changed, a
  different break time) start the beat again from nought, and the tool eases
  there from where it is drawn over `recover_s`. Taking the dig up again
  while the tool is easing back does the same, where the mockup snaps it to
  rest.
- **The shovel's keys stay in eye space** as the mockup lays them out for its
  Dig tab's view, which looks down `DIG_PITCH` (-0.56 rad). The game does
  not re-aim the entry by where the player looks.
- **The arm bend** (`arm_follow`) turns the arm's meshes (the wrist, the cuff
  and the sleeve) about the wrist joint, as the mockup's `HandPair` does; at
  the approved 1 it stays straight.
- **Sounds** (2.2): each impact but a block's last plays the dig sound; the
  last lands on the break, which plays the break sound. The thirds of
  progress are gone. Bare hands keep the beat for the sounds and draw
  nothing, as before.
- **Not in the game:** the camera kick (approved at 0; `held.ron` refuses any
  other value, since the walking camera's look is the player's raw mouse),
  and the mockup's chips flying off each hit and the hit marker. The
  shovel's clump of dirt is in, shaded as the tool is, without the mockup's
  burst of four chips as it tips off.
- **Weights:** a tool's weight at its tier uses `Tier::digit` (stone 0 to
  steel 3), the mockup's tier index.

Frame cost was not measured: the work was done in a cloud container with no
GPU (CLAUDE.md). The captures are on lavapipe.

## As built (2026-11-18): the shoulder stays low

Tools in Hand version 12's shovel in the game (owner, 2026-11-18: "cool now
implement all these"). `held/stroke.rs` already had `shovelKeys` and
`shovelAt` as the mockup now has them (tip down to `entry_deg` 6, the slide
forward along the ground, the lever about the blade's point, the pull up
and back); the change is two defaults, in the code and in `held.ron`:
`fist_drop_m` 0.04 to 0.16 and `arm_follow` 1 to 0.7. The wrist takes the
other 0.3 of the spade's turn, about the wrist joint as the mockup's
`HandPair` turns it. No difference from the mockup.

Frame cost was not measured (a cloud container, no GPU).

## Punch, and a swing on every click (2026-11-19)

The owner, 2026-11-19: "do tools always play animations when left clicking?
can you mock up hand punching stuff". Shown in the Punch tab of Tools in
Hand, version 14, the fist re-posed to the owner's photo in version 15 (`docs/mockups/tool-motion.html#punch`), beside today's.
Nothing in the game changes until the owner approves it there.

### Measured

- A tool strokes only while the dig runs. `held/stroke.rs`
  `StrokeClock::step` reads the dig's `DigBeat`, which `desktop/digging.rs`
  sets from `mining.breaking.timing()` while a block or a tree is breaking
  (and through the pause after a break). With nothing under the crosshair,
  out of reach, on water, or on a block the digger is refused on, the beat
  is `DigBeat::default()`, `working: false`: the tool eases to rest and
  stands still however often the button is clicked.
- Bare hands draw nothing. `held.rs` `stroke_tool` maps no tool to
  `StrokeTool::Hands`, kept for the dig sounds' beat, and the held meshes
  are hidden when nothing is in hand.
- What bare hands do (`dig.ron`, `tree-punch-shake`): a log takes
  `wood.axe` over `hand_speed`, 0.6 / 0.25 = 2.4 s; soft ground the
  shovel's time over it, 0.5 / 0.25 = 2.0 s; stone, rock and ore are
  refused (`Refusal::NotByHand`, the note "Too hard to dig by hand"). No
  digging tool is refused today (the stone tier reaches ore), so the hands
  are the only refused digger; the rod breaks nothing.
- The tree shakes on each log and, while held on it, every 0.5 s
  (`tree_shake::HOLD_HIT_S`), a timer of its own, on no stroke: 12 cm at
  the crown, 9 Hz, gone in 0.6 s, leaves 1.3x (`SHAKE_M`, `SHAKE_HZ`,
  `SHAKE_S`, `SHAKE_LEAF`).

### Decisions

10. **Bare hands punch.** The hand is the approved hand (`hand-model`),
    posed from the pickaxe's grip as the game already poses bare hands, with
    no tool and the grip's hollow closed by a plug in the palm's tone, so the
    fist reads solid. It punches in the four stages (decision 7), on the
    dig's steady beat (decision 5), the impacts on the rule's events: a log
    by hand is 2.4 s and the 0.15 s pause, six punches of 0.425 s, the sixth
    on the log; dirt by hand five of 0.43 s, the fifth on the break.
    - **The fist, matched to the owner's photo** (version 15; owner,
      2026-11-19, on version 14: "Fisty position is wrong. See reference").
      The reference is the owner's own right fist, seen first-person as it
      meets a hanging cloth ([`ref/owner-fist-punch.jpg`](ref/owner-fist-punch.jpg)),
      and what version 14 drew is on their phone
      ([`ref/owner-punch-v14-phone.png`](ref/owner-punch-v14-phone.png)).
      - In the photo: the forearm comes in long from the lower right and
        runs diagonally up and left to the target; the wrist is straight, in
        line with it; we see the back of the hand and the top of the wrist;
        the fist is closed and stands upright, the fingers' middle segments
        stacked in a column on its leading (left) edge toward the target,
        the knuckles leading; the thumb is tucked under and barely shows; it
        meets the target at the screen's centre-left.
      - What was wrong in version 14: the fist was aimed from the shoulder
        (`stroke.shoulder`, 0.32 m from the eye), so the forearm pointed
        almost straight away from the eye and was hidden behind the fist (a
        floating lump); the hand's frame was taken from the grip's bones
        without the grip's turn (`grip_turn`, 45 degrees), and the guard's
        twist turned it thumb up with the fingers facing the camera; the
        wrist kept the pickaxe grip's bend, so the forearm ran off at an
        angle to the back of the hand.
      - The grip: the same approved hand (`hand-model`), re-posed, not a new
        hand. The fingers stay closed on the grip's hollow, closed by the
        plug, now a shade of the skin (0.42 of the back's tone) so the gaps
        between the fingers read as creases, not light. The wrist is
        straightened: the wrist, cuff and sleeve turn about the wrist joint
        onto the back of the hand's line.
      - The frame: the forearm aims from a virtual elbow (`elbow_x` 0.49,
        `elbow_y` -0.41, `elbow_z` -0.68, eye space, m: low and to the
        right) through the fist, the elbow following `elbow_follow` 0.75 of
        the fist's travel, so the forearm keeps its slant and a long sleeve
        is in view from the lower right through the whole stroke. At twist
        0 the back of the hand faces the eye, the knuckles' column standing
        across the forearm with the little finger on top and the thumb
        tucked under; `guard_twist_rad` 0.25 turns it about the forearm at
        the guard and the hit's `twist_rad` 0.25 brings it back to 0.
      - The hand on the wrist: `bend_rad` 0.3 about the back of the hand's
        normal stands the column upright with the hand a little shallower
        than the forearm, as in the photo. The cock about the column turns
        the fingers' segments toward the eye (positive) or away:
        `guard_cock_rad` -0.2 at the guard, so the guard shows the back of
        the hand and not the fingers' fronts, `hit_cock_rad` 0 at the hit
        (more than about 0.1 opens the fist into a claw from the eye). The
        arm is turned back by both, so the forearm stays on its line.
      - The poses: the guard rests `fist_raise_m` 0.15 up and `fist_in_m`
        0.12 in from a tool's fist, lower right, the back of the hand to the
        eye; the hit lands the fist's leading edge at the crosshair's left.
    - **Windup** (`windup_s` 0.09, eased out): the fist draws back toward
      the chin, a little above and inside the shoulder (version 14 drew it
      toward the shoulder, which took the forearm out of view), `draw_m`
      0.08, `drop_m` 0, the wrist cocked a little,
      `cock_rad` 0.05.
    - **Anticipation** (`anticipate_s` 0.06): a further `anticipate_m` 0.025
      and a held coil.
    - **Hit** (`strike_s` 0.07, t cubed, then `impact_hold_s` 0.04): the
      fist drives `reach_m` 0.27 forward, `across_m` 0.23 in and `rise_m`
      0.16 up, to the crosshair's left, on a slight arc (`arc_m` 0.03),
      turning `twist_rad` 0.25 from its guard to the upright fist of the
      photo (version 14 turned it 0.6 toward palm down); `sweep_rad` 0 turns
      the aim sideways if wanted.
    - **Recoil** (`recoil_s` 0.07, `recoil_m` 0.06, with an overshoot, then
      `recover_s` 0.16).
    - The fist's frame comes from the grip's own bones after the grip's
      turn (index to little finger for the column, the back of the hand's
      bone for the knuckles' way, away from the fingertips for the back,
      the wrist bone's end for the joint), so a new hand keeps working.
    - `hands_weight` 0.8 scales the windup, the anticipation and the hold,
      as the tools' weights do: a bare hand is lighter and quicker.
    - The tree shakes on each stroke's impact, the hands' and the axe's, in
      place of the 0.5 s timer, so what is seen to land and what moves agree.
      It stays drawn only: the shake reads the stroke's impacts, which read
      the dig's beat, and no rule reads either. The log still comes on the
      break. (`tree-punch-shake`'s requirement, "sway from each hit", holds.)
11. **Every left click swings** (recommendation taken, ask only with
    screenshots; the Punch tab shows each case). A click with a digger in
    hand (a digging tool or bare hands) plays a stroke whatever is under the
    crosshair, and holding repeats it. Considered:
    - **A, recommended: always swing.** A block that breaks keeps the dig's
      beat. Anything else runs a free beat of `stroke_target_s` (0.45 s)
      from the press, the first impact one windup, anticipation and strike
      after it. When the aim comes onto something that breaks, the beat
      moves to the dig's, easing from the drawn pose as a change of block
      does today.
    - B, one swing per click and nothing more while held: reads as stuck
      when held at the air.
    - C, today: nothing; the owner's question shows it reads as broken.

    By what the stroke meets (the `swing` block, tools and hands alike):
    - **Air:** no hold, since nothing stops it. The hit carries through,
      `air_follow_m` 0.07 further for the fist and `air_follow_rad` 0.3
      further down the arm's arc for a tool, over `air_follow_s` 0.1, eased
      out (a whiff), and the recover is `air_recover_k` 1.3 slower. A tool's
      follow-through goes into the arm's angle with the tool's own pitch
      falling with it: turning the whole holder swings the sleeve up across
      the view (seen in the mockup and fixed there).
    - **Refused** (a thud): the hit stops dead and holds `thud_hold_s` 0.07,
      kicks back further (`thud_recoil_m` 0.05 for the fist,
      `thud_recoil_rad` 0.2 for a tool), and a jolt runs through the hand,
      `thud_shake_m` 0.008 at `thud_shake_hz` 22 over `thud_shake_s` 0.18,
      drawn on top of the pose. No cracks; the block does not move. The note
      ("Too hard to dig by hand") still comes once, on the press.
    - Out of reach and water count as the air.
    - The rod casts and is not a digger: unchanged.
12. **Sounds and secondary action.** A whoosh on each air swing's strike and
    a dull knock on each thud's impact, from the same schedule as the pose
    (decision 2): new cues by the `sound-effects` skill. No chips or hit
    marker (the game has none; see the follow-ups).

The `punch` and `swing` blocks go into `held.ron` beside `stroke` and
`shovel`, validated as `stroke` is (finite; sizes and times non-negative;
`strike_s`, `recoil_s`, `recover_s` and `air_follow_s` positive;
`air_recover_k` at least 1), with one source of defaults. The mockup's Tune
prints them:

```ron
    punch: (
        windup_s: 0.09, draw_m: 0.08, drop_m: 0.0, cock_rad: 0.05,
        anticipate_s: 0.06, anticipate_m: 0.025,
        strike_s: 0.07, reach_m: 0.27, across_m: 0.23, rise_m: 0.16,
        twist_rad: 0.25, sweep_rad: 0.0, arc_m: 0.03,
        impact_hold_s: 0.04, recoil_s: 0.07, recoil_m: 0.06, recover_s: 0.16,
        hands_weight: 0.8,
        fist_raise_m: 0.15, fist_in_m: 0.12, guard_twist_rad: 0.25,
        guard_cock_rad: -0.2, hit_cock_rad: 0.0, bend_rad: 0.3,
        elbow_x: 0.49, elbow_y: -0.41, elbow_z: -0.68, elbow_follow: 0.75,
    ),
    swing: (
        air_follow_s: 0.1, air_follow_m: 0.07, air_follow_rad: 0.3, air_recover_k: 1.3,
        thud_hold_s: 0.07, thud_recoil_m: 0.05, thud_recoil_rad: 0.2,
        thud_shake_m: 0.008, thud_shake_hz: 22.0, thud_shake_s: 0.18,
    ),
```

### In the game, once approved

- `digging.rs` writes what the click meets beside the beat: `DigBeat`
  gains `meets` (`Breaks`, `Air`, `Refused`) and the press time for the
  free beat, and `working` follows the button (a digger in hand, the
  pointer the walker's), not only the dig. Read only, as now: no rule reads
  it back.
- `StrokeClock::step` runs the free beat when nothing breaks and the dig's
  beat when something does, easing between them as it eases a change of
  block.
- `StrokeTool::Hands` draws: the held hand is shown with no tool mesh and
  with the fist's plug, posed by a `punch_pose` beside the stroke's.
- `tree_shake`: a hit comes from a `StrokeImpact` on a tree, and
  `HOLD_HIT_S` goes.
- Tests pin the schedule: six punches per log by hand, the sixth on the
  log; the free beat's first impact one windup, anticipation and strike
  after the press; no hold at the air; the thud's hold; no pop when what the
  click meets changes.
