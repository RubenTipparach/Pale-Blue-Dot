---
name: animation
description: Design, mock up and build Pale Blue Dot's motion the repository's way - first-person tool strokes (dig, chop, till, hammer, cast, reel), held items, creatures and townsfolk, vehicles' moving parts - by the twelve principles of animation, with every action broken into windup, anticipation, hit and recoil, the limb moving on arcs about a joint, keyed poses blended by curves with named tunables in validated RON, timed to the game's own rules (one steady beat across repeated actions, the impact on the rule's event), drawn only and never read by a rule, shown in a mockup beside today's motion before the game changes. Use whenever an animation, a swing, a stroke, a cast, an attack, a walk or idle cycle, a creature's or NPC's motion, a held-tool pose or a "make it feel weightier / snappier / more visceral" request comes up, or the owner says a motion looks weak, stiff, robotic, floaty or wrong ("animate", "animation", "swing", "chop animation", "attack animation", "walk cycle", "idle", "make it punchier", "anticipation", "recoil", "follow through").
metadata:
  author: Pale Blue Dot (Claude Code), from the owner's notes on the Tools in Hand mockup (M36, held-swing and rod-motion), 2026-11-17
  version: "1.0"
---

# Animation

The owner, 2026-11-17, while reviewing the swings in Tools in Hand: "make up an animation skill too
so we can reuse these ideas for future tool animations, and various characters". The ideas came from
the owner's own notes that day:

- "man that axe swing is weak, it should be more visceral, more faster";
- "why is it in sets of 3?", then "it should be like continous";
- "I think hand should extend outward when doing actions", pointing at a first-person chopping
  animation (https://sketchfab.com/3d-models/chopping-animation-8f4e1c3d4ec74e7580eae93d045e3581);
- Quaternius's Sword_Attack_Standing (https://quaternius.com/animviewer.html): "thats a sword but
  similar idea";
- "animation principles: windup, anticipation, hit, recoil";
- the twelve principles (https://en.wikipedia.org/wiki/Twelve_basic_principles_of_animation).

The worked example is `openspec/changes/held-swing/design.md` (decisions 5 to 8),
`openspec/changes/rod-motion/design.md` and the Tools in Hand mockup (M36,
`docs/mockups/tool-motion.html`).

## The order of work

1. **Write it up first** (CLAUDE.md, "Write it up before touching code"). Open or extend an
   OpenSpec change with:
   - what is wrong with today's motion, measured: rates, angles, stage times, what it is
     driven by;
   - the stages and their curves;
   - the tunables, with names and units;
   - what game event each beat lands on.
2. **Mock it up** beside today's motion, in the game's own models, with a Today / Proposed toggle in
   one view, sliders for every tunable, and a printout of the RON block the sliders make
   (`mockup-tracker` for publishing; the mockup rules in CLAUDE.md: lit at night, nothing moves on
   its own until played, notes in the info modal, one 3D view per tab). A "Slow x0.25" switch and a
   "Show poses" switch (freeze on each keyed pose in turn) let the owner judge each stage.
3. **Wait for the owner's approval** on the tracker before the game changes, unless the owner asked
   for the game change outright. A look change is the owner's call ("Ask only with screenshots").
4. **Port the same curves** into the game, the tunables into validated RON with one source for
   defaults, so the mockup's printout pastes in unchanged. Tests pin the schedule (which stage at
   which time, the impact on the event, no pop on a state change).
5. **Capture it** in the game (`--capture`), before and after, in `docs/screenshots/<change>/`.

## Every action has four stages

| Stage | What it does | Curve |
| --- | --- | --- |
| **Windup** | The limb raises or draws the tool back, away from the target. | Ease out. |
| **Anticipation** | At the top it settles a little further back and *holds*: the coiled beat before the release. Never skip it; without it a strike reads as a flick. | Slow ease into a held pose. |
| **Hit** | The fast move to the target, then a short held frame on contact. | Ease *in* (t cubed): fastest at contact, no slowing before it. |
| **Recoil** | The bounce back off the target with a slight overshoot, then the ease to rest or into the next action's windup. | Quick kick, then ease out. |

The stage times are tunables (`windup_s`, `anticipate_s`, `strike_s`, `impact_hold_s`,
`recoil_s`, `recover_s`), and so are their sizes (`raise_rad`, `anticipate_rad`, `reach_rad`,
`recoil_rad`). When the stroke must fit a fixed period, shorten the windup, the anticipation and the
recover in proportion. Never shorten the hit.

## The twelve principles, as they apply here

- **Squash and stretch.** Rigid tools do not deform. The limb stretches at full reach (`extend_m`)
  and gathers at the top. A creature's body squashes on a landing and stretches on a leap.
- **Anticipation.** The four stages above. A cast draws back before the flick, and a jump
  crouches first.
- **Staging.** The working end lands where the player aims, at the crosshair, and stays readable
  against the target at the hit.
- **Straight ahead and pose to pose.** Pose to pose: key the poses (rest, top, hit, recoil) and
  blend them with the curves. Never hand-tune frames.
- **Follow-through and overlapping action.** The parts do not move as one block:
  - the tool head lags the fist on the windup and carries past it at the hit (`lag_s`,
    `overlap_rad`);
  - a rod tip overshoots and wobbles after the flick;
  - a tail, ears or a coat settle after the body stops.
- **Slow in and slow out.** Ease into and out of every held pose. The one exception is the hit, which
  arrives at full speed.
- **Arcs.** Limbs swing about joints:
  - the fist moves on an arc about a virtual shoulder (`shoulder`, eye space), never on a straight
    line;
  - a rod tip, a paw and a head all trace arcs.
- **Secondary action.** Chips and dust, the hit marker, an optional small camera kick (added on top
  of the view, never touching raw mouse look), the breathing sway between actions, a sleeve.
- **Timing.**
  - A heavier tool or a bigger creature winds up slower and holds the impact longer.
  - A lower tier is a little slower than a higher one.
  - A repeated action keeps one steady beat (below).
- **Exaggeration.** One `exaggerate` value scales the raise, the recoil and the kick together, so
  the whole feel can be pushed or calmed in one move.
- **Solid drawing.** Check each keyed pose from the player's view: the hand reads as a solid
  fist, nothing passes through itself, and the arm runs off the screen edge at full reach.
- **Appeal.** Judged by the owner on the mockup.

## Timed to the game's rules, drawn only

- **The beat comes from the rule.** A repeated action has one steady period. Example: digging a
  block of break time T with a pause g between blocks takes N = max(1, round((T + g) /
  stroke_target_s)) strokes of period P = (T + g) / N. The k-th impact of cycle c lands at
  c(T + g) + T - (N - k)P: the last on the break, and no gap between blocks. Grouping the
  strokes per block ("sets of 3") is the mistake that formula fixes.
- **Sounds and effects share the schedule.** One function gives the stage and the impacts from the
  rule's progress. The pose and the sound both read from it, so they cannot drift apart.
- **Drawn only.** The animation never feeds a rule:
  - the break time, the catch and the damage stay where they are;
  - the motion follows them and is never read back by them;
  - a hold on impact holds the drawing, never game time.
- **No pop.** A state change (letting go, a new phase) springs or eases from wherever the pose is.
  It never snaps. For phase-driven motion like the rod, a damped spring toward each phase's target
  (`spring_hz`, `damping`) with shakes and twitches added on top.
- **Things that follow a moving part follow it as drawn.** The fishing line leaves the rod tip
  where the rod is drawn that frame.

## Characters and creatures

The same stages apply to:
- a townsperson's hammer or hoe;
- a creature's peck, bite or headbutt;
- a fish's run on the line;
- a crane's jib.

The keyed poses are on the body's joints instead of the fist. Walk and idle cycles:
- contact, down, passing and up poses on arcs;
- the hips' rise and fall;
- the head and tail as overlapping action;
- the speed set by the gait's stride, never sliding feet.

Reference animations (Quaternius's libraries, Sketchfab and the like) are reference for motion only.
Never copy or embed their files unless their licence allows that use and the provenance is recorded
(CLAUDE.md, "Architecture and authority").

## Do not

- Move a tool only by pitching it about a fixed fist: that reads as a wrist flick.
- Use one sine wave for a strike: a sine is fastest when it leaves and slowest when it lands, the
  opposite of a hit.
- Let a stroke clock run on its own, apart from the rule it shows.
- Change a motion in the game before the owner has seen it in a mockup beside today's.
- Report a motion as smooth from a cloud capture: those are fixed-step and software-rendered.
  Smoothness is judged on the owner's machine (`obs-record`).
