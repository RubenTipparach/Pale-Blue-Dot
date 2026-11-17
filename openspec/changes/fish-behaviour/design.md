## Context

The fish that comes to the lure is a fish of a school (`fauna/school.rs`).
The school steps at 30 Hz (`spawn.step_hz`); the line (`fishing.rs`) steps with
it and, from the first nibble to the landing, sets `School::held` to the lure
point 0.35 m under the float. The drawing (`pbd-app/src/fish.rs`,
`draw::build`) places each fish at `School::drawn` (between its last two steps,
`movement-smoothness`) and turns it along `School::velocities[i]`, the last
step's velocity, with the vertical part halved. The tail is `bodies::sway`,
`Sway::Fish`: `sin(t * 7 + phase - x * 6) * 0.075 * w^2`, always.

Mockup: M37, `docs/mockups/fish-behaviour.html`,
https://claude.ai/artifact/BkznFkfte6Ph8GLKWfczUz. Its Today side is a port of
the code above with the numbers of `assets/config/fauna.ron`; its Proposed
side is this design.

## 1. What is unnatural, in code terms

Frames of the owner's video at 1/4 speed show it, and Today reproduces it.

| What the owner sees | Why |
| --- | --- |
| The fish snaps onto the lure and stays there | `School::step`: a held fish's position is set to the lure. It is steered at the lure point itself (`flock.lure` 3.0) with a speed floor of half its slowest speed, so it arrives at full speed; the step that takes it inside `nibble_m` (0.45 m) moves it the rest of the way, about 13 m/s at 30 Hz. |
| It stands on its nose or tail at the float | A held fish's velocity is `(lure - position) / dt`, the lure's own movement. While the float dips (`nibble_dip_m`, `bite_dip_m`, eased at `dt * 14`), that is a small vertical vector; `normalize_or` makes it a unit vector, so the fish points straight up or down. When the lure is still the velocity is zero and the fallback is `up.any_orthonormal_vector()`, one fixed heading for every fish. |
| Rotation is snappy | Position is drawn between steps, heading is not: it changes at 30 Hz in jumps, and a boid that reverses does so in one step. |
| No approach, no pecks, no take | The nibble and the bite exist only as float dips; nothing in the fish's motion is tied to them. |
| Robotic swimming | Speed is clamped to the species' range and otherwise constant; the tail beat is fixed at 7 rad/s (1.1 Hz) and 0.075 of a body length whatever the speed; the body never rolls. |
| The fight looks wrong | On the line the fish is held at the lure again, so its facing is the float's direction of travel: backwards while reeled in, and a flip whenever the float stalls between run and rest. |

## 2. Proposed: the fish at the lure, phase by phase

The fishing rules decide when; the behaviour decides how. Every proposed
motion is read off the line's `Phase` and `timer`, so the timing a player
learns (watch for the third dip, strike when the float goes under) is
unchanged.

| Game phase or event | Proposed | Where |
| --- | --- | --- |
| `Event::Interest`, `Floating` with a target | Leaves the school and steers, turn-rate limited, to a stand-off point: on the side it came from, level with the lure, its mouth 15 cm off. Arrival: desired speed falls with distance inside a slow-down radius (1.3 m); no speed floor for this fish. Turns to face the lure as it settles. | sim: `School` steers the lured fish to the stand-off point, not the lure |
| `Floating` to `Nibble` | The rule (`distance < nibble_m`) is unchanged; the stand-off point is inside it. | none |
| `Nibble`, each timer | Hover: drift of a few cm on a slow cycle, tail sculling. Peck: start a dart `dart` (0.14 s) before the timer reaches 0.12 s, so the mouth meets the lure exactly when the game dips the float; head shake (12 deg at 7 Hz) and a tug of 2 cm on the lure while the dip lasts; back off to the stand-off point over 0.35 s. Between pecks, when the next timer leaves room (about 0.45 s), it may turn away and swim a small loop (radius 0.2 m) and come back facing the lure. | draw: a choreography keyed on `phase`, `timer`, the nibble index |
| `Event::Bite`, the hook window | The take: dart onto the lure (0.08 s), a hard turn (520 deg/s) and a run down (22 deg) and away from the angler. The drawn lure goes with the mouth, up to 0.3 m, and pulls the float aside on top of the game's 14 cm dip. | draw: the lure offset is drawn only; the rule's lure point is unchanged |
| `Event::TooEarly`, `Event::Missed` | Bolt: a burst away from the lure at 1.6 times top speed, easing over 1.2 s, then the fish rejoins the school's steering, still spooked (the game already frightens it). | draw, then sim (hand the drawn position and velocity back) |
| `Event::Hooked` | See 3. | |

## 3. Proposed: on the line

`Line` in `Hooked` already models runs (0.8 to 2.0 s at the species' `pull`)
and rests (1.2 to 3.0 s), tension, stamina and the float's swing across the
line. The fish is drawn from that, never changing it:

- its **mouth** is at the lure and its body swings round that point;
- **running** (`run > 0`): head away from the rod along the line, diving
  14 deg, full tail beat;
- **resting, being reeled** (`run == 0`): the hook turns the head toward the
  rod; head shakes (18 deg at 3.5 Hz) and a roll (30 deg) that flashes the
  flank;
- the turn between the two is rate-limited (300 deg/s) and eased (0.22 s), so
  a change from run to rest is a turn about the mouth, not a flip.

## 4. Proposed: every fish

- **Heading** is a per-fish pose, turned every frame toward the drawn
  velocity by yaw and pitch, each eased (0.12 s) and rate-limited
  (260 deg/s), pitch held inside 35 deg. Turning by yaw and pitch, not about
  the cross product, means a half turn goes round level and never through
  nose-down.
- **Bank**: roll toward the inside of a turn, 14 deg at 90 deg/s, capped at
  28 deg, eased over 0.18 s.
- **Tail**: frequency `1.3 Hz + 1.0 Hz per body length a second`; amplitude
  0.025 hovering, 0.05 cruising, 0.11 bursting, 0.012 gliding (in body
  lengths, today's sway shape).
- **Sim side**: the flock step limits how far a fish's velocity turns in a
  step (220 deg/s); each fish swims a burst-glide cycle of its own (1.8 s,
  35% burst, glide drag 0.9/s) inside the species' speed range; and wanders
  up to 0.18 m in depth round the school's goal.

## 5. Port plan

**Draw side** (`pbd-app`): a `FishPose` per drawn fish (heading, roll, tail
phase and amplitude, yaw rate), kept in a component beside the school's mesh
and keyed by the fish's stable id so a fish removed from the school does not
shift another's pose; `draw::build` turns it every frame instead of taking the
last step's velocity; `bodies::sway` takes the tail phase and amplitude. The
choreography of the line's fish (2, 3) is a function of the line's phase,
timer, nibble index and the frame's lure, evaluated every frame. It needs the
nibble index and the time since the phase began on `Line` (read-only
accessors, no rule change).

**Sim side** (`pbd-core`, deterministic, 30 Hz): the lured fish steers to the
stand-off point with arrival and no speed floor; the turn limit, the
burst-glide cycle and the depth wander in `School::step`; values in a new
`behaviour` block of `fauna.ron`, with units. School state is not saved, so
no save format moves. A per-fish phase is hashed from the school's seed and
the fish's id, as its length is.

**Unchanged**: `Line::step` and its rules, `FishingSettings`, the hook
window, what a strike does, the events, the HUD and the sounds that hang off
them.

**Interaction with `movement-smoothness`**: its requirement "a fish held at
the lure SHALL be drawn at the lure" becomes, once this lands, "its mouth is
at the lure"; that change's spec is updated in the same commit as the game
code.

## 6. Risks

- The nibble dip is detected per 30 Hz step; the peck is timed against the
  frame's interpolated timer, so the mouth may meet the lure up to a step
  early or late. At 1/4 speed in the mockup it lines up.
- The loop between pecks needs about 0.45 s of gap; with `nibble_s` at its
  shortest (0.6 s) there is no loop, only hover. That is the timing rule, left
  alone.
- The tug and the take move the drawn lure and float, not the rule's. A
  multiplayer peer sees the same drawn motion only if the choreography stays a
  pure function of line state, which this design keeps.

## 7. Open questions

Asked only with the mockup beside them (CLAUDE.md, "Ask the owner with a
survey"): the owner's review of M37 on the tracker answers whether the
proposed behaviour is right and which slider values to take.
