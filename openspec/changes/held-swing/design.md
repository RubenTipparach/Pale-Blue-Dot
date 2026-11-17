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
